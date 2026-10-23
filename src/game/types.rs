use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;
use tokio::sync::mpsc;
use crate::ws::messages::ServerMessage;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GameStateKind {
    Lobby,
    QuestionIntro,
    QuestionActive,
    QuestionResult,
    Leaderboard,
    Podium,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionView {
    pub id: String,
    pub text: String,
    pub color_idx: usize, // 0: Red, 1: Blue, 2: Yellow, 3: Green
    pub shape: String,    // "triangle", "diamond", "circle", "square"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionView {
    pub id: String,
    pub index: usize,
    pub total: usize,
    pub question_text: String,
    pub time_limit_sec: u64,
    pub points: i64,
    pub options: Vec<OptionView>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardEntry {
    pub player_id: String,
    pub nickname: String,
    pub score: i64,
    pub rank: usize,
    pub rank_change: i32, // e.g. +1, -1, 0
    pub streak: u32,
    pub is_connected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodiumEntry {
    pub player_id: String,
    pub nickname: String,
    pub score: i64,
    pub rank: usize, // 1, 2, 3
    pub correct_count: usize,
    pub total_answered: usize,
}

#[derive(Debug, Clone)]
pub struct PlayerAnswer {
    pub option_id: String,
    pub response_time_ms: u64,
    pub is_correct: bool,
    pub points_awarded: i64,
    pub submitted_at: Instant,
}

pub struct PlayerSession {
    pub id: String,
    pub nickname: String,
    pub score: i64,
    pub streak: u32,
    pub correct_count: usize,
    pub total_answered: usize,
    pub rank_history: Vec<usize>,
    pub is_connected: bool,
    pub last_seen: Instant,
    pub tx: Option<mpsc::UnboundedSender<ServerMessage>>,
}

pub enum RoomState {
    Lobby,
    QuestionIntro {
        question_index: usize,
        ends_at: Instant,
        total_seconds: u64,
    },
    QuestionActive {
        question_index: usize,
        started_at: Instant,
        duration_secs: u64,
        player_answers: HashMap<String, PlayerAnswer>,
    },
    QuestionResult {
        question_index: usize,
        correct_option_ids: Vec<String>,
        option_counts: HashMap<String, u32>,
        player_answers: HashMap<String, PlayerAnswer>,
    },
    Leaderboard {
        question_index: usize,
        rankings: Vec<LeaderboardEntry>,
    },
    Podium {
        podium: Vec<PodiumEntry>,
        full_results: Vec<LeaderboardEntry>,
    },
}
