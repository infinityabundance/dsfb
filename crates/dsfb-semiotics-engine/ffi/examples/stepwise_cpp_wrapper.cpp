#include "dsfb.hpp"

#include <array>
#include <iostream>

int main() {
  dsfb::SemioticsEngine engine(16, 0.72, 1.0);
  const std::array<double, 8> residuals{0.04, 0.08, 0.12, 0.18,
                                        0.27, 0.39, 0.55, 0.63};

  for (std::size_t step = 0; step < residuals.size(); ++step) {
    engine.push(static_cast<double>(step), residuals[step]);
    const auto snapshot = engine.snapshot();
    std::cout << "step=" << static_cast<unsigned long long>(snapshot.raw.step)
              << " syntax=" << snapshot.syntax_label
              << " grammar=" << snapshot.grammar_label << "/"
              << snapshot.grammar_reason_text
              << " semantics=" << snapshot.semantic_label
              << " trust=" << snapshot.trust_scalar() << '\n';
  }

  return 0;
}
