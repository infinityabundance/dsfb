PHOSPHORIC_TARGET_DIR ?= /tmp/phosphoric-target

# Bootstrap binary path. Resolved by the bootstrap chain (see bootstrap/STAGE0.md).
# When pcc-stage1.bin is cached under .cache/bootstrap/, builds use it directly;
# otherwise the bootstrap fetcher downloads stage0, hashes it against
# bootstrap/bootstrap.toml, and runs it once to produce stage1.
PCC_STAGE_BIN ?= .cache/bootstrap/pcc-stage1.bin

.PHONY: verify verify-demo demo-visual verify-docs verify-hygiene verify-archive-inert verify-boot-phosphoric-only verify-direct-pe \
        verify-legendary verify-court-active verify-scaffold-historical \
        verify-pnp1-r5-producer manufacture-pnp1-r5-historical \
        verify-pnp2-pfi-producer manufacture-pnp2-pfi-historical \
        verify-pnp3-verdict-producer manufacture-pnp3-verdict-historical \
        verify-phosphoric-only verify-host-profile verify-trusted-blocks \
        verify-bootstrap-manifest verify-fixpoint verify-tcb-budget verify-retirement-dates \
        verify-invariant-manifest verify-conformance verify-effect-lattice verify-bounded-loops \
        verify-pe-writer-audit verify-attest \
        verify-manifest-doctrine verify-producer-gaps-doctrine \
        verify-residual-byte-layout verify-residual-r5-byte-layout \
        verify-pfi-layout verify-verdict-replay verify-malformed-pfi \
        verify-no-silent-authority \
        verify-source-asm-byte-equal \
        audit-selfhost-blockers \
        verify-no-excess-grammar verify-effect-closure-subset-of-manifest \
        verify-capability-graph-exact verify-boot-image-manifest-hash \
        verify-runtime-trace-subset-of-seal

# ----------------------------------------------------------------------------
# Existing v0 verification (pre-elevation; remains the gate of record).
# ----------------------------------------------------------------------------

verify: verify-hygiene verify-archive-inert verify-docs verify-pnp-boot verify-pcc-stage2-encodes-demo verify-pnp-dsfb verify-pcc-stage2-compiles-dsfb-demo verify-dsfb-pe verify-dsfb-pfi-runtime verify-boot-phosphoric-only verify-direct-pe verify-bootstrap-status verify-pcc-host-fs-roundtrip verify-pcc-boot-ir-parity verify-pcc-fail-corpus-stable verify-pcc-boot-asm-parity
	tools/qemu-run/run_uefi_demo.sh

verify-pcc-host-fs-roundtrip:
	@tools/verify/check_pcc_host_fs_roundtrip.sh

verify-pcc-boot-ir-parity:
	@tools/verify/check_pcc_boot_ir_parity.sh

verify-pcc-fail-corpus-stable:
	@tools/verify/check_pcc_fail_corpus_stable.sh

verify-pcc-boot-asm-parity:
	@tools/verify/check_pcc_boot_asm_parity.sh

verify-pnp-boot:
	@tools/verify/check_pnp_boot_producer.sh

# v0.2 marker 3: pcc-stage2.bin (= pcc-stage1.bin compiler/pcc2.phos) encodes
# the 23 demo constants byte-for-byte in its Pass T helpers + entry IMM.
verify-pcc-stage2-encodes-demo:
	@build/phase0/pcc-stage1.bin compiler/pcc2.phos build/phase0/pcc-stage2.bin >/dev/null
	@tools/verify/check_pcc_stage2_encodes_demo.sh

manufacture-pnp-boot-historical:
	@bash tools/phosphoric/boot_pnp/manufacture.sh

# v0.3: DSFB-PNP manufacture + verify gates.
manufacture-pnp-dsfb-historical:
	@bash tools/phosphoric/dsfb_pnp/manufacture.sh

verify-pnp-dsfb:
	@tools/verify/check_pnp_dsfb_producer.sh

verify-pcc-stage2-compiles-dsfb-demo:
	@tools/verify/check_pcc_stage2_compiles_dsfb_demo.sh

verify-dsfb-pe:
	@bash tools/phosphoric/write_dsfb_efi.sh >/dev/null
	@bash tools/phosphoric/verify_dsfb_pe.sh build/uefi-demo/dsfb/dsfb_demo.efi

# v0.3 Session 2 — runtime PFI0 capture: boots the dsfb demo, captures
# the 3 typed residual records on debug_data_port (0x500), wraps them in
# PFI0 header/footer, asserts byte-equal to tests/golden/dsfb_demo.pfi.
# tools/qemu-run/run_dsfb_demo.sh already does the comparison; this
# target makes it a named gate.
verify-dsfb-pfi-runtime: verify-dsfb-pe
	@bash tools/qemu-run/run_dsfb_demo.sh
	@cmp -s build/uefi-demo/dsfb/dsfb_demo.pfi tests/golden/dsfb_demo.pfi \
		|| { echo "[dsfb-pfi-runtime] FAIL: runtime PFI does not match tests/golden/dsfb_demo.pfi" >&2; exit 1; }
	@printf '[dsfb-pfi-runtime] OK — runtime PFI byte-equal to golden (%s)\n' \
		"$$(sha256sum tests/golden/dsfb_demo.pfi | awk '{print substr($$1,1,16)}')..."

# verify-invariants is gated on the internal invariant manifest. The manifest
# is tracked outside the public source tree; this rule no-ops when it is
# absent so the public verify path stays self-contained.
verify-invariants:
	@if [ -f untracked/internaldocs/docs/invariant_manifest.toml ]; then \
		tools/verify/check_invariants.sh; \
	fi

# verify-bootstrap-status emits an informational warning when the stage 0
# bootstrap manifest still contains placeholder fields. This does not fail
# the verify gate — the project is allowed to ship without a stage 0 binary
# during the development phase. The warning surfaces the residual gap.
verify-bootstrap-status:
	@if grep -q 'status[[:space:]]*=[[:space:]]*"active"' bootstrap/bootstrap.toml; then \
		echo "== phosphoric: bootstrap/bootstrap.toml stage 0 is active =="; \
	elif grep -q 'status[[:space:]]*=[[:space:]]*"SCAFFOLD"' bootstrap/bootstrap.toml; then \
		echo "== phosphoric: bootstrap/bootstrap.toml [[stage0]] is SCAFFOLD (informational) =="; \
		echo "   Phase 0 binary is produced and hash-pinned (build/phase0/pcc-stage0.bin)."; \
		echo "   See bootstrap/STAGE0_STATUS.md."; \
	elif grep -q '"PLACEHOLDER"\|= "TBD"' bootstrap/bootstrap.toml; then \
		echo "== phosphoric: bootstrap/bootstrap.toml [[stage0]] is PLACEHOLDER (informational) =="; \
		echo "   Stage 0 binary has not yet been produced. See phase0/HANDBOOTSTRAP.md"; \
		echo "   for the active attester runbook. The active build path does not invoke"; \
		echo "   any external toolchain; the bootstrap chain is not exercised yet."; \
	else \
		echo "== phosphoric: bootstrap/bootstrap.toml stage 0 is populated =="; \
	fi

verify-demo:
	tools/qemu-run/run_uefi_demo.sh

# Visual variant: runs the demo without the isa-debug-exit device so QEMU
# stays alive long enough to capture framebuffer screendumps. Produces:
#   build/uefi-demo/qemu-debug.log       debug-port log (same as headless run)
#   build/uefi-demo/demo-summary.txt     human-readable run summary
#   build/uefi-demo/screenshot.{ppm,png} canonical final framebuffer state
#   build/uefi-demo/screenshots/         per-checkpoint and post-redraw frames
demo-visual:
	tools/qemu-run/run_uefi_demo_visual.sh

verify-docs:
	tools/verify/check_docs.sh

verify-hygiene:
	tools/verify/check_repo_hygiene.sh

verify-archive-inert:
	tools/verify/check_archive_inert.sh

verify-boot-phosphoric-only:
	tools/verify/check_boot_phosphoric_only.sh

verify-direct-pe:
	tools/verify/check_direct_pe_negative_tests.sh

# ----------------------------------------------------------------------------
# Legendary verification (post-elevation aggregate gate).
#
# Composes the existing `verify` target with every host-profile gate from
# the elevation plan. Each sub-target invokes a Phosphoric host program
# (compiled from .phos source via $(PCC_STAGE_BIN)). When a sub-target's
# implementation is still scaffolding, the rule no-ops with a reminder.
#
# ----------------------------------------------------------------------------

# ----------------------------------------------------------------------------
# Verification tier split (2026-05-03 ASM authority cutover).
#
# Per docs/ASM_AUTHORITY_CUTOVER.md, gates are partitioned into two tiers:
#
#   verify-court-active           ASM-FREE active forensic court verification
#   verify-scaffold-historical    ASM-rooted historical scaffold verification
#
# verify-legendary is now an alias for verify-court-active. ASM-rooted gates
# (verify-fixpoint, verify-fixture-corpus, verify-quine-fixpoint,
# verify-source-asm-byte-equal, verify-court-p1-source-exit) execute the
# stage0 ASM scaffold or binaries derived from it; they are preserved as
# scaffold-historical evidence and are runnable via the explicit target,
# but they do not run as part of `make verify-legendary` after the split.
# ----------------------------------------------------------------------------

verify-court-active: \
	verify \
	verify-phosphoric-only \
	verify-host-profile \
	verify-trusted-blocks \
	verify-bootstrap-manifest \
	verify-fixture-razor \
	verify-residual-byte-layout \
	verify-residual-r5-byte-layout \
	verify-pfi-layout \
	verify-verdict-replay \
	verify-malformed-pfi \
	verify-court-a1-b1 \
	verify-court-d1-verdict \
	verify-court-b1-case-validity \
	verify-pnp1-r5-producer \
	verify-pnp2-pfi-producer \
	verify-pnp3-verdict-producer \
	verify-no-silent-authority \
	verify-classifier-doctrine \
	verify-residual-doctrine \
	verify-manifest-schema-doctrine \
	verify-effect-lattice-doctrine \
	verify-producer-gaps-doctrine \
	verify-tcb-budget \
	verify-retirement-dates \
	verify-invariant-manifest \
	verify-conformance \
	verify-effect-lattice \
	verify-bounded-loops \
	verify-pe-writer-audit \
	verify-attest \
	verify-manifest-doctrine \
	verify-residual-substrate

verify-scaffold-historical: \
	verify-fixpoint \
	verify-fixture-corpus \
	verify-quine-fixpoint \
	verify-source-asm-byte-equal \
	verify-court-p1-source-exit

verify-legendary: verify-court-active

# ----------------------------------------------------------------------------
# verify-manifest-doctrine — the five forensic-boundary gates.
#
# Per docs/TASK_SEAL_V0.md §9, the doctrine requires five hard gates that
# collectively form the post-failure boundary-reconstruction harness.
# Each gate is a thin shell wrapper delegating to host binaries; uniform
# exit codes (0 pass, 1 boundary violation, 2 scaffolding missing).
# ----------------------------------------------------------------------------

verify-manifest-doctrine: \
	verify-no-excess-grammar \
	verify-effect-closure-subset-of-manifest \
	verify-capability-graph-exact \
	verify-boot-image-manifest-hash \
	verify-runtime-trace-subset-of-seal

verify-no-excess-grammar:
	@tools/verify/verify_no_excess_grammar.sh || rc=$$?; \
	if [ "$${rc:-0}" = "2" ]; then exit 0; else exit "$${rc:-0}"; fi

verify-effect-closure-subset-of-manifest:
	@tools/verify/verify_effect_closure_subset_of_manifest.sh || rc=$$?; \
	if [ "$${rc:-0}" = "2" ]; then exit 0; else exit "$${rc:-0}"; fi

verify-capability-graph-exact:
	@tools/verify/verify_capability_graph_exact.sh || rc=$$?; \
	if [ "$${rc:-0}" = "2" ]; then exit 0; else exit "$${rc:-0}"; fi

verify-boot-image-manifest-hash:
	@tools/verify/verify_boot_image_manifest_hash.sh || rc=$$?; \
	if [ "$${rc:-0}" = "2" ]; then exit 0; else exit "$${rc:-0}"; fi

verify-runtime-trace-subset-of-seal:
	@tools/verify/verify_runtime_trace_subset_of_seal.sh || rc=$$?; \
	if [ "$${rc:-0}" = "2" ]; then exit 0; else exit "$${rc:-0}"; fi

# ----------------------------------------------------------------------------
# verify-residual-substrate — the DSFB-native residual emission harness.
#
# Runs every gate that exercises the dynamic residual stream (R1..R7),
# the .pfi container, and the .pfa audit package. Per the endoduction
# substrate plan, this target collects all post-Phase-1 gates so the
# residual primitive can be verified as one unit independent of the
# static manifest gates above.
# ----------------------------------------------------------------------------

.PHONY: verify-residual-substrate verify-drift-idempotent verify-audit-package-round-trip \
        verify-residual-chain-algorithm verify-drift-classification

verify-residual-substrate: \
	verify-residual-chain-algorithm \
	verify-drift-classification \
	verify-drift-idempotent \
	verify-audit-package-round-trip

verify-residual-chain-algorithm:
	@tools/verify/verify_residual_chain_algorithm.sh || rc=$$?; \
	if [ "$${rc:-0}" = "2" ]; then exit 0; else exit "$${rc:-0}"; fi

verify-drift-classification:
	@tools/verify/verify_drift_classification.sh || rc=$$?; \
	if [ "$${rc:-0}" = "2" ]; then exit 0; else exit "$${rc:-0}"; fi

verify-drift-idempotent:
	@tools/verify/verify_drift_idempotent.sh || rc=$$?; \
	if [ "$${rc:-0}" = "2" ]; then exit 0; else exit "$${rc:-0}"; fi

verify-audit-package-round-trip:
	@tools/verify/verify_audit_package_round_trip.sh || rc=$$?; \
	if [ "$${rc:-0}" = "2" ]; then exit 0; else exit "$${rc:-0}"; fi

# Each sub-target runs the corresponding host program. Until pcc.phos can compile
# the .phos source, each rule is gated on the existence of the compiled binary
# and otherwise emits a scaffolding reminder.

verify-phosphoric-only:
	@if [ -x build/host-tools/check_phosphoric_only ]; then \
		build/host-tools/check_phosphoric_only; \
	else \
		echo "[scaffold] tools/phosphoric-host/check_phosphoric_only.phos not yet built"; \
	fi

verify-host-profile:
	@if [ -x build/host-tools/check_host_profile_separation ]; then \
		build/host-tools/check_host_profile_separation; \
	else \
		echo "[scaffold] tools/phosphoric-host/check_host_profile_separation.phos not yet built"; \
	fi

verify-trusted-blocks:
	@if [ -x build/host-tools/check_trusted_blocks ]; then \
		build/host-tools/check_trusted_blocks; \
	else \
		echo "[scaffold] tools/phosphoric-host/check_trusted_blocks.phos not yet built"; \
	fi

verify-bootstrap-manifest:
	@tools/verify/check_bootstrap_manifest.sh

verify-fixpoint:
	@tools/verify/check_fixpoint_chain.sh

# Bootstrap chain harness (per panel review: build the harness before the
# next codegen pass). Each is invokable separately to avoid lengthening
# verify-legendary's runtime.

verify-chain-trace:
	@tools/verify/run_chain.sh phase0/phase0_compiler.phos 5

verify-chain-diff:
	@tools/verify/diff_stages.sh stage1 stage2 | head -80

verify-fixture-corpus:
	@tools/verify/fixture_corpus.sh

# Fixture razor — every fixture must satisfy the doctrine in
# docs/FIXTURE_RAZOR.md (behavior_class + ambiguity_removed + at least
# one of the four required_for_* doctrine flags + non-duplicate
# behavior_class unless justified). Reads tools/verify/fixture_manifest.toml.
verify-fixture-razor:
	@tools/verify/check_fixture_razor.sh

# Residual byte-layout — first residual-truth fixture's gate (Session 12).
# Pins the 32-byte R1 record field declaration order/widths in
# kernel/residual.phos AND the chain_step prime constants AND the
# byte-stable chain_hash output on a fixed cap_issue test vector.
# Fails closed if any of these drift; this is the spec the producer
# must satisfy when runtime R1 emission lands (Session 13+).
verify-residual-byte-layout:
	@tools/verify/check_residual_byte_layout.sh

# R5 mmio_touch byte-layout — Stream C Milestone B gate (Session 14).
# Peer of verify-residual-byte-layout for R5 (mmio_touch). Pins the
# R5 payload schema (declared_lo, declared_hi, observed_addr) and the
# chain_hash byte output [8A A2 CA 5E] on the canonical MMIO boundary
# test vector. Couples to mmio_boundary_violation.pfi by asserting the
# .pfi's record bytes literally match the schema.
verify-residual-r5-byte-layout:
	@tools/verify/check_residual_r5_byte_layout.sh

# PFI0 layout — Stream C Milestone A gate (Session 13). Pins the
# byte-stable layout of `.pfi` evidentiary case-file containers per
# docs/PFI0.md: PFI0 magic, residual_count u32 LE, manifest_hash,
# image_hash, stream_hash (== sha256 of concatenated record bytes),
# 32-byte Residual records with closed-taxonomy kinds, chain_hash
# re-derived per kernel/residual.phos chain_step, monotonic seq, zeroed
# reserved regions, and final_chain_hash anchored to the last record.
# Iterates every well-formed .pfi under tools/verify/fixtures/pfi/.
# Adversarial fixtures live under fixtures/pfi/malformed/ and are
# routed to verify-malformed-pfi.
verify-pfi-layout:
	@tools/verify/check_pfi_layout.sh

# Verdict replay — Stream C Milestone C gate (Session 15). Pins the
# expected classifier verdict bytes for each .pfi case file in
# tools/verify/fixtures/pfi/, asserts the verdict matches the canonical
# 6-line format from docs/FORENSIC_PRIMACY.md §3, and asserts byte-
# identical replay (same .pfi → same verdict bytes; no host-dependent
# text, no timestamps, no nondeterminism).
verify-verdict-replay:
	@tools/verify/check_verdict_replay.sh

# Malformed-case rejection — Stream C Milestone D gate (Session 16).
# Adversarial .pfi fixtures under tools/verify/fixtures/pfi/malformed/
# (bad chain_hash, seq gap, manifest hash mismatch, truncated record,
# bad kind) MUST cause the layout verifier to fail with a deterministic
# named violation. Asserts the court refuses bad evidence rather than
# silently accepting it or hanging.
verify-malformed-pfi:
	@tools/verify/check_malformed_pfi.sh

# Court Requirement A1/B1 — runs tools/court/emit_mmio_boundary_pfi.sh
# (a host-side bash + awk + sha256sum reference emitter) and asserts
# its 192-byte PFI0 output is byte-identical to the canonical anchor
# at tools/verify/fixtures/pfi/mmio_boundary_violation.pfi. Converts
# the hand-encoded anchor into a deterministic reference-emitted
# output for one declared MMIO boundary violation: the R5 record
# bytes, the chain_step mixer output, and the SHA-256 stream_hash are
# all reproducible from the inputs (declared_lo=0x1000,
# declared_hi=0x10FF, observed=0x1100). Host reference emitter — not
# a Phosphoric-compiled binary; that promotion is a later step.
verify-court-a1-b1:
	@tools/verify/check_court_a1_b1.sh

# Court Requirement D1 (2026-05-03) — host reference verdict path
# replay gate. Closes the host-reference loop:
#   input vector → R5 record → PFI0 case → 6-line verdict
# The gate runs the A1/B1 emitter to produce the 192-byte PFI0,
# pipes it through tools/court/verdict_from_pfi.sh (a host reference
# R5-only verdict tool), and asserts the produced 6-line verdict
# bytes are byte-identical to
# tools/verify/fixtures/verdicts/mmio_boundary_violation.expect.
# Earned claim: deterministic host reference court produces R5 PFI0
# bytes and derives the canonical MMIO_BOUNDARY_PRESSURE verdict
# bytes byte-identical to the locked expectation. NOT yet earned:
# Phosphoric runtime classifier execution / court runtime
# adjudication — chain remains host-side bash + awk + od + sha256sum.
verify-court-d1-verdict:
	@tools/verify/check_court_d1_verdict.sh

# Court Requirement B1 (narrow) (2026-05-03) — R5 case-validity gate.
# Adds the missing semantic invariant that no other gate enforces:
# the R5 mmio_touch payload must encode an actual MMIO boundary
# violation, i.e. observed ∉ [declared_lo, declared_hi]. Layout /
# chain / hash validation belong to verify-pfi-layout,
# verify-residual-r5-byte-layout, verify-malformed-pfi,
# verify-court-a1-b1, verify-court-d1-verdict respectively — B1 does
# not duplicate them. Earned claim: host reference court validates
# R5 payload semantics. NOT yet earned: general PFI validator,
# general R5 classifier, runtime enforcement, Phosphoric-compiled
# validation — chain remains host-side bash + awk + od.
verify-court-b1-case-validity:
	@tools/verify/check_court_b1_case_validity.sh

# Court PNP-1 (R5 narrow) — first Phosphoric-source byte producer.
# Active path: runs already-manufactured binaries from tools/court/pnp1_r5_artifacts/ and
# compares concatenated exit codes to the canonical 32-byte R5 record.
# Does NOT invoke phase0_stub (eligible for verify-court-active).
# Manufacture (one-time, via historical scaffold) is a separate target.
verify-pnp1-r5-producer:
	@tools/verify/check_pnp1_r5_producer.sh

# Manufacture PNP-1 R5 binaries via the historical ASM scaffold.
# This target uses phase0_stub ONCE per binary to produce 32 ELFs
# from tools/court/pnp1_r5_bytes/byte_NN.phos. Scaffold-historical
# tier: do NOT add to verify-court-active. Run explicitly:
#   make manufacture-pnp1-r5-historical
manufacture-pnp1-r5-historical:
	@mkdir -p tools/court/pnp1_r5_artifacts
	@stage0=untracked/internaldocs/phase0_producer/phase0_stub; \
	if [ ! -x $$stage0 ]; then \
	    echo "[manufacture-pnp1] stage0 ASM scaffold not built; run produce_stage0.sh first" >&2; exit 2; \
	fi; \
	for n in 00 01 02 03 04 05 06 07 08 09 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31; do \
	    src=tools/court/pnp1_r5_bytes/byte_$$n.phos; \
	    out=tools/court/pnp1_r5_artifacts/byte_$$n.bin; \
	    $$stage0 $$src $$out 2>/dev/null && chmod +x $$out || { echo "[manufacture-pnp1] failed at byte_$$n" >&2; exit 1; }; \
	done; \
	echo "[manufacture-pnp1] manufactured 32 binaries to tools/court/pnp1_r5_artifacts/byte_NN.bin"

# Court PNP-2 (PFI0 narrow) — first Phosphoric-source 192-byte producer.
# Active gate (verify-pnp2-pfi-producer) wired into verify-court-active;
# manufacture target uses phase0_stub once per source (scaffold-historical).
verify-pnp2-pfi-producer:
	@tools/verify/check_pnp2_pfi_producer.sh

manufacture-pnp2-pfi-historical:
	@mkdir -p tools/court/pnp2_pfi_artifacts
	@stage0=untracked/internaldocs/phase0_producer/phase0_stub; \
	if [ ! -x $$stage0 ]; then \
	    echo "[manufacture-pnp2] stage0 ASM scaffold not built" >&2; exit 2; \
	fi; \
	for n in $$(seq -f "%03g" 0 191); do \
	    src=tools/court/pnp2_pfi_bytes/byte_$$n.phos; \
	    out=tools/court/pnp2_pfi_artifacts/byte_$$n.bin; \
	    $$stage0 $$src $$out 2>/dev/null && chmod +x $$out || { echo "[manufacture-pnp2] failed at byte_$$n" >&2; exit 1; }; \
	done; \
	echo "[manufacture-pnp2] manufactured 192 binaries to tools/court/pnp2_pfi_artifacts/byte_NNN.bin"

# Court PNP-3 (verdict narrow) — first Phosphoric-source 104-byte producer.
verify-pnp3-verdict-producer:
	@tools/verify/check_pnp3_verdict_producer.sh

manufacture-pnp3-verdict-historical:
	@mkdir -p tools/court/pnp3_verdict_artifacts
	@stage0=untracked/internaldocs/phase0_producer/phase0_stub; \
	if [ ! -x $$stage0 ]; then \
	    echo "[manufacture-pnp3] stage0 ASM scaffold not built" >&2; exit 2; \
	fi; \
	for n in $$(seq -f "%03g" 0 103); do \
	    src=tools/court/pnp3_verdict_bytes/byte_$$n.phos; \
	    out=tools/court/pnp3_verdict_artifacts/byte_$$n.bin; \
	    $$stage0 $$src $$out 2>/dev/null && chmod +x $$out || { echo "[manufacture-pnp3] failed at byte_$$n" >&2; exit 1; }; \
	done; \
	echo "[manufacture-pnp3] manufactured 104 binaries to tools/court/pnp3_verdict_artifacts/byte_NNN.bin"

# Court Promotion P1-source (narrow) (2026-05-03) — first
# Phosphoric-source court artifact gate. Compiles
# tools/court/p1_source_exit.phos via the existing bootstrap chain
# (stage0 ASM scaffold → stage1 = stage0(phase0_compiler.phos) →
# court binary), runs the produced binary, and asserts it returns
# the canonical MMIO_BOUNDARY_PRESSURE EXIT code (= 6) by performing
# the observed > declared_hi runtime comparison (4352 > 4351). Also
# asserts stage0 and stage1 produce byte-identical output for this
# court source (same byte-equal property the 51 sealed fixtures
# already prove). The court source uses only the sealed 51-shape
# repertoire (Session J entry-call-with-INT-arg + Session P
# PARAM_MATCH); no phase0_stub.S edits, no new producer shapes, no
# new candidate promotions, no fixture corpus additions, no foreign
# builders. Earned claim: a Phosphoric-source-defined court artifact
# now derives the canonical EXIT=6 from canonical inputs through the
# existing bootstrap toolchain. NOT yet earned: byte emission
# (SYS_WRITE not in lowered shapes); on-device computation; classifier
# execution. The bootstrap chain still roots in phase0_stub.S — that
# remains scaffold-historical and is not extended by this gate.
verify-court-p1-source-exit:
	@tools/verify/check_court_p1_source_exit.sh

# No silent authority — Stream C Milestone E gate (Session 17). The
# load-bearing forensic invariant: every authority transition has
# either a manifest edge or a typed residual; no third path. Locks
# docs/NO_SILENT_AUTHORITY.md doctrine sentence and asserts kernel/
# residual.phos's record() fn (the canonical residual emission entry
# point) remains intact with the chain_hash composition logic. This is
# what distinguishes Phosphoric from a tracing/observability system.
verify-no-silent-authority:
	@tools/verify/verify_no_silent_authority.sh

# Source ↔ ASM byte-equal — STATE ENFORCEMENT gate (2026-05-02).
# Per user directive 2026-05-02 (memory feedback_state_enforcement_byte_equal.md):
# Phosphoric is in HARD FREEZE against scope expansion until
# phase0_compiler.phos compiles every fixture in fixture_manifest.toml
# to byte-identical output as the ASM stub stage0 = phase0_stub.
# When this gate is green (N/total == 1.0), byte-equal source ↔ ASM
# convergence is demonstrated and ASM retirement eligibility begins.
# Currently failing N=0/total=82; freeze remains in effect.
verify-source-asm-byte-equal:
	@tools/verify/verify_source_asm_byte_equal.sh

# Classifier doctrine — locks the residual-court framing in the
# classifier source-as-spec at tools/phosphoric-host/phosphoric_drift.phos.
# Fails closed if the closed DriftClass enum loses a name, if any
# log-analyzer vocabulary appears (score, maybe, probably, suspicious,
# anomaly, heuristic, ...), or if the exit-code table drifts from the
# canonical mapping in docs/FORENSIC_PRIMACY.md §3.
# Apex framing this gate protects: "drift-and-slew residual court,
# not log analyzer". Adjudication, not observability.
verify-classifier-doctrine:
	@tools/verify/check_classifier_doctrine.sh

# Residual doctrine — locks the evidence-side of the residual court
# in kernel/residual.phos (source-as-spec). Parallel to
# verify-classifier-doctrine. Fails closed if the closed 8-kind
# taxonomy loses a name, if the numeric kind→name table drifts,
# if the 32-byte Residual struct layout changes, if any
# log-analyzer vocabulary appears, or if the kind→Rn mapping
# diverges from FORENSIC_PRIMACY.md §2.
# Apex framing this gate protects: "drift-and-slew residual court,
# not log analyzer" — evidence side. Adjudication, not observability.
verify-residual-doctrine:
	@tools/verify/check_residual_doctrine.sh

# Manifest schema doctrine — locks the task-seal-side of the residual
# court in compiler/manifest.phos (host parser) and kernel/manifest.phos
# (runtime mirror). Third in the doctrine-gate trilogy alongside
# verify-classifier-doctrine and verify-residual-doctrine. Fails closed
# if any of these closed sets drift: 10 forbidden keys, 4 profiles,
# 4 capability kinds with their numeric ids, M-001..M-012 diagnostic
# codes, host/runtime shared-field agreement, log-analyzer vocabulary,
# or apex framing references. Distinct from verify-manifest-doctrine
# (the existing target name occupied by the five forensic-boundary
# subgates from TASK_SEAL_V0.md §9) — this one is schema-side.
verify-manifest-schema-doctrine:
	@tools/verify/check_manifest_doctrine.sh

# Effect lattice doctrine — locks the closed-set authority taxonomy
# at the compiler layer. Cross-checks docs/language/effect_lattice.toml
# (machine-readable manifest) against compiler/effects.phos (per-bit
# bitset layout + profile masks). Fails closed if any of: 4 alphabets,
# per-profile label set, E-001..E-007 namespace, bit-layout label
# coverage, profile alphabet bitmask values (31, 2016, runtime sum),
# or apex framing references drift.
verify-effect-lattice-doctrine:
	@tools/verify/check_effect_lattice_doctrine.sh

# Selfhost blocker audit — informational priority record (not a gate).
# Tallies per-extension fn-body blockers in phase0/phase0_compiler.phos
# to keep docs/SELFHOST_BACKLOG.md's numbers honest. Re-run after any
# producer extension lands; if bucket counts shift, update the doc.
# Exit 0 always — this is an evidence script, not a doctrine gate.
audit-selfhost-blockers:
	@tools/verify/audit_selfhost_blockers.sh

# Producer soundness gaps — symmetric counterpart of fixture corpus.
# fixture_corpus.sh locks correct producer paths byte-equally;
# this gate locks INCORRECT producer paths semantically (by rc), so
# known mis-lowerings stay enumerated and gated rather than silent.
# Reads tools/verify/producer_soundness_gaps.toml. Fails closed if any
# gap closes (producer extension fixed it; remove entry, add fixture)
# or shifts (gap moved to a new wrong outcome; investigate before
# re-pinning). Apex framing this gate protects: producer soundness
# gaps are tracked, not silent — evidence + adjudication.
verify-producer-gaps-doctrine:
	@tools/verify/check_producer_gaps_doctrine.sh

# Pass B-4: byte-equal fixpoint gate. Compiles tools/verify/fixtures/
# quine_self.phos to stage0, then iterates stage_N(any-input, stage_{N+1})
# for N=0..2 and asserts stage_N == stage_{N+1} byte-equal at each step.
# Fails if the chain diverges. This is the convergence gate (Path B
# destination).
verify-quine-fixpoint:
	@tools/verify/quine_fixpoint.sh

verify-tcb-budget:
	@if [ -x build/host-tools/check_tcb_budget ]; then \
		build/host-tools/check_tcb_budget; \
	else \
		echo "[scaffold] tools/phosphoric-host/check_tcb_budget.phos not yet built"; \
	fi

verify-retirement-dates:
	@if [ -x build/host-tools/check_retirement_dates ]; then \
		build/host-tools/check_retirement_dates; \
	else \
		echo "[scaffold] tools/phosphoric-host/check_retirement_dates.phos not yet built"; \
	fi

verify-invariant-manifest:
	@if [ -x build/host-tools/phosphoric_invariant_check ]; then \
		build/host-tools/phosphoric_invariant_check; \
	else \
		echo "[scaffold] tools/phosphoric-host/phosphoric_invariant_check.phos not yet built"; \
	fi

verify-conformance:
	@if [ -x build/host-tools/phosphoric_conform ]; then \
		build/host-tools/phosphoric_conform; \
	else \
		echo "[scaffold] tools/phosphoric-host/phosphoric_conform.phos not yet built"; \
	fi

verify-effect-lattice:
	@if [ -x build/host-tools/phosphoric_effect_check ]; then \
		build/host-tools/phosphoric_effect_check; \
	else \
		echo "[scaffold] tools/phosphoric-host/phosphoric_effect_check.phos not yet built"; \
	fi

verify-bounded-loops:
	@if [ -x build/host-tools/phosphoric_bound_check ]; then \
		build/host-tools/phosphoric_bound_check; \
	else \
		echo "[scaffold] tools/phosphoric-host/phosphoric_bound_check.phos not yet built"; \
	fi

verify-pe-writer-audit:
	@if [ -x build/host-tools/check_writer_audit_lines ]; then \
		build/host-tools/check_writer_audit_lines; \
	else \
		echo "[scaffold] tools/phosphoric-host/check_writer_audit_lines.phos not yet built"; \
	fi

verify-attest:
	@if [ -x build/host-tools/phosphoric_attest ]; then \
		build/host-tools/phosphoric_attest; \
	else \
		echo "[scaffold] tools/phosphoric-host/phosphoric_attest.phos not yet built"; \
	fi
