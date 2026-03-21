#include "dsfb_semiotics_engine.h"

#include <array>
#include <iostream>

int main() {
  const std::array<double, 6> samples{0.05, 0.12, 0.18, 0.34, 0.52, 0.61};
  EngineHandle *handle = dsfb_semiotics_engine_create(32, 1.0, 1.0);
  if (handle == nullptr) {
    std::array<char, 256> error_buffer{};
    dsfb_semiotics_engine_copy_last_error(error_buffer.data(), error_buffer.size());
    std::cerr << "failed to create DSFB semiotics engine handle: "
              << error_buffer.data() << '\n';
    return 1;
  }

  for (std::size_t index = 0; index < samples.size(); ++index) {
    DsfbCurrentStatus status{};
    double trust = 0.0;
    std::array<char, 64> syntax_label{};
    std::array<char, 64> grammar_label{};
    std::array<char, 64> semantic_label{};

    if (dsfb_semiotics_engine_push_sample(handle, static_cast<double>(index),
                                          samples[index]) != DSFB_FFI_OK) {
      std::array<char, 256> error_buffer{};
      dsfb_semiotics_engine_copy_last_error(error_buffer.data(),
                                            error_buffer.size());
      std::cerr << "push failed at step " << index << ": " << error_buffer.data()
                << '\n';
      dsfb_semiotics_engine_destroy(handle);
      return 1;
    }

    DsfbFfiResult syntax_result = dsfb_semiotics_engine_copy_current_syntax_label(
        handle, syntax_label.data(), syntax_label.size());
    DsfbFfiResult grammar_result = dsfb_semiotics_engine_copy_current_grammar_label(
        handle, grammar_label.data(), grammar_label.size());
    DsfbFfiResult semantic_result = dsfb_semiotics_engine_copy_current_semantic_label(
        handle, semantic_label.data(), semantic_label.size());
    if (dsfb_semiotics_engine_current_status(handle, &status) != DSFB_FFI_OK ||
        dsfb_semiotics_engine_current_trust_scalar(handle, &trust) != DSFB_FFI_OK ||
        (syntax_result != DSFB_FFI_OK &&
         syntax_result != DSFB_FFI_BUFFER_TOO_SMALL) ||
        (grammar_result != DSFB_FFI_OK &&
         grammar_result != DSFB_FFI_BUFFER_TOO_SMALL) ||
        (semantic_result != DSFB_FFI_OK &&
         semantic_result != DSFB_FFI_BUFFER_TOO_SMALL)) {
      std::array<char, 256> error_buffer{};
      dsfb_semiotics_engine_copy_last_error(error_buffer.data(),
                                            error_buffer.size());
      std::cerr << "status query failed at step " << index << ": "
                << error_buffer.data() << '\n';
      dsfb_semiotics_engine_destroy(handle);
      return 1;
    }

    std::cout << "step=" << static_cast<unsigned long long>(status.step)
              << " syntax_code=" << static_cast<int>(status.syntax_code)
              << " syntax=" << syntax_label.data()
              << " grammar=" << grammar_label.data()
              << " semantic=" << semantic_label.data() << " trust=" << trust
              << '\n';
  }

  dsfb_semiotics_engine_destroy(handle);
  return 0;
}
