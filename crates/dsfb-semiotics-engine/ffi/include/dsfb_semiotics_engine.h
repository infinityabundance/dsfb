#ifndef DSFB_SEMIOTICS_ENGINE_H
#define DSFB_SEMIOTICS_ENGINE_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef enum DsfbFfiResult {
  DSFB_FFI_OK = 0,
  DSFB_FFI_NULL_HANDLE = 1,
  DSFB_FFI_NULL_OUTPUT = 2,
  DSFB_FFI_INVALID_ARGUMENT = 3,
  DSFB_FFI_ENGINE_ERROR = 4,
  DSFB_FFI_BUFFER_TOO_SMALL = 5
} DsfbFfiResult;

typedef enum DsfbSyntaxCode {
  DSFB_SYNTAX_WEAKLY_STRUCTURED_BASELINE_LIKE = 0,
  DSFB_SYNTAX_MIXED_STRUCTURED = 1,
  DSFB_SYNTAX_PERSISTENT_OUTWARD_DRIFT = 2,
  DSFB_SYNTAX_COORDINATED_OUTWARD_RISE = 3,
  DSFB_SYNTAX_DISCRETE_EVENT_LIKE = 4,
  DSFB_SYNTAX_CURVATURE_RICH_TRANSITION = 5,
  DSFB_SYNTAX_INWARD_COMPATIBLE_CONTAINMENT = 6,
  DSFB_SYNTAX_NEAR_BOUNDARY_RECURRENT = 7,
  DSFB_SYNTAX_BOUNDED_OSCILLATORY_STRUCTURED = 8,
  DSFB_SYNTAX_STRUCTURED_NOISY_ADMISSIBLE = 9,
  DSFB_SYNTAX_OTHER = 255
} DsfbSyntaxCode;

typedef enum DsfbGrammarState {
  DSFB_GRAMMAR_ADMISSIBLE = 0,
  DSFB_GRAMMAR_BOUNDARY = 1,
  DSFB_GRAMMAR_VIOLATION = 2
} DsfbGrammarState;

typedef enum DsfbGrammarReason {
  DSFB_REASON_ADMISSIBLE = 0,
  DSFB_REASON_BOUNDARY = 1,
  DSFB_REASON_RECURRENT_BOUNDARY_GRAZING = 2,
  DSFB_REASON_SUSTAINED_OUTWARD_DRIFT = 3,
  DSFB_REASON_ABRUPT_SLEW_VIOLATION = 4,
  DSFB_REASON_ENVELOPE_VIOLATION = 5
} DsfbGrammarReason;

typedef enum DsfbSemanticDisposition {
  DSFB_SEMANTIC_MATCH = 0,
  DSFB_SEMANTIC_COMPATIBLE_SET = 1,
  DSFB_SEMANTIC_AMBIGUOUS = 2,
  DSFB_SEMANTIC_UNKNOWN = 3
} DsfbSemanticDisposition;

typedef struct DsfbCurrentStatus {
  uint64_t step;
  double time;
  double residual_norm;
  double drift_norm;
  double slew_norm;
  double trust_scalar;
  uint64_t history_buffer_capacity;
  uint64_t current_history_len;
  uint64_t offline_history_len;
  DsfbSyntaxCode syntax_code;
  DsfbGrammarState grammar_state;
  DsfbGrammarReason grammar_reason;
  DsfbSemanticDisposition semantic_disposition;
} DsfbCurrentStatus;

typedef struct EngineHandle EngineHandle;

EngineHandle *dsfb_semiotics_engine_create(size_t history_buffer_capacity,
                                           double envelope_radius,
                                           double dt);
EngineHandle *dsfb_semiotics_engine_create_with_channels(
    size_t history_buffer_capacity, size_t channel_count, double envelope_radius,
    double dt);
void dsfb_semiotics_engine_destroy(EngineHandle *handle);
DsfbFfiResult dsfb_semiotics_engine_push_sample(EngineHandle *handle,
                                                double time,
                                                double residual_value);
DsfbFfiResult dsfb_semiotics_engine_push_sample_batch(
    EngineHandle *handle, const double *times, const double *residual_values,
    size_t sample_count);
DsfbFfiResult dsfb_semiotics_engine_current_status(const EngineHandle *handle,
                                                   DsfbCurrentStatus *out_status);
DsfbFfiResult dsfb_semiotics_engine_current_trust_scalar(
    const EngineHandle *handle, double *out_trust);
DsfbFfiResult dsfb_semiotics_engine_copy_current_syntax_label(
    const EngineHandle *handle, char *out_buffer, size_t buffer_len);
DsfbFfiResult dsfb_semiotics_engine_copy_current_grammar_label(
    const EngineHandle *handle, char *out_buffer, size_t buffer_len);
DsfbFfiResult dsfb_semiotics_engine_copy_current_grammar_reason_text(
    const EngineHandle *handle, char *out_buffer, size_t buffer_len);
DsfbFfiResult dsfb_semiotics_engine_copy_current_semantic_label(
    const EngineHandle *handle, char *out_buffer, size_t buffer_len);
size_t dsfb_semiotics_engine_last_error_length(void);
DsfbFfiResult dsfb_semiotics_engine_copy_last_error(char *out_buffer,
                                                    size_t buffer_len);
DsfbFfiResult dsfb_semiotics_engine_reset(EngineHandle *handle);

#ifdef __cplusplus
}
#endif

#endif
