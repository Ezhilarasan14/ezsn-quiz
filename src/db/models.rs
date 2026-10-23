use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Organization {
    pub id: String,
    pub name: String,
    pub logo_url: Option<String>,
    pub primary_color: String,
    pub secondary_color: String,
    pub background_color: String,
    pub button_color: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateOrganizationRequest {
    pub name: String,
    pub logo_url: Option<String>,
    pub primary_color: Option<String>,
    pub secondary_color: Option<String>,
    pub background_color: Option<String>,
    pub button_color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AdminUser {
    pub id: String,
    pub organization_id: Option<String>,
    pub username: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
    pub admin: AdminUser,
    pub organization: Option<Organization>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct QuizSummary {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub question_count: i64,
    pub organization_id: Option<String>,
    pub theme_logo_url: Option<String>,
    pub theme_primary_color: Option<String>,
    pub theme_secondary_color: Option<String>,
    pub theme_bg_color: Option<String>,
    pub theme_btn_color: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuizDetail {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub organization_id: Option<String>,
    pub theme_logo_url: Option<String>,
    pub theme_primary_color: Option<String>,
    pub theme_secondary_color: Option<String>,
    pub theme_bg_color: Option<String>,
    pub theme_btn_color: Option<String>,
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CreateQuizRequest {
    pub title: String,
    pub description: Option<String>,
    pub organization_id: Option<String>,
    pub theme_logo_url: Option<String>,
    pub theme_primary_color: Option<String>,
    pub theme_secondary_color: Option<String>,
    pub theme_bg_color: Option<String>,
    pub theme_btn_color: Option<String>,
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateQuizRequest {
    pub title: String,
    pub description: Option<String>,
    pub organization_id: Option<String>,
    pub theme_logo_url: Option<String>,
    pub theme_primary_color: Option<String>,
    pub theme_secondary_color: Option<String>,
    pub theme_bg_color: Option<String>,
    pub theme_btn_color: Option<String>,
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
    pub email: Option<String>,
    pub total_score: i64,
    pub final_rank: i64,
    pub correct_count: i64,
    pub total_answered: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PlayerHistoryItem {
    pub game_id: String,
    pub quiz_title: String,
    pub room_pin: String,
    pub score: i64,
    pub rank: i64,
    pub total_players: i64,
    pub correct_count: i64,
    pub total_answered: i64,
    pub played_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerStats {
    pub email: String,
    pub nickname: String,
    pub quizzes_played: i64,
    pub total_score: i64,
    pub wins_count: i64,
    pub podium_count: i64,
    pub total_correct: i64,
    pub total_answered: i64,
    pub avg_accuracy: f64,
    pub history: Vec<PlayerHistoryItem>,
}
