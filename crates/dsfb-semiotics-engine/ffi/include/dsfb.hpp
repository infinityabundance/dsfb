#ifndef DSFB_SEMIOTICS_ENGINE_HPP
#define DSFB_SEMIOTICS_ENGINE_HPP

#include "dsfb_semiotics_engine.h"

#include <array>
#include <stdexcept>
#include <string>
#include <utility>
#include <vector>

namespace dsfb {

struct StatusSnapshot {
  DsfbCurrentStatus raw{};
  std::string syntax_label;
  std::string grammar_label;
  std::string grammar_reason_text;
  std::string semantic_label;

  [[nodiscard]] double trust_scalar() const noexcept { return raw.trust_scalar; }
  [[nodiscard]] DsfbSyntaxCode syntax_code() const noexcept { return raw.syntax_code; }
  [[nodiscard]] DsfbGrammarState grammar_state() const noexcept {
    return raw.grammar_state;
  }
  [[nodiscard]] DsfbGrammarReason grammar_reason() const noexcept {
    return raw.grammar_reason;
  }
  [[nodiscard]] DsfbSemanticDisposition semantic_disposition() const noexcept {
    return raw.semantic_disposition;
  }
};

// TRACE:INTERFACE:IFACE-CPP-WRAPPER:Header-only C plus plus wrapper:RAII wrapper exposes the C ABI through an idiomatic C++17 surface.
class SemioticsEngine {
 public:
  explicit SemioticsEngine(std::size_t history_buffer_capacity,
                           double envelope_radius = 1.0, double dt = 1.0)
      : handle_(dsfb_semiotics_engine_create(history_buffer_capacity,
                                             envelope_radius, dt)),
        next_time_(0.0),
        dt_(dt) {
    if (handle_ == nullptr) {
      throw std::runtime_error(last_error_string());
    }
  }

  SemioticsEngine(std::size_t history_buffer_capacity, std::size_t channel_count,
                  double envelope_radius, double dt)
      : handle_(dsfb_semiotics_engine_create_with_channels(history_buffer_capacity,
                                                           channel_count,
                                                           envelope_radius, dt)),
        next_time_(0.0),
        dt_(dt) {
    if (handle_ == nullptr) {
      throw std::runtime_error(last_error_string());
    }
  }

  ~SemioticsEngine() noexcept {
    if (handle_ != nullptr) {
      dsfb_semiotics_engine_destroy(handle_);
    }
  }

  SemioticsEngine(const SemioticsEngine&) = delete;
  SemioticsEngine& operator=(const SemioticsEngine&) = delete;

  SemioticsEngine(SemioticsEngine&& other) noexcept
      : handle_(other.handle_), next_time_(other.next_time_), dt_(other.dt_) {
    other.handle_ = nullptr;
  }

  SemioticsEngine& operator=(SemioticsEngine&& other) noexcept {
    if (this != &other) {
      if (handle_ != nullptr) {
        dsfb_semiotics_engine_destroy(handle_);
      }
      handle_ = other.handle_;
      next_time_ = other.next_time_;
      dt_ = other.dt_;
      other.handle_ = nullptr;
    }
    return *this;
  }

  void reset() {
    check_result(dsfb_semiotics_engine_reset(handle_));
    next_time_ = 0.0;
  }

  void push(double residual_value) {
    push(next_time_, residual_value);
    next_time_ += dt_;
  }

  void push(double time, double residual_value) {
    check_result(dsfb_semiotics_engine_push_sample(handle_, time, residual_value));
    next_time_ = time + dt_;
  }

  void push_batch(const std::vector<double>& times,
                  const std::vector<double>& residual_values) {
    if (times.empty()) {
      check_result(dsfb_semiotics_engine_push_sample_batch(handle_, nullptr, nullptr, 0));
      return;
    }
    check_result(dsfb_semiotics_engine_push_sample_batch(handle_, times.data(),
                                                         residual_values.data(),
                                                         times.size()));
    next_time_ = times.back() + dt_;
  }

  [[nodiscard]] DsfbCurrentStatus current_status() const {
    DsfbCurrentStatus status{};
    check_result(dsfb_semiotics_engine_current_status(handle_, &status));
    return status;
  }

  [[nodiscard]] double current_trust_scalar() const {
    double trust = 0.0;
    check_result(dsfb_semiotics_engine_current_trust_scalar(handle_, &trust));
    return trust;
  }

  [[nodiscard]] std::string current_syntax_label() const {
    return copy_label(dsfb_semiotics_engine_copy_current_syntax_label);
  }

  [[nodiscard]] std::string current_grammar_label() const {
    return copy_label(dsfb_semiotics_engine_copy_current_grammar_label);
  }

  [[nodiscard]] std::string current_grammar_reason_text() const {
    return copy_label(dsfb_semiotics_engine_copy_current_grammar_reason_text);
  }

  [[nodiscard]] std::string current_semantic_label() const {
    return copy_label(dsfb_semiotics_engine_copy_current_semantic_label);
  }

  [[nodiscard]] StatusSnapshot snapshot() const {
    StatusSnapshot snapshot{};
    snapshot.raw = current_status();
    snapshot.syntax_label = current_syntax_label();
    snapshot.grammar_label = current_grammar_label();
    snapshot.grammar_reason_text = current_grammar_reason_text();
    snapshot.semantic_label = current_semantic_label();
    return snapshot;
  }

 private:
  using CopyLabelFn = DsfbFfiResult (*)(const EngineHandle*, char*, std::size_t);

  [[nodiscard]] static std::string last_error_string() {
    std::size_t required = dsfb_semiotics_engine_last_error_length();
    std::string buffer(required, '\0');
    const DsfbFfiResult result =
        dsfb_semiotics_engine_copy_last_error(buffer.data(), buffer.size());
    if (result != DSFB_FFI_OK && result != DSFB_FFI_BUFFER_TOO_SMALL) {
      return "unknown DSFB FFI error";
    }
    if (!buffer.empty() && buffer.back() == '\0') {
      buffer.pop_back();
    }
    return buffer;
  }

  static void check_result(DsfbFfiResult result) {
    if (result == DSFB_FFI_OK || result == DSFB_FFI_BUFFER_TOO_SMALL) {
      return;
    }
    throw std::runtime_error(last_error_string());
  }

  [[nodiscard]] std::string copy_label(CopyLabelFn function) const {
    std::array<char, 256> buffer{};
    check_result(function(handle_, buffer.data(), buffer.size()));
    return std::string(buffer.data());
  }

  EngineHandle* handle_;
  double next_time_;
  double dt_;
};

}  // namespace dsfb

#endif
