#include "dsfb.hpp"

#include <array>
#include <iostream>

int main() {
  dsfb::SemioticsEngine engine(32, 1.0, 1.0);
  const std::array<double, 5> samples{0.05, 0.11, 0.19, 0.31, 0.47};

  for (double sample : samples) {
    engine.push(sample);
    const auto status = engine.snapshot();
    std::cout << "syntax_code=" << static_cast<int>(status.syntax_code())
              << " syntax=" << status.syntax_label
              << " grammar=" << status.grammar_label
              << " semantics=" << status.semantic_label
              << " trust=" << status.trust_scalar() << '\n';
  }

  return 0;
}
