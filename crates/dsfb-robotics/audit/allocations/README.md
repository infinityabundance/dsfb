# Allocation evidence

This directory holds Valgrind-massif outputs that empirically verify
the steady-state allocation claim in
[`docs/allocation_budget.md`](../../docs/allocation_budget.md).

## Reproduce

```bash
valgrind --tool=massif --pages-as-heap=yes \
    --massif-out-file=audit/allocations/<slug>.massif \
    target/release/paper-lock <slug> > /dev/null
ms_print audit/allocations/<slug>.massif | head -100
```

Expected: a stepped initialisation phase, a flat plateau through the
streaming loop, a final freeing step at process exit. No upward
staircase during the streaming section.

The Nix flake at the crate root pulls in `valgrind` so
`nix develop` makes this step one command.
