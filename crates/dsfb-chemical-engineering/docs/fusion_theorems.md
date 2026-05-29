# The DSFB Detector-Fusion Theorem, in Full

This document states the deterministic detector-fusion calculus of DSFB-Chemical-Engineering in its
entirety — the per-detector grammar that feeds it, the fusion operator, episode formation, every
episode functional, the compression statistic, and the properties that make the construction
replayable.

It is the standalone counterpart of **Appendix “The DSFB fusion theorem, in full”** in
[`paper/dsfb_chemical_engineering.tex`](../paper/dsfb_chemical_engineering.tex) and the exact
specification implemented in
[`crates/dsfb-chemical-engineering-edge/src/fusion.rs`](../crates/dsfb-chemical-engineering-edge/src/fusion.rs).
Provenance — which results are classical and to whom they are due — is in
[§ Provenance & credits](#provenance--credits).

> **Scope note.** The fusion calculus is original to this work. It is assembled on classical
> foundations (Shannon entropy; multi-sensor decision fusion; PCA/SPE residual energy; moving-average
> and finite-difference estimators), each credited below to its originator.

---

## 1. Objects and notation

| Symbol | Meaning |
|---|---|
| $\mathcal{D}=\{1,\dots,m\}$ | detectors; $m=\lvert\mathcal{D}\rvert$ |
| $\varphi:\mathcal{D}\to\mathcal{F}$ | detector-to-family map |
| $i\in\{0,\dots,n-1\}$ | discrete step; $n=\max_d T_d$ (longest timeline) |
| $\mathcal{D}_i=\{d: i<T_d\}$ | detectors present at step $i$ |
| $\mathcal{G}$ | grammar alphabet, $\lvert\mathcal{G}\rvert=8$ |
| $g_{d,i}\in\mathcal{G}$ | grammar state of detector $d$ at step $i$ |
| $K$ | family-quorum threshold (`min_families`, default $2$) |
| $L_{\min}$ | minimum episode length (`min_steps`, default $2$) |
| $(r,\delta,\sigma)$ | residual, drift, slew triple |

---

## 2. The per-detector grammar (fusion inputs)

Each detector emits a one-sided exceedance stream $r_k=\max(0,\ \mathrm{score}_k/\tau-1)\ge 0$
(see the detector-equations appendix), from which the grammar engine computes a causal **drift** and
**slew**:

$$\delta_k=\frac{1}{w}\sum_{j=k-w+1}^{k} r_j, \qquad \sigma_k=\frac{r_k-r_{k-1}}{\Delta t_k},\quad \sigma_0=0.$$

The triple $(r_k,\delta_k,\sigma_k)$ is classified against a calibrated admissibility envelope

$$\mathcal{E}=[r_{\min},r_{\max}]\times[\delta_{\min},\delta_{\max}]\times[\sigma_{\min},\sigma_{\max}]$$

with a grazing band, yielding one of eight grammar states

$$\mathcal{G}=\{\mathrm{NOM},\ \mathrm{DA},\ \mathrm{SS},\ \mathrm{EV},\ \mathrm{BG},\ \mathrm{RC},\ \mathrm{CP},\ \mathrm{SF}\}$$

(nominal, drift-accumulation, slew-spike, envelope-violation, boundary-grazing, recovery, compound,
sensor-fault). Write $\mathcal{G}^{\ne}=\mathcal{G}\setminus\{\mathrm{NOM}\}$ for the **non-nominal**
states and $\mathcal{G}^{\star}=\mathcal{G}\setminus\{\mathrm{NOM},\mathrm{SF}\}$ for the **firing**
states — non-nominal evidence that is not an isolated sensor fault.

---

## 3. The fusion operator $\Phi$

For each step $i$:

$$
\begin{aligned}
\text{fire:}\quad & \chi_{d,i}=\mathbf{1}\!\left[d\in\mathcal{D}_i\ \wedge\ g_{d,i}\in\mathcal{G}^{\star}\right]\\[2pt]
\text{families / quorum:}\quad & \Phi_i=\{\varphi(d): \chi_{d,i}=1\},\qquad q_i=\mathbf{1}\!\left[\lvert\Phi_i\rvert\ge K\right]\\[2pt]
\text{consensus:}\quad & f_i=\tfrac{1}{m}\textstyle\sum_{d\in\mathcal{D}}\chi_{d,i}\\[2pt]
\text{distribution:}\quad & c_i(g)=\bigl\lvert\{d\in\mathcal{D}_i: g_{d,i}=g\}\bigr\rvert,\qquad M_i=\textstyle\sum_{g}c_i(g)=\lvert\mathcal{D}_i\rvert\\[2pt]
\text{entropy:}\quad & H_i=-\textstyle\sum_{g\in\mathcal{G}}\frac{c_i(g)}{M_i}\,\ln\frac{c_i(g)}{M_i}\quad(0\ln 0:=0;\ H_i:=0\text{ if }M_i=0)
\end{aligned}
$$

---

## 4. Episode formation

**Definition (fused episode).** A *fused episode* is a maximal contiguous interval $[s,e]$ with
$q_i=1$ for every $i\in[s,e]$, retained iff its length $L=e-s+1\ge L_{\min}$. Episodes are emitted in
increasing $s$; by maximality they are pairwise disjoint and separated by at least one sub-quorum
step.

---

## 5. Episode functionals

Over an episode $[s,e]$ of length $L$:

$$
\begin{aligned}
\text{consensus strength:}\quad & \bar f=\tfrac{1}{L}\textstyle\sum_{i=s}^{e} f_i\\[2pt]
\text{disagreement entropy:}\quad & \bar H=\tfrac{1}{L}\textstyle\sum_{i=s}^{e} H_i\\[2pt]
\text{participants / families:}\quad & P=\{d:\exists\, i\in[s,e],\ \chi_{d,i}=1\},\qquad \text{families}=\varphi(P)\\[2pt]
\text{dominant motif:}\quad & g^{\star}=\operatorname*{arg\,max}_{g\in\mathcal{G}^{\star}}\ \textstyle\sum_{i=s}^{e}\sum_{d}\mathbf{1}\!\left[\chi_{d,i}=1\wedge g_{d,i}=g\right]\\[2pt]
\text{peak drift / slew:}\quad & \hat\delta=\max_{\,i\in[s,e],\,\chi_{d,i}=1}\lvert\delta_{d,i}\rvert,\qquad \hat\sigma=\max_{\,i\in[s,e],\,\chi_{d,i}=1}\lvert\sigma_{d,i}\rvert
\end{aligned}
$$

Ties in $g^{\star}$ are broken by the fixed total order on $\mathcal{G}$, so the motif is
deterministic.

---

## 6. Compression

With $B=\sum_{d\in\mathcal{D}}\lvert\{i:\text{breach}_{d,i}\}\rvert$ the raw per-detector breach
volume and $E$ the number of fused episodes, the **compression ratio** is

$$\rho=\frac{B}{E}\quad(E>0).$$

---

## 7. Properties

Each statement formalises a property asserted in the paper’s *Formal definitions and deterministic
properties* section.

### Theorem 1 (Determinism and well-definedness)

Fix the timelines $(g_{d,i})$, the family map $\varphi$, the thresholds $K,L_{\min}$, and the total
orders on $\mathcal{G}$ and on detector identifiers. Then $\Phi$ produces a unique, ordered set of
episodes, and every functional $(\bar f,\bar H,g^{\star},\hat\delta,\hat\sigma,P,\varphi(P))$ is a
single-valued function of those inputs.

*Proof.* Each of $\chi_{d,i},\Phi_i,q_i,f_i,c_i,H_i$ is a finite sum, set-cardinality, or quotient
over the finite index set $\mathcal{D}\times\{0,\dots,n-1\}$, hence determined by the inputs. The
binary sequence $(q_i)$ has a unique decomposition into maximal runs, and $L\ge L_{\min}$ is a
deterministic predicate on each run. The only candidate for ambiguity, the $\arg\max$ in $g^{\star}$,
is resolved by the fixed total order on $\mathcal{G}$. No randomness or order-sensitive
floating-point reduction enters. ∎

### Lemma 2 (Entropy bounds and the disagreement certificate)

For every step with $M_i\ge 1$,

$$0\le H_i\le \ln\bigl\lvert\{g:c_i(g)>0\}\bigr\rvert\le\ln\min(8,M_i).$$

Moreover $H_i=0$ iff all present detectors occupy a single state (unanimity), and $\bar H=0$ iff
every step of the episode is unanimous.

*Proof.* $H_i$ is the Shannon entropy of the empirical distribution $p_i(g)=c_i(g)/M_i$.
Non-negativity and the maximum $\ln(\#\,\text{support})$ are Gibbs’ inequality; the support has at
most $\min(\lvert\mathcal{G}\rvert,M_i)=\min(8,M_i)$ atoms. A probability vector has zero entropy iff
it is a point mass, i.e. unanimity. As $\bar H$ is a mean of non-negative terms, it vanishes iff each
does. ∎

> **Remark.** Hence $\bar H>0$ is a *certificate* of genuine cross-detector disagreement within an
> episode — information a scalar “$k$ alarms fired” vote discards. This is the precise sense in which
> DSFB fusion preserves disagreement as a first-class output.

### Proposition 3 (Quorum implies multi-family support)

Every fused episode satisfies $\lvert\Phi_i\rvert\ge K$ at each step $i\in[s,e]$, hence
$\lvert\varphi(P)\rvert\ge K$. No episode is sustained by detectors of fewer than $K$ distinct
families; for $K\ge 2$, a single-family alarm storm never forms an episode.

*Proof.* $q_i=1$ requires $\lvert\Phi_i\rvert\ge K$ for all $i$ in the episode, and
$\Phi_i\subseteq\varphi(P)$, so $\lvert\varphi(P)\rvert\ge\max_i\lvert\Phi_i\rvert\ge K$. ∎

### Theorem 4 (Read-only compression)

The episode map is a deterministic function of the breach evidence that never modifies it: the
multiset $\{(d,i):\text{breach}_{d,i}\}$ is an input, not an output. The ratio $\rho=B/E$ reports the
reduction from $B$ raw breach-steps to $E$ operator-readable intervals, with $\rho\ge 1$ whenever
$B\ge E$. Compression changes representation only — it adds, alters, or removes no detector evidence
— and therefore does not by itself improve detection accuracy.

*Proof.* $B$ and $E$ are obtained by counting; neither $\Phi$ nor any functional writes back into a
detector stream. The breach multiset is thus invariant under fusion and $\rho$ is a derived
statistic; $\rho\ge 1$ is immediate when $E\le B$. ∎

---

## Provenance & credits

The fusion calculus is original to this work; it is assembled on classical foundations, credited here
to their originators. Items marked **this work** are the contribution of the present disclosure.

| Result / object | Where | Originator(s) |
|---|---|---|
| Shannon entropy of the step state-distribution | §3 (entropy) | C. E. Shannon, 1948 |
| Maximum-entropy / non-negativity bound | Lemma 2 | Gibbs’ inequality; Shannon, 1948 |
| Multi-sensor (k-of-n) decision-fusion lineage | §3 (fire/quorum) | Z. Chair & P. K. Varshney, 1986 |
| Causal windowed-mean drift | §2 (drift) | classical moving-average smoothing |
| First-difference slew | §2 (slew) | classical finite differences |
| Upstream PCA $T^2$ / SPE residual energy | detector appendix | H. Hotelling, 1933; J. E. Jackson, 1991 |
| EWMA / CUSUM / Mann–Kendall detector inputs | detector appendix | S. W. Roberts, 1959; E. S. Page, 1954; H. B. Mann, 1945 |
| Tennessee Eastman case-study benchmark | case studies | J. J. Downs & E. F. Vogel, 1993 |
| Drift–slew grammar, admissibility envelope, family-quorum operator $\Phi$, episode calculus, disagreement-entropy functional $\bar H$, compression ratio $\rho$ | this document | **this work** (R. de Beer) |

### References

- C. E. Shannon (1948). *A mathematical theory of communication.* Bell System Technical Journal, 27(3), 379–423.
- J. W. Gibbs (1902). *Elementary Principles in Statistical Mechanics.* (Gibbs’ inequality.)
- Z. Chair & P. K. Varshney (1986). *Optimal data fusion in multiple sensor detection systems.* IEEE Trans. Aerospace and Electronic Systems, AES-22(1), 98–101.
- H. Hotelling (1933). *Analysis of a complex of statistical variables into principal components.* Journal of Educational Psychology, 24(6), 417–441.
- J. E. Jackson (1991). *A User’s Guide to Principal Components.* Wiley.
- S. W. Roberts (1959). *Control chart tests based on geometric moving averages.* Technometrics, 1(3), 239–250.
- E. S. Page (1954). *Continuous inspection schemes.* Biometrika, 41(1/2), 100–115.
- H. B. Mann (1945). *Nonparametric tests against trend.* Econometrica, 13(3), 245–259.
- J. J. Downs & E. F. Vogel (1993). *A plant-wide industrial process control problem.* Computers & Chemical Engineering, 17(3), 245–255.
