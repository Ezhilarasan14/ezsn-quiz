use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Organization {
    pub id: String,
    pub name: String,
    #[serde(default)]
    #[sqlx(default)]
    pub branch_name: Option<String>,
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
    pub branch_name: Option<String>,
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
    #[serde(default)]
    #[sqlx(default)]
    pub email: Option<String>,
    #[serde(default)]
    #[sqlx(default)]
    pub name: Option<String>,
    #[serde(default)]
    #[sqlx(default)]
    pub avatar_url: Option<String>,
    #[serde(default = "default_admin_role")]
    #[sqlx(default)]
    pub role: String,
}

fn default_admin_role() -> String {
    "admin".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AdminAllowlistEntry {
    pub email: String,
    pub role: String,
    pub notes: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddAllowlistRequest {
    pub email: String,
    pub role: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleAuthUrlResponse {
    pub url: String,
    pub state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleCallbackQuery {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GoogleTokenResponse {
    pub access_token: String,
    pub id_token: Option<String>,
    pub token_type: String,
    pub expires_in: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GoogleUserInfo {
    pub sub: String,
    pub email: String,
    pub email_verified: Option<bool>,
    pub name: Option<String>,
    pub picture: Option<String>,
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
    #[serde(default)]
    #[sqlx(default)]
    pub host_name: Option<String>,
    #[serde(default)]
    #[sqlx(default)]
    pub branch_name: Option<String>,
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
    #[serde(default)]
    pub host_name: Option<String>,
    #[serde(default)]
    pub branch_name: Option<String>,
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
    #[serde(alias = "text", alias = "prompt")]
    pub question_text: String,
    #[serde(alias = "time_limit_secs")]
    pub time_limit_sec: i64,
    pub points: i64,
    pub order_index: i64,
    pub options: Vec<OptionDetail>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct OptionDetail {
    pub id: String,
    pub question_id: String,
    #[serde(alias = "text")]
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
    pub host_name: Option<String>,
    pub branch_name: Option<String>,
    pub questions: Vec<CreateQuestionRequest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateQuestionRequest {
    #[serde(alias = "text", alias = "prompt")]
    pub question_text: String,
    #[serde(alias = "time_limit_secs", default)]
    pub time_limit_sec: Option<i64>,
    #[serde(default)]
    pub points: Option<i64>,
    pub options: Vec<CreateOptionRequest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateOptionRequest {
    #[serde(alias = "text")]
    pub option_text: String,
    #[serde(default)]
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
    pub host_name: Option<String>,
    pub branch_name: Option<String>,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_create_quiz_with_option_text() {
        let json = r#"{
            "title": "General Knowledge",
            "questions": [{
                "question_text": "Capital of France?",
                "time_limit_sec": 20,
                "options": [
                    { "option_text": "Paris", "is_correct": true },
                    { "option_text": "London", "is_correct": false }
                ]
            }]
        }"#;

        let req: CreateQuizRequest = serde_json::from_str(json).expect("Should deserialize option_text");
        assert_eq!(req.title, "General Knowledge");
        assert_eq!(req.questions[0].options[0].option_text, "Paris");
        assert!(req.questions[0].options[0].is_correct);
    }

    #[test]
    fn test_deserialize_create_quiz_with_text_alias() {
        let json = r#"{
            "title": "General Knowledge",
            "questions": [{
                "text": "Capital of France?",
                "time_limit_secs": 20,
                "options": [
                    { "text": "Paris", "is_correct": true },
                    { "text": "London", "is_correct": false }
                ]
            }]
        }"#;

        let req: CreateQuizRequest = serde_json::from_str(json).expect("Should deserialize with text alias");
        assert_eq!(req.title, "General Knowledge");
        assert_eq!(req.questions[0].question_text, "Capital of France?");
        assert_eq!(req.questions[0].time_limit_sec, Some(20));
        assert_eq!(req.questions[0].options[0].option_text, "Paris");
        assert!(req.questions[0].options[0].is_correct);
    }
}
