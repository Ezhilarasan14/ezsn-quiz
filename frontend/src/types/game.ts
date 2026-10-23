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
