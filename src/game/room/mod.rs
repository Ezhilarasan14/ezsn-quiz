pub mod answers;
pub mod gameplay;
pub mod registration;
pub mod state_sync;

use crate::db::models::{Organization, QuizDetail};
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
    pub state: RoomState,
    pub host_tx: Option<mpsc::UnboundedSender<ServerMessage>>,
    pub players: HashMap<String, PlayerSession>,
    pub created_at: Instant,
    pub last_activity: Instant,
    pub timer_token: u64,
}

impl GameRoom {
    pub fn new(
        pin: String,
        game_id: String,
        host_token: String,
        quiz: QuizDetail,
        organization: Option<Organization>,
    ) -> Self {
        Self {
            pin,
            game_id,
            host_token,
            quiz,
            organization,
            state: RoomState::Lobby,
            host_tx: None,
            players: HashMap::new(),
            created_at: Instant::now(),
            last_activity: Instant::now(),
            timer_token: 0,
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

        ThemeConfig {
            logo_url,
            primary_color,
            secondary_color: self.quiz.theme_secondary_color.clone().filter(|s| !s.is_empty()).or_else(|| Some("#ec4899".to_string())),
            background_color: self.quiz.theme_bg_color.clone().filter(|s| !s.is_empty()).or_else(|| Some("#0f172a".to_string())),
            button_color: self.quiz.theme_btn_color.clone().filter(|s| !s.is_empty()).or_else(|| Some("#3b82f6".to_string())),
            org_name: Some(org_name),
        }
    }

    pub fn is_abandoned(&self) -> bool {
        let inactive_duration = self.last_activity.elapsed();
        if self.host_tx.is_none() && inactive_duration > Duration::from_secs(900) {
            return true;
        }
        if inactive_duration > Duration::from_secs(7200) {
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
