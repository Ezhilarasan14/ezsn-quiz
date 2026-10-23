pub struct ScoreCalculator;

impl ScoreCalculator {
    /// Calculate points for a question answer.
    /// - `is_correct`: whether the selected option matches the correct answer
    /// - `max_points`: base points for the question (e.g. 1000)
    /// - `response_time_ms`: elapsed time from question start in milliseconds
    /// - `total_duration_ms`: total question duration in milliseconds
    /// - `current_streak`: player's current consecutive correct answers prior to this question
    pub fn calculate(
        is_correct: bool,
        max_points: i64,
        response_time_ms: u64,
        total_duration_ms: u64,
        current_streak: u32,
    ) -> (i64, u32) {
        if !is_correct {
            return (0, 0); // Incorrect answer gives 0 points and resets streak
        }

        let total_ms = if total_duration_ms == 0 { 1000.0 } else { total_duration_ms as f64 };
        let elapsed_ms = (response_time_ms as f64).min(total_ms).max(0.0);

        // Speed multiplier: 1.0 at 0ms elapsed, 0.5 at full duration
        let speed_ratio = elapsed_ms / total_ms;
        let speed_factor = (1.0 - speed_ratio * 0.5).max(0.5).min(1.0);

        let base_awarded = (max_points as f64 * speed_factor).round() as i64;

        // Streak bonus: +100 for 2-streak, +200 for 3-streak, etc. capped at +500
        let new_streak = current_streak + 1;
        let streak_bonus = if new_streak > 1 {
            ((new_streak - 1) as i64 * 100).min(500)
        } else {
            0
        };

        (base_awarded + streak_bonus, new_streak)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_incorrect_answer_gives_zero_and_resets_streak() {
        let (points, streak) = ScoreCalculator::calculate(false, 1000, 100, 20000, 5);
        assert_eq!(points, 0);
        assert_eq!(streak, 0);
    }

    #[test]
    fn test_instant_correct_answer_gives_full_points() {
        let (points, streak) = ScoreCalculator::calculate(true, 1000, 0, 20000, 0);
        assert_eq!(points, 1000);
        assert_eq!(streak, 1);
    }

    #[test]
    fn test_half_time_correct_answer() {
        let (points, streak) = ScoreCalculator::calculate(true, 1000, 10000, 20000, 0);
        // speed_ratio = 0.5 -> speed_factor = 1.0 - 0.25 = 0.75 -> 750 points
        assert_eq!(points, 750);
        assert_eq!(streak, 1);
    }

    #[test]
    fn test_last_second_correct_answer() {
        let (points, streak) = ScoreCalculator::calculate(true, 1000, 20000, 20000, 0);
        // speed_ratio = 1.0 -> speed_factor = 0.5 -> 500 points
        assert_eq!(points, 500);
        assert_eq!(streak, 1);
    }

    #[test]
    fn test_streak_bonuses() {
        // First correct: streak becomes 1, bonus 0
        let (p1, s1) = ScoreCalculator::calculate(true, 1000, 0, 20000, 0);
        assert_eq!(p1, 1000);
        assert_eq!(s1, 1);

        // Second correct: streak becomes 2, bonus 100
        let (p2, s2) = ScoreCalculator::calculate(true, 1000, 0, 20000, 1);
        assert_eq!(p2, 1100);
        assert_eq!(s2, 2);

        // High streak capped at 500 bonus
        let (p_cap, s_cap) = ScoreCalculator::calculate(true, 1000, 0, 20000, 10);
        assert_eq!(p_cap, 1500);
        assert_eq!(s_cap, 11);
    }
}
