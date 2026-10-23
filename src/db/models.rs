use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct QuizSummary {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub question_count: i64,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuizDetail {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub questions: Vec<QuestionDetail>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct QuestionRow {
    pub id: String,
    pub quiz_id: String,
    pub question_text: String,
    pub time_limit_sec: i64,
    pub points: i64,
    pub order_index: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionDetail {
    pub id: String,
    pub quiz_id: String,
    pub question_text: String,
    pub time_limit_sec: i64,
    pub points: i64,
    pub order_index: i64,
    pub options: Vec<OptionDetail>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct OptionDetail {
    pub id: String,
    pub question_id: String,
    pub option_text: String,
    pub is_correct: bool,
    pub order_index: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateQuizRequest {
    pub title: String,
    pub description: Option<String>,
    pub questions: Vec<CreateQuestionRequest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateQuestionRequest {
    pub question_text: String,
    pub time_limit_sec: Option<i64>,
    pub points: Option<i64>,
    pub options: Vec<CreateOptionRequest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateOptionRequest {
    pub option_text: String,
    pub is_correct: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateQuizRequest {
    pub title: String,
    pub description: Option<String>,
    pub questions: Vec<CreateQuestionRequest>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct GameHistoryItem {
    pub id: String,
    pub quiz_id: String,
    pub quiz_title: String,
    pub room_pin: String,
    pub total_players: i64,
    pub created_at: NaiveDateTime,
    pub ended_at: Option<NaiveDateTime>,
    pub winner_nickname: Option<String>,
    pub winner_score: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct GamePlayerResultRow {
    pub id: String,
    pub game_id: String,
    pub player_id: String,
    pub nickname: String,
    pub total_score: i64,
    pub final_rank: i64,
    pub correct_count: i64,
    pub total_answered: i64,
}
