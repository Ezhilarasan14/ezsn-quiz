export type GameState = 'LOBBY' | 'QUESTION' | 'REVEAL' | 'LEADERBOARD' | 'PODIUM' | 'FINISHED';

export interface Player {
  id: string;
  nickname: string;
  email?: string;
  score: number;
  streak: number;
  rank?: number;
  connected?: boolean;
}

export interface PlayerAnswerResult {
  correct: boolean;
  pointsEarned: number;
  totalScore: number;
  streak: number;
  rank: number;
}

export interface OptionDistribution {
  optionId: string;
  count: number;
  isCorrect?: boolean;
}

export interface QuestionStats {
  totalAnswered: number;
  totalPlayers: number;
  distribution: OptionDistribution[];
  correctOptionId: string;
}

export interface PodiumEntry {
  rank: number;
  id: string;
  nickname: string;
  email?: string;
  score: number;
}

export interface PlayerHistoryItem {
  game_id: string;
  quiz_title: string;
  room_pin: string;
  score: number;
  rank: number;
  total_players: number;
  correct_count: number;
  total_answered: number;
  played_at: string;
}

export interface PlayerStats {
  email: string;
  nickname: string;
  quizzes_played: number;
  total_score: number;
  wins_count: number;
  podium_count: number;
  total_correct: number;
  total_answered: number;
  avg_accuracy: number;
  history: PlayerHistoryItem[];
}

export interface QuestionReviewItem {
  question_id: string;
  question_index: number;
  question_text: string;
  selected_option_id: string | null;
  selected_option_text: string | null;
  correct_option_id: string;
  correct_option_text: string;
  is_correct: boolean;
  points_awarded: number;
  response_time_ms: number;
  time_limit_sec: number;
}

export interface PlayerQuizSummary {
  total_score: number;
  correct_count: number;
  incorrect_count: number;
  total_questions: number;
  accuracy_percentage: number;
  final_rank: number;
  total_players: number;
  total_time_ms: number;
  reviews: QuestionReviewItem[];
}
