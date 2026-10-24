import type { ThemeConfig } from './quiz';
import type { PodiumEntry, PlayerQuizSummary } from './game';

export interface LobbyPlayer {
  player_id: string;
  nickname: string;
  is_connected: boolean;
}

export interface OptionView {
  id: string;
  text: string;
  order_num: number;
}

export interface LeaderboardEntry {
  player_id: string;
  nickname: string;
  score: number;
  streak: number;
  rank: number;
  total_time_ms?: number;
}

// Client -> Server messages
export type ClientMessage =
  | { type: 'host_connect'; payload: { pin: string; token: string } }
  | { type: 'player_connect'; payload: { pin: string; nickname: string; player_id?: string; email?: string } }
  | { type: 'host_start_game'; payload?: Record<string, never> }
  | { type: 'host_next_question'; payload?: Record<string, never> }
  | { type: 'host_kick_player'; payload: { player_id: string } }
  | { type: 'player_submit_answer'; payload: { question_id: string; option_id: string } }
  | { type: 'ping' };

// Server -> Client messages
export type ServerMessage =
  | {
      type: 'connected';
      payload: {
        role: 'host' | 'player';
        pin: string;
        player_id?: string;
        nickname?: string;
        is_reconnect: boolean;
      };
    }
  | {
      type: 'lobby_state';
      payload: {
        pin: string;
        quiz_title: string;
        player_count: number;
        players: LobbyPlayer[];
      };
    }
  | {
      type: 'player_joined';
      payload: {
        player_id: string;
        nickname: string;
        player_count: number;
      };
    }
  | {
      type: 'player_left';
      payload: {
        player_id: string;
        nickname: string;
        player_count: number;
      };
    }
  | {
      type: 'player_kicked';
      payload: {
        reason: string;
      };
    }
  | {
      type: 'game_starting';
      payload: {
        countdown_secs: number;
      };
    }
  | {
      type: 'question_intro';
      payload: {
        question_index: number;
        total_questions: number;
        question_text: string;
        countdown_secs: number;
      };
    }
  | {
      type: 'question_active';
      payload: {
        question_index: number;
        total_questions: number;
        question_text: string;
        time_limit_sec: number;
        options: OptionView[];
        already_answered: boolean;
      };
    }
  | {
      type: 'answer_accepted';
      payload: {
        question_index: number;
        response_time_ms?: number;
      };
    }
  | {
      type: 'answer_rejected';
      payload: {
        reason: string;
      };
    }
  | {
      type: 'host_answer_count_update';
      payload: {
        answered_count: number;
        total_players: number;
      };
    }
  | {
      type: 'question_result';
      payload: {
        question_index: number;
        correct_option_ids: string[];
        option_counts: Record<string, number>;
        player_correct?: boolean;
        points_awarded?: number;
        total_score?: number;
        streak?: number;
      };
    }
  | {
      type: 'leaderboard_update';
      payload: {
        rankings: LeaderboardEntry[];
        my_rank?: number;
        my_score?: number;
      };
    }
  | {
      type: 'podium';
      payload: {
        podium: PodiumEntry[];
        full_results: LeaderboardEntry[];
        my_rank?: number;
        player_summary?: PlayerQuizSummary;
      };
    }
  | {
      type: 'theme_config';
      payload: {
        theme: ThemeConfig;
      };
    }
  | {
      type: 'error';
      payload: {
        message: string;
      };
    }
  | {
      type: 'pong';
    };
