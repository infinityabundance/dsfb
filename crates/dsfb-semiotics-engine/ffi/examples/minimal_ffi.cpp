#include "dsfb_semiotics_engine.h"

#include <iostream>

int main() {
  EngineHandle *handle = dsfb_semiotics_engine_create(32, 1.0, 1.0);
  if (handle == nullptr) {
    std::cerr << "failed to create DSFB semiotics engine handle\n";
    return 1;
  }

  dsfb_semiotics_engine_push_sample(handle, 0.0, 0.10);
  dsfb_semiotics_engine_push_sample(handle, 1.0, 0.18);
  dsfb_semiotics_engine_push_sample(handle, 2.0, 0.35);

  DsfbCurrentStatus status{};
  if (dsfb_semiotics_engine_current_status(handle, &status) == DSFB_FFI_OK) {
    std::cout << "step=" << static_cast<unsigned long long>(status.step)
              << " residual_norm=" << status.residual_norm
              << " grammar_state=" << static_cast<int>(status.grammar_state)
              << " semantic=" << static_cast<int>(status.semantic_disposition)
              << '\n';
  }

  dsfb_semiotics_engine_destroy(handle);
  return 0;
}
