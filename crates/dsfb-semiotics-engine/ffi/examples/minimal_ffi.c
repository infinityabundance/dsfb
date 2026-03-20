#include "dsfb_semiotics_engine.h"

#include <stdio.h>

int main(void) {
  EngineHandle *handle = dsfb_semiotics_engine_create(32, 1.0, 1.0);
  if (handle == NULL) {
    fprintf(stderr, "failed to create DSFB semiotics engine handle\n");
    return 1;
  }

  dsfb_semiotics_engine_push_sample(handle, 0.0, 0.10);
  dsfb_semiotics_engine_push_sample(handle, 1.0, 0.18);
  dsfb_semiotics_engine_push_sample(handle, 2.0, 0.35);

  DsfbCurrentStatus status = {0};
  if (dsfb_semiotics_engine_current_status(handle, &status) == DSFB_FFI_OK) {
    printf("step=%llu residual_norm=%f grammar_state=%d semantic=%d\n",
           (unsigned long long)status.step, status.residual_norm,
           (int)status.grammar_state, (int)status.semantic_disposition);
  }

  dsfb_semiotics_engine_destroy(handle);
  return 0;
}
