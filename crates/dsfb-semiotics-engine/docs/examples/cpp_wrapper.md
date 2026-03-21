# C++ Wrapper Quickstart

The raw C ABI in [`ffi/include/dsfb_semiotics_engine.h`](../../ffi/include/dsfb_semiotics_engine.h)
is the narrowest integration boundary, but many C++ hosts want a drop-in RAII surface instead of
manual opaque-handle management. For that case the crate now ships the single-header C++17 wrapper
[`ffi/include/dsfb.hpp`](../../ffi/include/dsfb.hpp).

The wrapper is intentionally boring:

- RAII lifetime management for the engine handle
- `push(value)` or `push(time, value)` without raw handle calls
- `push_batch(times, residual_values)` for row-major batch ingress through the same C ABI
- numeric status codes still come from the underlying C ABI
- human-readable labels are copied into `std::string`
- errors raise `std::runtime_error` in the common path

Thirty-second usage:

```cpp
#include "dsfb.hpp"

int main() {
  dsfb::SemioticsEngine engine(32);
  engine.push(0.12);
  auto snapshot = engine.snapshot();
}
```

Compile-smoked examples:

- raw ABI: [`ffi/examples/minimal_ffi.cpp`](../../ffi/examples/minimal_ffi.cpp)
- header-only wrapper hello-world: [`ffi/examples/minimal_cpp_wrapper.cpp`](../../ffi/examples/minimal_cpp_wrapper.cpp)
- stepwise loop example: [`ffi/examples/stepwise_cpp_wrapper.cpp`](../../ffi/examples/stepwise_cpp_wrapper.cpp)

The wrapper is an architectural convenience layer over the same bounded online engine. It does not
change the scientific posture of the crate and it should not be read as a certification claim.
