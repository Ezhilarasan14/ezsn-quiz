use ezsn_quiz::game::scoring::ScoreCalculator;

#[test]
fn test_scoring_speed_decay_and_streak() {
    // 1. Instant response gives 1000 points
    let (pts, streak) = ScoreCalculator::calculate(true, 1000, 0, 20000, 0);
    assert_eq!(pts, 1000);
    assert_eq!(streak, 1);

    // 2. Response at midpoint (10s of 20s) gives 750 points
    let (pts, streak) = ScoreCalculator::calculate(true, 1000, 10000, 20000, 0);
    assert_eq!(pts, 750);
    assert_eq!(streak, 1);

    // 3. Response at deadline (20s) gives 500 points
    let (pts, streak) = ScoreCalculator::calculate(true, 1000, 20000, 20000, 0);
    assert_eq!(pts, 500);
    assert_eq!(streak, 1);

    // 4. Consecutive streak gives streak bonus: streak 2 adds +100
    let (pts, streak) = ScoreCalculator::calculate(true, 1000, 0, 20000, 1);
    assert_eq!(pts, 1100);
    assert_eq!(streak, 2);

    // 5. Wrong answer gives 0 points and resets streak
    let (pts, streak) = ScoreCalculator::calculate(false, 1000, 500, 20000, 10);
    assert_eq!(pts, 0);
    assert_eq!(streak, 0);
}
