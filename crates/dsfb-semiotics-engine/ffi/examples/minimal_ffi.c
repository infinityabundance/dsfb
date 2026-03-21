#include "dsfb_semiotics_engine.h"

#include <stdio.h>
int main(void) {
  const double samples[] = {0.05, 0.12, 0.18, 0.34, 0.52, 0.61};
  EngineHandle *handle = dsfb_semiotics_engine_create(32, 1.0, 1.0);
  if (handle == NULL) {
    char error_buffer[256] = {0};
    dsfb_semiotics_engine_copy_last_error(error_buffer, sizeof(error_buffer));
    fprintf(stderr, "failed to create DSFB semiotics engine handle: %s\n",
            error_buffer);
    return 1;
  }

  for (size_t index = 0; index < sizeof(samples) / sizeof(samples[0]); ++index) {
    DsfbCurrentStatus status = {0};
    double trust = 0.0;
    char syntax_label[64] = {0};
    char grammar_label[64] = {0};
    char semantic_label[64] = {0};

    if (dsfb_semiotics_engine_push_sample(handle, (double)index, samples[index]) !=
        DSFB_FFI_OK) {
      char error_buffer[256] = {0};
      dsfb_semiotics_engine_copy_last_error(error_buffer, sizeof(error_buffer));
      fprintf(stderr, "push failed at step %zu: %s\n", index, error_buffer);
      dsfb_semiotics_engine_destroy(handle);
      return 1;
    }

    DsfbFfiResult syntax_result = dsfb_semiotics_engine_copy_current_syntax_label(
        handle, syntax_label, sizeof(syntax_label));
    DsfbFfiResult grammar_result = dsfb_semiotics_engine_copy_current_grammar_label(
        handle, grammar_label, sizeof(grammar_label));
    DsfbFfiResult semantic_result = dsfb_semiotics_engine_copy_current_semantic_label(
        handle, semantic_label, sizeof(semantic_label));
    if (dsfb_semiotics_engine_current_status(handle, &status) != DSFB_FFI_OK ||
        dsfb_semiotics_engine_current_trust_scalar(handle, &trust) != DSFB_FFI_OK ||
        (syntax_result != DSFB_FFI_OK &&
         syntax_result != DSFB_FFI_BUFFER_TOO_SMALL) ||
        (grammar_result != DSFB_FFI_OK &&
         grammar_result != DSFB_FFI_BUFFER_TOO_SMALL) ||
        (semantic_result != DSFB_FFI_OK &&
         semantic_result != DSFB_FFI_BUFFER_TOO_SMALL)) {
      char error_buffer[256] = {0};
      dsfb_semiotics_engine_copy_last_error(error_buffer, sizeof(error_buffer));
      fprintf(stderr, "status query failed at step %zu: %s\n", index, error_buffer);
      dsfb_semiotics_engine_destroy(handle);
      return 1;
    }

    printf(
        "step=%llu syntax_code=%d syntax=%s grammar=%s semantic=%s trust=%.3f\n",
        (unsigned long long)status.step, (int)status.syntax_code, syntax_label,
        grammar_label, semantic_label, trust);
  }

  dsfb_semiotics_engine_destroy(handle);
  return 0;
}
