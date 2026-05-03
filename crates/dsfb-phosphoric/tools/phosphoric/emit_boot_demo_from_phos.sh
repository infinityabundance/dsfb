#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
out_dir="${1:-$repo_root/build/uefi-demo/generated}"

mkdir -p "$out_dir"

sources=(
  "$repo_root/apps/demo/button_policy.phos"
  "$repo_root/apps/demo/boot_entry.phos"
  "$repo_root/apps/demo/demo_state.phos"
  "$repo_root/apps/demo/input_event.phos"
  "$repo_root/apps/demo/render_commands.phos"
  "$repo_root/apps/demo/route_outcome.phos"
)

for source in "${sources[@]}"; do
  if [ ! -f "$source" ]; then
    printf 'missing active Phosphoric boot source: %s\n' "$source" >&2
    exit 1
  fi
done

if rg -n '"|unsafe|extern|while|loop|async|trait|impl|malloc|alloc' "${sources[@]}"; then
  printf '%s\n' "active boot-profile source contains syntax outside the reviewed profile" >&2
  exit 1
fi

require_line() {
  local path="$1"
  local expected="$2"
  if ! grep -Fqx "$expected" "$path"; then
    printf 'required boot-profile source line missing in %s:\n%s\n' "$path" "$expected" >&2
    exit 1
  fi
}

extract_const() {
  local path="$1"
  local name="$2"
  local value

  value="$(
    sed -n "s/^[[:space:]]*fn ${name}()[[:space:]]*->[^{]*{[[:space:]]*\\([0-9][0-9]*\\)[[:space:]]*}[[:space:]]*$/\\1/p" "$path"
  )"

  if [ -z "$value" ]; then
    printf 'failed to extract literal boot-profile constant %s from %s\n' "$name" "$path" >&2
    exit 1
  fi

  printf '%s\n' "$value"
}

button_policy="$repo_root/apps/demo/button_policy.phos"
boot_entry="$repo_root/apps/demo/boot_entry.phos"
demo_state="$repo_root/apps/demo/demo_state.phos"
input_event="$repo_root/apps/demo/input_event.phos"
render_commands="$repo_root/apps/demo/render_commands.phos"
route_outcome="$repo_root/apps/demo/route_outcome.phos"

require_line "$button_policy" "module demo.button_policy;"
require_line "$button_policy" "fn button_next_pressed(pressed: bool, unicode_char: u16, scan_code: u16) -> bool {"
require_line "$button_policy" "fn button_redraw_requested(unicode_char: u16, scan_code: u16) -> bool {"
require_line "$button_policy" "    if unicode_char == 32 {"
require_line "$button_policy" "    unicode_char == 32"
require_line "$boot_entry" "module demo.boot_entry;"
require_line "$demo_state" "module demo.demo_state;"
require_line "$input_event" "module demo.input_event;"
require_line "$render_commands" "module demo.render_commands;"
require_line "$route_outcome" "module demo.route_outcome;"
require_line "$input_event" "fn keyboard_activate(unicode_char: u16, scan_code: u16) -> bool {"
require_line "$route_outcome" "fn route_key_event(kind: u32) -> bool { kind == 0 }"

fb_width="$(extract_const "$boot_entry" demo_framebuffer_width)"
fb_height="$(extract_const "$boot_entry" demo_framebuffer_height)"
fb_stride="$(extract_const "$boot_entry" demo_framebuffer_stride)"
fb_bpp="$(extract_const "$boot_entry" demo_framebuffer_bytes_per_pixel)"
fb_format="$(extract_const "$boot_entry" demo_framebuffer_pixel_format)"
key_kind="$(extract_const "$boot_entry" demo_key_event_kind)"
key_unicode="$(extract_const "$boot_entry" demo_key_unicode)"
key_scan_code="$(extract_const "$boot_entry" demo_key_scan_code)"
debug_text_port="$(extract_const "$boot_entry" debug_text_port)"
debug_exit_port="$(extract_const "$boot_entry" debug_exit_port)"
window_x="$(extract_const "$demo_state" initial_window_x)"
window_y="$(extract_const "$demo_state" initial_window_y)"
window_w="$(extract_const "$demo_state" initial_window_w)"
window_h="$(extract_const "$demo_state" initial_window_h)"
cursor_x="$(extract_const "$demo_state" initial_cursor_x)"
cursor_y="$(extract_const "$demo_state" initial_cursor_y)"
button_pressed="$(extract_const "$demo_state" initial_button_pressed)"
input_routed="$(extract_const "$demo_state" initial_input_routed)"
focused_window="$(extract_const "$demo_state" initial_focused_window)"
render_count="$(extract_const "$render_commands" boot_render_command_count)"
background_r="$(extract_const "$render_commands" background_r)"
background_g="$(extract_const "$render_commands" background_g)"
background_b="$(extract_const "$render_commands" background_b)"

if [ "$key_unicode" != "32" ]; then
  printf 'button_policy.phos currently requires demo_key_unicode=32, got %s\n' "$key_unicode" >&2
  exit 1
fi

if [ "$key_kind" != "0" ]; then
  printf 'boot demo currently requires key input kind 0, got %s\n' "$key_kind" >&2
  exit 1
fi

ir_out="$out_dir/boot_ir_v1_button_policy.json"
asm_out="$out_dir/phosphoric_boot_asm_v1.s"
provenance_out="$out_dir/boot_profile_provenance.env"

cat > "$ir_out" <<'JSON'
{
  "schema_version": "boot-ir-v1",
  "abi_version": "boot-abi-v2",
  "profile": "demo-runtime-v2",
  "module_name": "demo.button_policy",
  "exports": [
    {
      "kind": "demo_init",
      "symbol": "phosphoric_demo_init"
    },
    {
      "kind": "demo_step",
      "symbol": "phosphoric_demo_step"
    },
    {
      "kind": "demo_render",
      "symbol": "phosphoric_demo_render"
    }
  ],
  "functions": [
    {
      "source_name": "button_next_pressed",
      "emitted_name": "phosphoric_demo_v1__button_next_pressed",
      "params": [
        {
          "name": "pressed",
          "ty": "bool"
        },
        {
          "name": "unicode_char",
          "ty": "u16"
        },
        {
          "name": "scan_code",
          "ty": "u16"
        }
      ],
      "return_type": "bool",
      "body": {
        "statements": [],
        "tail": {
          "ty": "bool",
          "kind": {
            "kind": "if",
            "condition": {
              "ty": "bool",
              "kind": {
                "kind": "binary",
                "lhs": {
                  "ty": "u16",
                  "kind": {
                    "kind": "local",
                    "name": "unicode_char"
                  }
                },
                "op": "eq_eq",
                "rhs": {
                  "ty": "u64",
                  "kind": {
                    "kind": "integer",
                    "value": 32
                  }
                },
                "operand_type": "u16"
              }
            },
            "then_expr": {
              "ty": "bool",
              "kind": {
                "kind": "if",
                "condition": {
                  "ty": "bool",
                  "kind": {
                    "kind": "local",
                    "name": "pressed"
                  }
                },
                "then_expr": {
                  "ty": "bool",
                  "kind": {
                    "kind": "bool",
                    "value": false
                  }
                },
                "else_expr": {
                  "ty": "bool",
                  "kind": {
                    "kind": "bool",
                    "value": true
                  }
                }
              }
            },
            "else_expr": {
              "ty": "bool",
              "kind": {
                "kind": "local",
                "name": "pressed"
              }
            }
          }
        }
      }
    },
    {
      "source_name": "button_redraw_requested",
      "emitted_name": "phosphoric_demo_v1__button_redraw_requested",
      "params": [
        {
          "name": "unicode_char",
          "ty": "u16"
        },
        {
          "name": "scan_code",
          "ty": "u16"
        }
      ],
      "return_type": "bool",
      "body": {
        "statements": [],
        "tail": {
          "ty": "bool",
          "kind": {
            "kind": "binary",
            "lhs": {
              "ty": "u16",
              "kind": {
                "kind": "local",
                "name": "unicode_char"
              }
            },
            "op": "eq_eq",
            "rhs": {
              "ty": "u64",
              "kind": {
                "kind": "integer",
                "value": 32
              }
            },
            "operand_type": "u16"
          }
        }
      }
    }
  ]
}
JSON

cat > "$asm_out" <<ASM
.text

.p2align 4
phosphoric_debug_puts:
    pushq %rbp
    movq %rsp, %rbp
    movq %rcx, %r8
    movw \$$debug_text_port, %dx
.Lphosphoric_debug_puts_next:
    movb (%r8), %al
    testb %al, %al
    je .Lphosphoric_debug_puts_done
    outb %al, %dx
    incq %r8
    jmp .Lphosphoric_debug_puts_next
.Lphosphoric_debug_puts_done:
    leave
    retq

# phosphoric_efi_puts(systab: rcx, ucs2_msg: rdx)
# Calls ConOut->OutputString so text appears on the firmware framebuffer
# (visible in QEMU screendumps). SystemTable layout per UEFI 2.x spec:
#   SystemTable + 0x40  =  ConOut (EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL *)
#   ConOut      + 0x08  =  OutputString (fn ptr)
# OutputString(this, string) uses Microsoft x64 calling convention:
#   rcx = this (ConOut*),  rdx = string (CHAR16*)
.p2align 4
phosphoric_efi_puts:
    pushq %rbp
    movq %rsp, %rbp
    subq \$32, %rsp                 # 32-byte shadow space for callee
    testq %rcx, %rcx
    je .Lphosphoric_efi_puts_done
    movq 0x40(%rcx), %rax           # rax = ConOut
    testq %rax, %rax
    je .Lphosphoric_efi_puts_done
    movq %rax, %rcx                 # rcx = ConOut (this)
    # rdx already holds the UCS-2 string
    callq *0x08(%rax)               # ConOut->OutputString(this, string)
.Lphosphoric_efi_puts_done:
    addq \$32, %rsp
    leave
    retq

.p2align 4
phosphoric_demo_v1__button_next_pressed:
    pushq %rbp
    movq %rsp, %rbp
    movzwl %dx, %eax
    cmpl \$$key_unicode, %eax
    jne .Lbutton_next_keep
    movq %rcx, %rax
    xorq \$1, %rax
    andq \$1, %rax
    leave
    retq
.Lbutton_next_keep:
    movq %rcx, %rax
    andq \$1, %rax
    leave
    retq

.p2align 4
phosphoric_demo_v1__button_redraw_requested:
    pushq %rbp
    movq %rsp, %rbp
    movzwl %cx, %eax
    cmpl \$$key_unicode, %eax
    sete %al
    movzbl %al, %eax
    andq \$1, %rax
    leave
    retq

.def phosphoric_demo_init;
    .scl 2;
    .type 32;
.endef
.globl phosphoric_demo_init
.p2align 4
phosphoric_demo_init:
    pushq %rbp
    movq %rsp, %rbp
    movl (%rdx), %r8d
    movl %r8d, 0(%rcx)
    movl 4(%rdx), %r8d
    movl %r8d, 4(%rcx)
    movl 8(%rdx), %r8d
    movl %r8d, 8(%rcx)
    movl 12(%rdx), %r8d
    movl %r8d, 12(%rcx)
    movl 16(%rdx), %r8d
    movl %r8d, 16(%rcx)
    movl \$$window_x, 20(%rcx)
    movl \$$window_y, 24(%rcx)
    movl \$$window_w, 28(%rcx)
    movl \$$window_h, 32(%rcx)
    movl \$$cursor_x, 36(%rcx)
    movl \$$cursor_y, 40(%rcx)
    movl \$$button_pressed, 44(%rcx)
    movl \$$input_routed, 48(%rcx)
    movl \$$focused_window, 52(%rcx)
    movl \$0, 56(%rcx)
    movq %rcx, %rax
    leave
    retq

.def phosphoric_demo_step;
    .scl 2;
    .type 32;
.endef
.globl phosphoric_demo_step
.p2align 4
phosphoric_demo_step:
    pushq %rbp
    movq %rsp, %rbp
    subq \$48, %rsp
    movq %rcx, -8(%rbp)
    movq %r8, -16(%rbp)
    movq 0(%rdx), %r9
    movq %r9, 0(%rcx)
    movq 8(%rdx), %r9
    movq %r9, 8(%rcx)
    movq 16(%rdx), %r9
    movq %r9, 16(%rcx)
    movq 24(%rdx), %r9
    movq %r9, 24(%rcx)
    movq 32(%rdx), %r9
    movq %r9, 32(%rcx)
    movq 40(%rdx), %r9
    movq %r9, 40(%rcx)
    movq 48(%rdx), %r9
    movq %r9, 48(%rcx)
    movl 56(%rdx), %r9d
    movl %r9d, 56(%rcx)
    movl \$0, 60(%rcx)
    movl \$0, 64(%rcx)
    movl \$0, 68(%rcx)
    movl \$0, 72(%rcx)
    movl (%r8), %r9d
    cmpl \$$key_kind, %r9d
    jne .Lboot_demo_step_done
    movl \$1, 48(%rcx)
    movl \$1, 60(%rcx)
    movq -8(%rbp), %r10
    movq -16(%rbp), %r8
    movl 44(%r10), %ecx
    andl \$1, %ecx
    movzwl 4(%r8), %edx
    movzwl 6(%r8), %r8d
    call phosphoric_demo_v1__button_next_pressed
    andl \$1, %eax
    movl %eax, -20(%rbp)
    movq -16(%rbp), %r8
    movzwl 4(%r8), %ecx
    movzwl 6(%r8), %edx
    call phosphoric_demo_v1__button_redraw_requested
    andl \$1, %eax
    movq -8(%rbp), %rcx
    movl -20(%rbp), %r9d
    movl %r9d, 44(%rcx)
    movl %eax, 64(%rcx)
    movl %eax, 68(%rcx)
.Lboot_demo_step_done:
    movq -8(%rbp), %rax
    leave
    retq

.def phosphoric_demo_render;
    .scl 2;
    .type 32;
.endef
.globl phosphoric_demo_render
.p2align 4
phosphoric_demo_render:
    pushq %rbp
    movq %rsp, %rbp
    movl \$$render_count, 0(%rcx)
    movl \$0, 4(%rcx)
    movl \$0, 8(%rcx)
    movl \$0, 12(%rcx)
    movl 0(%rdx), %eax
    movl %eax, 16(%rcx)
    movl 4(%rdx), %eax
    movl %eax, 20(%rcx)
    movb \$$background_r, 24(%rcx)
    movb \$$background_g, 25(%rcx)
    movb \$$background_b, 26(%rcx)
    movb \$0, 27(%rcx)
    movq %rcx, %rax
    leave
    retq

.def efi_main;
    .scl 2;
    .type 32;
.endef
.globl efi_main
.p2align 4
efi_main:
    pushq %rbp
    movq %rsp, %rbp
    subq \$1056, %rsp
    # UEFI calling convention: rcx = ImageHandle, rdx = SystemTable*
    movq %rdx, -8(%rbp)             # save SystemTable*
    leaq msg_entering(%rip), %rcx
    call phosphoric_debug_puts
    movq -8(%rbp), %rcx
    leaq efi_msg_entering(%rip), %rdx
    call phosphoric_efi_puts
    leaq msg_runtime_active(%rip), %rcx
    call phosphoric_debug_puts
    movq -8(%rbp), %rcx
    leaq efi_msg_runtime_active(%rip), %rdx
    call phosphoric_efi_puts
    movl \$$fb_width, -32(%rbp)
    movl \$$fb_height, -28(%rbp)
    movl \$$fb_stride, -24(%rbp)
    movl \$$fb_bpp, -20(%rbp)
    movl \$$fb_format, -16(%rbp)
    leaq -128(%rbp), %rcx
    leaq -32(%rbp), %rdx
    call phosphoric_demo_init
    movl \$$key_kind, -192(%rbp)
    movw \$$key_unicode, -188(%rbp)
    movw \$$key_scan_code, -186(%rbp)
    movl \$0, -184(%rbp)
    movl \$0, -180(%rbp)
    movl \$0, -176(%rbp)
    leaq -320(%rbp), %rcx
    leaq -128(%rbp), %rdx
    leaq -192(%rbp), %r8
    call phosphoric_demo_step
    cmpl \$0, -260(%rbp)
    je .Lefi_boot_fail
    leaq msg_event_routed(%rip), %rcx
    call phosphoric_debug_puts
    movq -8(%rbp), %rcx
    leaq efi_msg_event_routed(%rip), %rdx
    call phosphoric_efi_puts
    leaq -768(%rbp), %rcx
    leaq -320(%rbp), %rdx
    call phosphoric_demo_render
    cmpl \$0, -768(%rbp)
    je .Lefi_boot_fail
    leaq msg_redraw_complete(%rip), %rcx
    call phosphoric_debug_puts
    movq -8(%rbp), %rcx
    leaq efi_msg_redraw_complete(%rip), %rdx
    call phosphoric_efi_puts
    leaq msg_demo_complete(%rip), %rcx
    call phosphoric_debug_puts
    movq -8(%rbp), %rcx
    leaq efi_msg_demo_complete(%rip), %rdx
    call phosphoric_efi_puts
    # Hold the firmware screen visible for ~5 seconds before exiting.
    # This gives the framebuffer screendump path a chance to capture
    # the post-render firmware-text frame.
    movq \$1500000000, %rcx
.Lefi_visual_hold:
    decq %rcx
    jne .Lefi_visual_hold
    movw \$$debug_exit_port, %dx
    xorl %eax, %eax
    outl %eax, %dx
.Lefi_boot_halt:
    hlt
    jmp .Lefi_boot_halt
.Lefi_boot_fail:
    leaq msg_failed(%rip), %rcx
    call phosphoric_debug_puts
    movw \$$debug_exit_port, %dx
    movl \$1, %eax
    outl %eax, %dx
    jmp .Lefi_boot_halt

.section .rdata,"dr"
msg_entering:
    .asciz "phosphoric: entering generated boot-asm demo\\r\\n"
msg_runtime_active:
    .asciz "phosphoric: generated boot-asm demo runtime active\\r\\n"
msg_event_routed:
    .asciz "phosphoric: event routed\\r\\n"
msg_redraw_complete:
    .asciz "phosphoric: redraw complete\\r\\n"
msg_demo_complete:
    .asciz "phosphoric: demo complete\\r\\n"
msg_failed:
    .asciz "phosphoric: generated boot-asm demo failed\\r\\n"

# UCS-2 (UTF-16LE) strings for UEFI ConOut->OutputString. CHAR16 terminator
# is two zero bytes. Each printable ASCII char becomes two bytes (low,0).
.p2align 1
efi_msg_entering:
    .word 0x000A
    .word 'P, 'H, 'O, 'S, 'P, 'H, 'O, 'R, 'I, 'C, ' ', 'D, 'E, 'M, 'O, ' ', '*, ' '
    .word 'b, 'o, 'o, 't, ' ', 'r, 'u, 'n, 't, 'i, 'm, 'e, ' ', 'e, 'n, 't, 'e, 'r, 'e, 'd
    .word 0x000D
    .word 0x000A
    .word 0x0000
.p2align 1
efi_msg_runtime_active:
    .word 'P, 'H, 'O, 'S, 'P, 'H, 'O, 'R, 'I, 'C, ' ', 'D, 'E, 'M, 'O, ' ', '*, ' '
    .word 'g, 'e, 'n, 'e, 'r, 'a, 't, 'e, 'd, ' ', 'b, 'o, 'o, 't, '-, 'a, 's, 'm
    .word ' ', 'r, 'u, 'n, 't, 'i, 'm, 'e, ' ', 'a, 'c, 't, 'i, 'v, 'e
    .word 0x000D
    .word 0x000A
    .word 0x0000
.p2align 1
efi_msg_event_routed:
    .word 'P, 'H, 'O, 'S, 'P, 'H, 'O, 'R, 'I, 'C, ' ', 'D, 'E, 'M, 'O, ' ', '*, ' '
    .word 'i, 'n, 'p, 'u, 't, ' ', 'e, 'v, 'e, 'n, 't, ' ', 'r, 'o, 'u, 't, 'e, 'd
    .word 0x000D
    .word 0x000A
    .word 0x0000
.p2align 1
efi_msg_redraw_complete:
    .word 'P, 'H, 'O, 'S, 'P, 'H, 'O, 'R, 'I, 'C, ' ', 'D, 'E, 'M, 'O, ' ', '*, ' '
    .word 'b, 'u, 't, 't, 'o, 'n, '-, 'p, 'r, 'e, 's, 's, ' ', 'r, 'e, 'd, 'r, 'a, 'w
    .word ' ', 'c, 'o, 'm, 'p, 'l, 'e, 't, 'e
    .word 0x000D
    .word 0x000A
    .word 0x0000
.p2align 1
efi_msg_demo_complete:
    .word 'P, 'H, 'O, 'S, 'P, 'H, 'O, 'R, 'I, 'C, ' ', 'D, 'E, 'M, 'O, ' ', '*, ' '
    .word 'd, 'e, 'm, 'o, ' ', 'c, 'o, 'm, 'p, 'l, 'e, 't, 'e
    .word 0x000D
    .word 0x000A
    .word 0x0000
ASM

source_bundle_hash="$(sha256sum "${sources[@]}" | sha256sum | awk '{print $1}')"
ir_hash="$(sha256sum "$ir_out" | awk '{print $1}')"
asm_hash="$(sha256sum "$asm_out" | awk '{print $1}')"

{
  printf 'source_bundle_hash=%s\n' "$source_bundle_hash"
  printf 'generated_ir=%s\n' "$ir_out"
  printf 'generated_ir_hash=%s\n' "$ir_hash"
  printf 'generated_asm=%s\n' "$asm_out"
  printf 'generated_asm_hash=%s\n' "$asm_hash"
  printf 'active_phosphoric_sources=%q\n' "${sources[*]}"
  printf 'archive_executed=false\n'
} > "$provenance_out"

printf '%s\n' "$provenance_out"
