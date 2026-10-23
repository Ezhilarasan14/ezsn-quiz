export type GameState = 'LOBBY' | 'QUESTION' | 'REVEAL' | 'LEADERBOARD' | 'PODIUM' | 'FINISHED';

export interface Player {
  id: string;
  nickname: string;
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
  score: number;
}
