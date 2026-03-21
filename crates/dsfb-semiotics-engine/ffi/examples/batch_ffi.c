#include "dsfb_semiotics_engine.h"

#include <stdio.h>

int main(void) {
  EngineHandle *handle =
      dsfb_semiotics_engine_create_with_channels(16, 2, 1.0, 1.0);
  if (handle == NULL) {
    return 1;
  }

  const double times[] = {0.0, 1.0, 2.0};
  const double residuals[] = {
      0.10, 0.00,
      0.14, 0.01,
      0.18, 0.03,
  };
  DsfbCurrentStatus status = {0};

  if (dsfb_semiotics_engine_push_sample_batch(handle, times, residuals, 3) !=
          DSFB_FFI_OK ||
      dsfb_semiotics_engine_current_status(handle, &status) != DSFB_FFI_OK) {
    dsfb_semiotics_engine_destroy(handle);
    return 2;
  }

  printf("step=%llu syntax_code=%d grammar_reason=%d trust=%.3f\n",
         (unsigned long long)status.step, (int)status.syntax_code,
         (int)status.grammar_reason, status.trust_scalar);

  dsfb_semiotics_engine_destroy(handle);
  return 0;
}
