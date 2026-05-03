# PhosphorOS — Operating System Layer Reference

PhosphorOS is the operating system layer above Phosphoric and Ember. It owns the runtime objects — tasks, IPC channels, capability handles, windows, framebuffers — and the per-tick boot loop that drives input → IPC → render dispatch.

This document describes the runtime layer's architecture, its capability model, its IPC model, and the demo loop that stitches it together.

---

## 1. Position in the Stack

```
┌─────────────────────────────────────────────────────────────┐
│ Application (apps/demo/)                                    │
│   - boot profile programs                                   │
│   - state machines, render commands, route outcomes         │
└─────────────────────────────────────────────────────────────┘
                            ↓ calls
┌─────────────────────────────────────────────────────────────┐
│ PhosphorOS (kernel/)                                        │
│   - capability table                                        │
│   - task table                                              │
│   - IPC channel table                                       │
│   - window table                                            │
│   - framebuffer table                                       │
│   - boot loop                                               │
│   (runtime profile)                                         │
└─────────────────────────────────────────────────────────────┘
                            ↓ typed primitives
┌─────────────────────────────────────────────────────────────┐
│ Ember (ember/)                                              │
│   - input_poll, present_frame, debug_log, qemu_exit         │
│   - per-arch reset, MPU/SAU/PMP, OTP, SHA-256, TRNG         │
│   (trusted profile)                                         │
└─────────────────────────────────────────────────────────────┘
                            ↓ trusted! blocks
                       Hardware
```

PhosphorOS does not touch hardware directly. Every machine-dangerous operation crosses through Ember via the typed primitives in [kernel/abi.phos](../kernel/abi.phos).

---

## 2. The Runtime Profile

PhosphorOS source compiles under the **runtime profile** (see [docs/language/RUNTIME_PROFILE.md](language/RUNTIME_PROFILE.md)). The profile permits:

- All v0 boot-profile language forms.
- The runtime effect alphabet: `draw`, `ipc`, `sched`, `time`, `mmio` (inherited from boot) plus `cap-issue`, `cap-revoke`.
- Calls into Ember-supplied primitives (input_poll, present_frame, etc.) by typed signature.

The profile **forbids**:

- Allocation (`no_alloc` rule from v0 inherited).
- `trusted!` blocks (Z-001).
- Recursion (the call graph is checked).
- Cross-profile imports of Ember internals beyond the typed boundary primitives.

---

## 3. Capacities (load-bearing)

All runtime objects are fixed-capacity. The capacities are pinned in [docs/language/runtime_profile.toml](language/runtime_profile.toml):

| Object | Capacity | Reason |
|---|---|---|
| Tasks | 64 | Single-window demo + a small allowance |
| IPC channels | 128 | Two channels per task |
| Windows | 16 | Small GUI surface |
| Framebuffers | 4 | Multi-buffer support headroom |
| Capabilities | 449 | TASK_MAX + WINDOW_MAX + CHANNEL_MAX + FRAMEBUFFER_MAX |
| Per-channel queue depth | 8 | Enough for input event bursts |
| IPC payload bytes | 256 | Bounded per-message payload |

Compile-time arrays are sized against these constants. Adding a new object class requires updating the manifest; the compiler cross-checks the literal capacities used in `.phos` source against the manifest and rejects mismatches.

---

## 4. Capability Model

### 4.1 Generation-Tagged Handles

A capability handle carries `(slot, generation, kind)`. On revocation, the slot's generation is incremented; subsequent lookups with the old generation return `KernelError::StaleHandle`.

```phos
struct TaskHandle {
    slot: u16,
    generation: u16,
    kind: CapabilityKind,
}
```

The `kind` field gates cross-kind misuse: a `TaskHandle` cannot be used to look up a window slot. The compiler's affine-capability type tracks each handle move; the runtime's generation tag catches stale references at the slot level.

### 4.2 Capability Kinds

```phos
enum CapabilityKind {
    Vacant,
    Task,
    Channel,
    Window,
    Framebuffer,
}
```

A vacant slot has `kind = Vacant` and `active = false`. Allocation finds the first vacant slot, bumps its generation, sets `active = true`, and returns the handle.

### 4.3 Allocate / Validate / Revoke

Every kernel object class follows the same pattern:

```phos
fn allocate_task(table: TaskTable, capability_slot: u16)
    -> Result[TaskEntry, KernelError];

fn validate_task(table: TaskTable, slot: u16, generation: u16)
    -> Result[u32, KernelError];

fn clear_task(table: TaskTable, slot: u16)
    -> Result[TaskEntry, KernelError];
```

`allocate` returns `TableFull` when no slot is vacant. `validate` returns `InvalidHandle` when the slot is out of range or vacant, `StaleHandle` when the generation differs. `clear` (or `revoke` for capabilities) bumps the generation and zeros the slot.

There is no panic path. Every failure is a typed `Result`.

---

## 5. IPC Model

### 5.1 Channel Tables

```phos
struct ChannelEntry {
    active: bool,
    generation: u16,
    capability_slot: u16,
    head: u16,
    tail: u16,
    queue: [MessagePayload; 8],
}
```

Each channel owns a fixed-capacity ring buffer of 8 message slots. `head` and `tail` are 16-bit counters; the depth is `(tail - head) mod 8`. The buffer is full when depth == 7 (one slot reserved to disambiguate full from empty).

### 5.2 Send / Recv

```phos
fn send_message(
    table: ChannelTable,
    slot: u16,
    generation: u16,
    payload: MessagePayload,
) -> Result[ChannelEntry, KernelError];

fn recv_message(
    table: ChannelTable,
    slot: u16,
    generation: u16,
) -> Result[RecvResult, KernelError];
```

`send_message` returns `QueueFull` when the ring is at capacity. `recv_message` returns `QueueEmpty` when the ring is empty, otherwise returns `RecvResult { payload, updated }` where `updated` is the new ChannelEntry the caller writes back.

### 5.3 Payload Size

`MessagePayload` carries a fixed `[u8; 256]` byte array plus a `len: u16`. `payload_from_bytes(src, len)` enforces `len <= 256` and returns `PayloadTooLarge` on overflow.

### 5.4 No Blocking

There is no `recv` that blocks. The boot loop polls and uses match-on-Result to handle empty.

---

## 6. Window Model

### 6.1 Windows and Damage

```phos
struct WindowEntry {
    active: bool,
    generation: u16,
    capability_slot: u16,
    owner_task_slot: u16,
    width: u32,
    height: u32,
    damage: DamageRect,
    has_focus: bool,
}
```

Each window owns a single damage rectangle that accumulates per-frame damage. `mark_damage(table, slot, gen, rect)` unions the supplied rect into the entry's accumulated damage; `drain_damage(table, slot, gen)` returns the current damage and resets it to empty.

### 6.2 Focus

A `WindowTable` carries a `focused_slot: Option[u16]`. At most one window holds focus. Input events route to the focused window's owning task channel. `dispatch_input` returns `NoFocusedWindow` when no window holds focus.

### 6.3 Compositing

`compose_render_commands(tables)` walks every active window, drains its damage rect, and emits a `RenderCommand::FillRect(window_handle, x, y, w, h, color)` for each non-empty rect. Each command carries a typed `WindowHandle` per **E19 (Kernel Capability Typing)**: the runtime kernel constructs the handle from the active window's slot + generation; Ember validates the handle before any pixel write. The bounded list is capped at 64 commands; overflow returns `OutputOverflow`. The list is terminated with a `RenderCommand::Present(focused_window_handle)` if any rects were drained; a frame with no focused window during a Present returns `NoFocusedWindow`.

### 6.4 Interrupt discipline

`compose_render_commands` walks `GlobalTables.windows` and may issue multiple `drain_damage` calls that mutate per-window state. In a single-address-space kernel without atomics, an interrupt handler running concurrently could observe a half-updated table. The compose pass therefore wraps the entire walk in `ember.sched.enter_critical_section()` / `ember.sched.exit_critical_section()` (CLI/STI on x86_64; equivalent CPU-control-register operations on the embedded arches). Every return path — including error returns for `OutputOverflow` and `NoFocusedWindow` — calls `exit_critical_section` before unwinding. The window of disabled interrupts is bounded by the 16-window cap × the per-window damage rect bookkeeping, well below any timer-tick deadline on the named arches.

---

## 7. Framebuffer Model

### 7.1 Pixel Formats

```phos
enum PixelFormat {
    Rgba8888,
    Bgra8888,
    Rgb888,
}
```

The demo path supports `Rgba8888` only (4 bytes per pixel). Bgra8888 and Rgb888 were cut in Phase 7 Tranche B (endoduction-substrate plan); format negotiation is substrate for a feature the doctrine forecloses, so the format enum is now single-variant for ABI compatibility only.

### 7.2 Validation

`validate_dimensions(width, height)` enforces 4096x4096 ceiling on both dimensions. `register_framebuffer` returns `UnsupportedFramebuffer` on out-of-range dimensions or `TableFull` when no slot is available.

---

## 8. The Boot Loop

The kernel's entrypoint is `kernel_main(boot: BootInfo) -> Result[u32, KernelError]`:

```phos
fn kernel_main(boot: BootInfo) -> Result[u32, KernelError] {
    match validate_boot_info(boot) {
        Err(e) => Err(e),
        Ok(_)  => {
            let mut tables: GlobalTables = empty_global_tables(boot.manifest);
            // Stage 1: register framebuffer.
            // Stage 2: allocate demo task.
            // Stage 3: allocate demo window + grant focus.
            // Stage 4: enter the bounded demo tick loop.
            #[bound = 65536]
            for tick in 0..65536 {
                // Per-tick:
                //   1. ember.lib.input_poll() -> Option[InputEvent]
                //   2. dispatch_input(tables, evt) on Some
                //   3. compose_render_commands(tables)
                //   4. ember.lib.present_frame(commands)
                //   5. check demo_complete signal
            }
            Ok(0)
        },
    }
}
```

The loop is bounded. Single-threaded. Single-task-space. Cooperative — there is no preemption.

### 8.1 Frame Outcomes

```phos
enum FrameOutcome {
    Continue,
    DemoComplete,
    Faulted(KernelError),
}
```

Each tick returns one outcome. `Continue` means another tick may run. `DemoComplete` is the success exit. `Faulted(e)` is a typed error that exits with the error.

---

## 9. Error Codes

```phos
enum KernelError {
    TableFull,                  // status = 1
    InvalidHandle,              // status = 2
    StaleHandle,                // status = 3
    QueueFull,                  // status = 4
    QueueEmpty,                 // status = 5
    Revoked,                    // status = 6
    UnsupportedFramebuffer,     // status = 7
    NoFocusedWindow,            // status = 8
    OutputOverflow,             // status = 9
    InvalidInputEvent,          // status = 10
    PayloadTooLarge,            // status = 11
    NotInitialized,             // status = 12
}
```

The status integers are stable wire-protocol values. Any change is a wire-protocol change.

---

## 10. The ABI Boundary

[kernel/abi.phos](../kernel/abi.phos) declares every typed shape that crosses between kernel and Ember. There is no FFI. Every call is statically resolved at link time.

```phos
struct BootInfo {
    framebuffer_width: u32,
    framebuffer_height: u32,
    framebuffer_stride: u32,
    framebuffer_format: PixelFormat,
    serial_present: bool,
}

enum InputEvent {
    Key(KeyEvent),
    MouseClick(MouseClickEvent),
    MouseMove(MouseMoveEvent),
}

enum RenderCommand {
    FillRect(u32, u32, u32, u32, u32),   // x, y, w, h, rgba
    WritePixel(u32, u32, u32),           // x, y, rgba
    Present,                              // commit current frame
}
```

The shapes are pinned. Adding a new variant or field requires:

1. Updating `abi.phos`.
2. Updating any Ember-side handler that consumes the type.
3. Re-running golden fixtures to confirm byte stability of the boot-asm output.

---

## 11. Per-Module Documentation

Each kernel module has its own design document under [kernel/docs/](../kernel/docs/):

- [task_model.md](../kernel/docs/task_model.md)
- [ipc.md](../kernel/docs/ipc.md)
- [capabilities.md](../kernel/docs/capabilities.md)
- [window_model.md](../kernel/docs/window_model.md)
- [framebuffer.md](../kernel/docs/framebuffer.md)
- [compositor.md](../kernel/docs/compositor.md)
- [input.md](../kernel/docs/input.md)

Each document specifies the module's invariants, capacities, and interaction patterns.

---

## 12. The Demo Today

The first booted vertical slice ([apps/demo/](../apps/demo/)) is a single-button-press redraw exercise. Six `.phos` files, ~150 LOC. The demo's logic — state machine, render commands, route outcomes — lives at the boot-profile layer; PhosphorOS proper provides the table types and the boot loop scaffolding the demo runs against.

The demo emits these QEMU markers:

```
phosphoric: entering generated boot-asm demo
phosphoric: generated boot-asm demo runtime active
phosphoric: event routed
phosphoric: redraw complete
phosphoric: demo complete
```

Each marker corresponds to a completed stage of the boot loop. The markers are part of the public verification gate — `make verify` requires all five.

---

## 13. What PhosphorOS Is Not

- Not a multi-tasking OS. Cooperative single-task-space.
- Not a networking stack. No sockets, no protocol stacks, no driver framework.
- Not a filesystem. Pure RAM-disk operation; reset wipes everything except OTP-burned trust roots.
- Not a desktop compositor. Single-window today; the multi-window pathway is reserved for a future scope but not v0.
- Not preemptive. The boot loop runs cooperatively; tasks yield by returning from per-tick handlers.
- Not a Unix-shaped kernel. No process/thread distinction, no signals, no fork.

The OS layer's narrowness mirrors the language's: no aspirational generality, no features that expand the review surface without serving the project's narrow target.

---

## See Also

- [docs/PHOSPHORIC.md](PHOSPHORIC.md) — the language reference
- [docs/EMBER.md](EMBER.md) — the trusted nucleus
- [docs/COMPILER.md](COMPILER.md) — the compiler
- [docs/language/RUNTIME_PROFILE.md](language/RUNTIME_PROFILE.md) — runtime profile manifest
- [kernel/](../kernel/) — kernel source
- [apps/demo/](../apps/demo/) — booted demo
