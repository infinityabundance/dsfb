# Boot ABI V1

This document defines the first reviewed native-code boundary for Phosphoric-generated code on the current `x86_64 + UEFI + QEMU` path.

It is intentionally narrow.

## Purpose

`BOOT_ABI_V1` exists to freeze one low-entropy machine boundary that both of these tracks can share:

- direct native backend work
- future Phosphoric-written compiler-core work

It is not a general FFI surface.

## Exported Symbols

Current exported golden-boot symbols:

- `efi_main`
- `phosphoric_demo_init`
- `phosphoric_demo_step`
- `phosphoric_demo_render`

These remain the only golden boot exports until parity work is reviewed and a broader ABI is explicitly accepted. The legacy C oracle still emits `phosphoric_demo_button_next` and `phosphoric_demo_button_rect`, but those symbols are not linked into the golden boot path.

## Data Shapes

Current narrow exported shapes:

```c
typedef struct {
    _Bool next_pressed;
    _Bool redraw_requested;
} DemoButtonTransition;

typedef struct {
    unsigned int x;
    unsigned int y;
    unsigned int w;
    unsigned int h;
} DemoRect;
```

Current fixed-capacity machine-boundary runtime shapes remain:

- `BootFramebuffer`
- `BootInputEvent`
- `RouteOutcome`
- `RenderOutput`
- `RenderCommand`

Those shapes are mirrored by the generated `boot-asm-v1` artifact and by the runtime kernel's ABI module at [`kernel/abi.phos`](../kernel/abi.phos).

## Calling Convention

Current target ABI:

- target: `x86_64-pc-windows-msvc`
- object format: COFF
- image format: PE32+ EFI application
- image writer: `tools/phosphoric/write_boot_efi_from_ir.sh`
- image validator: `tools/phosphoric/verify_pe_efi_image.sh`

Current reviewed expectations:

- `efi_main` follows the UEFI application entry ABI and is emitted by `boot-asm-v1`
- `phosphoric_demo_init` follows the platform ABI for `(out: *mut DemoState, framebuffer: *const BootFramebuffer) -> *mut DemoState`
- `phosphoric_demo_step` follows the platform ABI for `(out: *mut DemoStepResult, state: *const DemoState, event: *const BootInputEvent) -> *mut DemoStepResult`
- `phosphoric_demo_render` follows the platform ABI for `(out: *mut RenderOutput, state: *const DemoState) -> *mut RenderOutput`
- generated code assumes no libc, no heap, and no floating-point runtime dependency

## Scope Limits

`BOOT_ABI_V1` does not permit:

- general application-defined external calls from Phosphoric source
- arbitrary struct export
- arbitrary enum export
- imports or module linking in source
- dynamic discovery or plugin-style loading

## Shared Contract With `boot_ir_v1`

`boot_ir_v1` is the backend-internal representation that feeds this ABI today.

That means:

- frontend-reviewed source is lowered into `boot_ir_v1`
- `boot-asm-v1` emits native assembly evidence from `boot_ir_v1`
- the direct PE/COFF writer emits the current reviewed EFI machine image shape from the same boot-profile contract
- the legacy C oracle also lowers from `boot_ir_v1`

The contract is intentionally fixed-capacity and profile-driven so future Phosphoric-written compiler-core work can target the same reviewed shapes.
