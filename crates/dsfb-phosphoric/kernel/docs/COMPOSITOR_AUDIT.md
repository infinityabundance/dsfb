# Compositor Trust Audit

`compose_render_commands` in [kernel/kernel.phos](../kernel.phos) is a trusted-zone boundary. It walks `GlobalTables.windows`, constructs `RenderCommand` values, and (via the Present command) hands the framebuffer to Ember. The framebuffer is Ember-owned memory; any pixel write originated by the compositor is ultimately a memory write that crosses into ultra-trusted territory.

This file is the per-line audit that justifies the compositor's right to touch what it touches. It mirrors the discipline of [ember/docs/EMBER_TRUST_AUDIT.md](../../ember/docs/EMBER_TRUST_AUDIT.md): each construct that affects the frame must be named, justified, and shown to be bounded.

## Trust-zone classification

- **Untrusted:** none of the compositor.
- **Trusted:** the entire `compose_render_commands` body. Reasons: it constructs WindowHandle values, allocates RenderCommand entries into a fixed-capacity list, and invokes `ember.sched.{enter,exit}_critical_section()`. None of these are ultra-trusted.
- **Ultra-trusted:** Ember's eventual `present_frame(commands)` consumer. Out of scope for this file.

## Per-construct audit

| Construct in `compose_render_commands` | Justification | Bound |
|---|---|---|
| `ember.sched.enter_critical_section()` at entry | Walk reads-and-writes per-window state shared with interrupt handlers | One call per compositor invocation |
| `ember.sched.exit_critical_section()` on every exit path | Symmetric pair — every entry must be matched | Four exit sites covered (Ok, OutputOverflow, NoFocusedWindow, OutputOverflow-on-Present) |
| `for slot in 0..16` over `tables.windows.entries[slot]` | Bounded by `WINDOW_MAX = 16` from [config/runtime_profile.toml](../../config/runtime_profile.toml) | 16 iterations max |
| `RenderCommand::FillRect(win_handle, 0, 0, w, h, color)` per active window | Full-bounds redraw — recessed in Phase 7 Tranche B from the prior damage-rect path | One FillRect per active window, ≤ 16 total |
| `RenderCommand::Present(present_handle)` trailing | Required to drive Ember's framebuffer flip | One Present per non-empty list |
| `tables.residuals = pcc.kernel.residual.record(...)` for R-kinds emitted from this path | Residual emission at the compositor boundary is a future R-kind extension; currently no R-kind emits from compose_render | Zero today; if added, bounded by 16 + 1 (one per window + one per Present) |

## Critical-section discipline

`compose_render_commands` enters a critical section before the walk and exits before every return. This is the same CLI/STI discipline that `dispatch_input` uses around `send_message`, and that `kernel_main` uses around `input_poll` (per Phase 7 Tranche B + the atomic-input-polling fix). The critical section bounds:

- **Window of disabled interrupts:** ~hundreds of cycles (16 windows × per-window read/write).
- **Worst-case mid-critical timer-tick miss:** one tick at the demo's tick rate. Acceptable.

Adding anything to this function that takes more than O(WINDOW_MAX) work breaks the bound. Any future extension that needs more time must split the work across critical-section boundaries, not lengthen this one.

## Forensic claim

`compose_render_commands` is part of the substrate's deterministic-replay claim: same `GlobalTables` input → same `RenderCommandList` output, byte-for-byte. The recessed full-bounds-redraw path (no damage tracking) is what makes this true; damage rects would have introduced per-frame state that's hard to reproduce off-device.

## What the compositor must NEVER do

- Touch framebuffer memory directly. Only Ember does that, via `present_frame`.
- Allocate. The RenderCommand list is fixed-capacity (64 entries).
- Emit RenderCommands beyond the list bound. Overflow returns `OutputOverflow`.
- Call into Ember outside `enter/exit_critical_section`. The compositor's Ember interactions are exactly: enter, walk, exit. Every other Ember call is in `kernel_main`'s outer loop, not here.
- Skip the trailing Present when commands were emitted. Absent Present = NoFocusedWindow returned to caller.

## Audit ownership

The compositor's audit is owned by whoever owns [kernel/kernel.phos](../kernel.phos). Per the trust-zone partition (docs/TASK_SEAL_V0.md §0.4), this is the *trusted* zone — the compositor is one zone below ultra-trusted Ember. Any change to `compose_render_commands` updates the table above; otherwise the audit drifts and the trust claim weakens.
