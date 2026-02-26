#[test]
fn test_update() {
    let mut obs = HretObserver::new(2, 1, vec![0, 0], 0.95, vec![0.9], vec![1.0, 1.0], vec![1.0], vec![vec![1.0, 1.0]]).unwrap();
    let (delta, weights, _, _) = obs.update(vec![0.1, 0.2]).unwrap();
    assert_eq!(weights.len(), 2);
}