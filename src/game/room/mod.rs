pub mod answers;
pub mod gameplay;
pub mod registration;
pub mod state_sync;

use crate::db::models::QuizDetail;
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
    pub state: RoomState,
    pub host_tx: Option<mpsc::UnboundedSender<ServerMessage>>,
    pub players: HashMap<String, PlayerSession>,
    pub created_at: Instant,
    pub last_activity: Instant,
    pub timer_token: u64,
}

impl GameRoom {
    pub fn new(pin: String, game_id: String, host_token: String, quiz: QuizDetail) -> Self {
        Self {
            pin,
            game_id,
            host_token,
            quiz,
            state: RoomState::Lobby,
            host_tx: None,
            players: HashMap::new(),
            created_at: Instant::now(),
            last_activity: Instant::now(),
            timer_token: 0,
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
