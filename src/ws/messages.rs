use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::game::types::{LeaderboardEntry, OptionView, PodiumEntry};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LobbyPlayer {
    pub player_id: String,
    pub nickname: String,
    pub is_connected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum ClientMessage {
    HostConnect {
        pin: String,
        token: String,
    },
    PlayerConnect {
        pin: String,
        nickname: String,
        player_id: Option<String>,
    },
    HostStartGame,
    HostNextQuestion,
    HostKickPlayer {
        player_id: String,
    },
    PlayerSubmitAnswer {
        question_id: String,
        option_id: String,
    },
    Ping,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum ServerMessage {
    Connected {
        role: String, // "host" or "player"
        pin: String,
        player_id: Option<String>,
        nickname: Option<String>,
        is_reconnect: bool,
    },
    LobbyState {
        pin: String,
        quiz_title: String,
        player_count: usize,
        players: Vec<LobbyPlayer>,
    },
    PlayerJoined {
        player_id: String,
        nickname: String,
        player_count: usize,
    },
    PlayerLeft {
        player_id: String,
        nickname: String,
        player_count: usize,
    },
    PlayerKicked {
        reason: String,
    },
    GameStarting {
        countdown_secs: u64,
    },
    QuestionIntro {
        question_index: usize,
        total_questions: usize,
        question_text: String,
        countdown_secs: u64,
    },
    QuestionActive {
        question_index: usize,
        total_questions: usize,
        question_text: String,
        time_limit_sec: u64,
        options: Vec<OptionView>,
        already_answered: bool,
    },
    AnswerAccepted {
        question_index: usize,
    },
    AnswerRejected {
        reason: String,
    },
    HostAnswerCountUpdate {
        answered_count: usize,
        total_players: usize,
    },
    QuestionResult {
        question_index: usize,
        correct_option_ids: Vec<String>,
        option_counts: HashMap<String, u32>,
        player_correct: Option<bool>,
        points_awarded: Option<i64>,
        total_score: Option<i64>,
        streak: Option<u32>,
    },
    LeaderboardUpdate {
        rankings: Vec<LeaderboardEntry>,
        my_rank: Option<usize>,
        my_score: Option<i64>,
    },
    Podium {
        podium: Vec<PodiumEntry>,
        full_results: Vec<LeaderboardEntry>,
        my_rank: Option<usize>,
    },
    Error {
        message: String,
    },
    Pong,
}
