# x86_64 QEMU Bring-Up Plan

This document defines the bring-up plan for the first `Ember` boot target.

The target is intentionally narrow:

- architecture: `x86_64`
- firmware path: `UEFI`
- emulator: `QEMU`
- graphics target: linear framebuffer exposed through the firmware handoff
- early observability target: serial logging

This plan exists so boot scaffolding can be reviewed against an explicit target instead of growing opportunistically.

## Boot Assumptions

The first bring-up assumes:

- the system boots under `QEMU` with `UEFI` firmware available
- firmware provides framebuffer metadata through the boot handoff
- the initial prototype uses one CPU core
- the first runnable image is a statically linked kernel image built by the project toolchain
- no writable filesystem is required for the first milestone

Out-of-scope for this bring-up:

- SMP
- networking
- process isolation
- GPU acceleration
- dynamic driver loading

## Early Output Strategy

Two output paths are required from the start:

### Serial Logging

Serial logging is the first observability path.

Requirements:

- initialize a stable serial output path as early as possible after firmware handoff
- use serial logs for boot milestones, trap diagnostics, and early failure reporting
- keep the serial path simple and deterministic

Initial success condition:

- print a boot banner and milestone markers over the serial path in QEMU

### Framebuffer Output

Framebuffer output is the first graphics path.

Requirements:

- accept framebuffer metadata from firmware through the trusted handoff boundary
- validate width, height, stride, pixel format, and base address before exposing the framebuffer upward
- support at least clearing the screen to a solid color

Initial success condition:

- clear the linear framebuffer to a known background color

## Interrupt Bring-Up Order

Interrupt and timer bring-up must happen in a controlled order:

1. establish basic boot execution and serial output
2. install descriptor tables and trap entry points
3. prove trap entry works with known-safe diagnostics
4. configure timer interrupt machinery
5. enable interrupts only after the system can log, receive, and contain faults

Rules:

- do not enable interrupts before trap handlers and diagnostic paths are ready
- do not depend on the scheduler before timer delivery is known to work
- do not layer input-device interrupt work on top of an unverified timer path

## Bring-Up Stages

### Stage A: Boot Banner

Deliverable:

- reach the trusted entrypoint after firmware handoff
- print serial banner

Success signal:

- QEMU serial output shows entry into `Ember`

### Stage B: Trusted Handoff Validation

Deliverable:

- parse and validate boot-time metadata needed by the runtime
- validate framebuffer metadata

Success signal:

- serial output confirms accepted framebuffer dimensions and format

### Stage C: Framebuffer Clear

Deliverable:

- map or accept the framebuffer boundary
- clear the screen to a known color

Success signal:

- QEMU display shows a stable full-screen background color

### Stage D: Trap Table Install

Deliverable:

- build descriptor tables
- install trap entry points
- verify a basic exception path can report safely

Success signal:

- serial output confirms trap initialization without recursive failure

### Stage E: Timer Bring-Up

Deliverable:

- configure the first timer path needed for later scheduler work

Success signal:

- serial output reports timer ticks or trusted timer events

### Stage F: Input Foundation

Deliverable:

- prepare the interrupt path needed for PS/2 keyboard and PS/2 mouse work in later phases

Success signal:

- serial output confirms the input boundary is ready for later driver work

## Framebuffer Target

The first graphics target is:

- linear framebuffer only
- software rendering only
- no GPU acceleration
- rectangular full-screen clear as the initial primitive

Required metadata:

- width
- height
- stride or pitch
- bytes per pixel or pixel format description
- base address

Required review rule:

- no framebuffer operation is allowed before metadata validation succeeds

## Serial Target

The serial path is the canonical early-debug channel.

Required uses:

- boot milestone markers
- panic or fatal-stop breadcrumbs in trusted code
- trap diagnostics during bring-up

Forbidden uses:

- treating serial output as proof that the rest of the system is correct
- relying on serial output in place of validating framebuffer metadata or interrupt order

## Acceptance Gate Before Boot Scaffold

Any initial boot scaffold must reflect this plan by making the following explicit:

- where firmware handoff enters `Ember`
- where serial initialization lives
- where framebuffer handoff validation lives
- where descriptor-table or trap setup lives
- where timer bring-up will attach later

## What Success Looks Like

The first credible bring-up milestone is:

- QEMU boots the image through `UEFI`
- serial output prints a stable boot banner
- framebuffer metadata is validated and logged
- the framebuffer is cleared to a known color
- descriptor tables and trap entry are installed without immediate failure
- timer bring-up has a defined next attachment point

This is enough to justify moving on to minimal boot scaffolding. It is not yet a working OS.
