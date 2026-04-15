use dsfb_gray::AuditTrace;

#[test]
fn loom_smoke_model_runs() {
    loom::model(|| {
        let trace = AuditTrace::new();
        assert_eq!(trace.total_count(), 0);
    });
}
