pub mod answers;
pub mod gameplay;
pub mod registration;
pub mod state_sync;

use crate::db::models::{Organization, QuizDetail};
use crate::db::DbPool;
use crate::game::types::*;
use crate::ws::messages::ServerMessage;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

pub const SHAPES: [&str; 4] = ["triangle", "diamond", "circle", "square"];

pub struct GameRoom {
    pub pin: String,
    pub game_id: String,
    pub host_token: String,
    pub quiz: QuizDetail,
    pub organization: Option<Organization>,
    pub db_pool: DbPool,
    pub state: RoomState,
    pub host_tx: Option<mpsc::UnboundedSender<ServerMessage>>,
    pub host_connection_id: Option<String>,
    pub players: HashMap<String, PlayerSession>,
    pub all_answers: HashMap<usize, HashMap<String, PlayerAnswer>>,
    pub created_at: Instant,
    pub last_activity: Instant,
    pub timer_token: u64,
    pub question_time_secs: u64,
    pub room_abandoned_timeout_secs: u64,
    pub max_room_lifetime_secs: u64,
}

impl GameRoom {
    pub fn new(
        pin: String,
        game_id: String,
        host_token: String,
        quiz: QuizDetail,
        organization: Option<Organization>,
        db_pool: DbPool,
        question_time_secs: u64,
        room_abandoned_timeout_secs: u64,
        max_room_lifetime_secs: u64,
    ) -> Self {
        Self {
            pin,
            game_id,
            host_token,
            quiz,
            organization,
            db_pool,
            state: RoomState::Lobby,
            host_tx: None,
            host_connection_id: None,
            players: HashMap::new(),
            all_answers: HashMap::new(),
            created_at: Instant::now(),
            last_activity: Instant::now(),
            timer_token: 0,
            question_time_secs: if question_time_secs == 0 { 20 } else { question_time_secs },
            room_abandoned_timeout_secs: if room_abandoned_timeout_secs == 0 { 1800 } else { room_abandoned_timeout_secs },
            max_room_lifetime_secs: if max_room_lifetime_secs == 0 { 18000 } else { max_room_lifetime_secs },
        }
    }

    pub fn get_theme(&self) -> ThemeConfig {
        let org_logo = self.organization.as_ref()
            .and_then(|o| o.logo_url.clone())
            .filter(|s| !s.is_empty());

        let org_name = self.organization.as_ref()
            .map(|o| o.name.clone())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "EZSN Quiz".to_string());

        let org_primary = self.organization.as_ref()
            .map(|o| o.primary_color.clone())
            .filter(|s| !s.is_empty());

        let logo_url = self.quiz.theme_logo_url.clone()
            .filter(|s| !s.is_empty())
            .or(org_logo);

        let primary_color = self.quiz.theme_primary_color.clone()
            .filter(|s| !s.is_empty())
            .or(org_primary)
            .or_else(|| Some("#0F172A".to_string()));

        let branch_name = self.organization.as_ref()
            .and_then(|o| o.branch_name.clone())
            .filter(|s| !s.is_empty())
            .or_else(|| self.quiz.branch_name.clone().filter(|s| !s.is_empty()))
            .or_else(|| Some("Main Campus".to_string()));

        let host_name = self.quiz.host_name.clone()
            .filter(|s| !s.is_empty())
            .or_else(|| Some("Ezhil Arasan".to_string()));

        ThemeConfig {
            logo_url,
            primary_color,
            secondary_color: self.quiz.theme_secondary_color.clone().filter(|s| !s.is_empty()).or_else(|| Some("#ec4899".to_string())),
            background_color: self.quiz.theme_bg_color.clone().filter(|s| !s.is_empty()).or_else(|| Some("#0f172a".to_string())),
            button_color: self.quiz.theme_btn_color.clone().filter(|s| !s.is_empty()).or_else(|| Some("#3b82f6".to_string())),
            org_name: Some(org_name),
            host_name,
            branch_name,
        }
    }

    pub fn touch(&mut self) {
        self.last_activity = Instant::now();
    }

    pub fn is_abandoned(&self) -> bool {
        // Absolute maximum lifetime safety cap (e.g. 5 hours)
        if self.created_at.elapsed() > Duration::from_secs(self.max_room_lifetime_secs) {
            return true;
        }

        let inactive_duration = self.last_activity.elapsed();

        // If host is disconnected and room has been inactive for abandoned timeout (e.g. 30 mins)
        if self.host_tx.is_none() && inactive_duration > Duration::from_secs(self.room_abandoned_timeout_secs) {
            return true;
        }

        // If room has no host AND no connected players, and inactive for abandoned timeout
        if self.host_tx.is_none() && self.active_player_count() == 0 && inactive_duration > Duration::from_secs(self.room_abandoned_timeout_secs) {
            return true;
        }

        // Safety idle timeout: if completely inactive (no answers, no host clicks, no pings) for 3 hours
        if inactive_duration > Duration::from_secs(10800) {
            return true;
        }

        false
    }

    pub fn active_player_count(&self) -> usize {
        self.players.values().filter(|p| p.is_connected).count()
    }

    pub fn total_registered_players(&self) -> usize {
        self.players.len()
    }

    pub fn broadcast(&self, message: &ServerMessage) {
        if let Some(ref h_tx) = self.host_tx {
            let _ = h_tx.send(message.clone());
        }

        for player in self.players.values() {
            if let Some(ref tx) = player.tx {
                let _ = tx.send(message.clone());
            }
        }
    }
}
