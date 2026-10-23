use crate::game::room::GameRoom;
use crate::game::types::*;
use crate::ws::messages::{LobbyPlayer, ServerMessage};
use std::time::Instant;
use tokio::sync::mpsc;
use uuid::Uuid;

impl GameRoom {
    pub fn register_host(&mut self, tx: mpsc::UnboundedSender<ServerMessage>) {
        self.last_activity = Instant::now();
        self.host_tx = Some(tx.clone());

        let _ = tx.send(ServerMessage::Connected {
            role: "host".to_string(),
            pin: self.pin.clone(),
            player_id: None,
            nickname: None,
            is_reconnect: false,
        });

        let _ = tx.send(ServerMessage::ThemeConfig {
            theme: self.get_theme(),
        });

        self.send_host_current_state();
    }

    pub fn register_player(
        &mut self,
        player_id_opt: Option<String>,
        nickname: String,
        tx: mpsc::UnboundedSender<ServerMessage>,
    ) -> Result<String, String> {
        self.last_activity = Instant::now();

        // Check if this is a reconnect
        if let Some(pid) = player_id_opt {
            if let Some(player) = self.players.get_mut(&pid) {
                player.is_connected = true;
                player.tx = Some(tx.clone());
                player.last_seen = Instant::now();

                let nick = player.nickname.clone();
                let _ = tx.send(ServerMessage::Connected {
                    role: "player".to_string(),
                    pin: self.pin.clone(),
                    player_id: Some(pid.clone()),
                    nickname: Some(nick),
                    is_reconnect: true,
                });

                let _ = tx.send(ServerMessage::ThemeConfig {
                    theme: self.get_theme(),
                });

                self.send_player_current_state(&pid);
                self.broadcast_lobby_update();
                return Ok(pid);
            }
        }

        // If not reconnect, check if game has already started
        match &self.state {
            RoomState::Lobby => {}
            _ => {
                return Err("Game has already started. New players cannot join right now.".to_string());
            }
        }

        let trimmed_name = nickname.trim();
        if trimmed_name.is_empty() {
            return Err("Nickname cannot be empty.".to_string());
        }
        if trimmed_name.len() > 20 {
            return Err("Nickname must be 20 characters or less.".to_string());
        }

        // Check duplicate nickname (case-insensitive)
        let name_lower = trimmed_name.to_lowercase();
        for p in self.players.values() {
            if p.nickname.to_lowercase() == name_lower && p.is_connected {
                return Err("Nickname is already taken. Please choose another name.".to_string());
            }
        }

        let new_id = Uuid::new_v4().to_string();
        let session = PlayerSession {
            id: new_id.clone(),
            nickname: trimmed_name.to_string(),
            score: 0,
            streak: 0,
            correct_count: 0,
            total_answered: 0,
            rank_history: Vec::new(),
            is_connected: true,
            last_seen: Instant::now(),
            tx: Some(tx.clone()),
        };

        self.players.insert(new_id.clone(), session);

        let _ = tx.send(ServerMessage::Connected {
            role: "player".to_string(),
            pin: self.pin.clone(),
            player_id: Some(new_id.clone()),
            nickname: Some(trimmed_name.to_string()),
            is_reconnect: false,
        });

        let _ = tx.send(ServerMessage::ThemeConfig {
            theme: self.get_theme(),
        });

        // Notify host and other players
        let player_count = self.active_player_count();
        if let Some(ref h_tx) = self.host_tx {
            let _ = h_tx.send(ServerMessage::PlayerJoined {
                player_id: new_id.clone(),
                nickname: trimmed_name.to_string(),
                player_count,
            });
        }

        self.broadcast_lobby_update();
        Ok(new_id)
    }

    pub fn disconnect_host(&mut self) {
        self.host_tx = None;
        self.last_activity = Instant::now();
    }

    pub fn disconnect_player(&mut self, player_id: &str) {
        let nickname_opt = if let Some(player) = self.players.get_mut(player_id) {
            player.is_connected = false;
            player.tx = None;
            player.last_seen = Instant::now();
            Some(player.nickname.clone())
        } else {
            None
        };

        if let Some(nickname) = nickname_opt {
            let p_count = self.active_player_count();
            if let Some(ref h_tx) = self.host_tx {
                let _ = h_tx.send(ServerMessage::PlayerLeft {
                    player_id: player_id.to_string(),
                    nickname,
                    player_count: p_count,
                });
            }

            self.broadcast_lobby_update();
        }
    }

    pub fn kick_player(&mut self, player_id: &str) -> bool {
        if let Some(player) = self.players.remove(player_id) {
            if let Some(tx) = player.tx {
                let _ = tx.send(ServerMessage::PlayerKicked {
                    reason: "You were removed from the room by the host.".to_string(),
                });
            }
            self.broadcast_lobby_update();
            true
        } else {
            false
        }
    }

    pub fn get_lobby_players(&self) -> Vec<LobbyPlayer> {
        self.players
            .values()
            .map(|p| LobbyPlayer {
                player_id: p.id.clone(),
                nickname: p.nickname.clone(),
                is_connected: p.is_connected,
            })
            .collect()
    }

    pub fn broadcast_lobby_update(&self) {
        if !matches!(self.state, RoomState::Lobby) {
            return;
        }

        let players = self.get_lobby_players();
        let count = players.iter().filter(|p| p.is_connected).count();

        let msg = ServerMessage::LobbyState {
            pin: self.pin.clone(),
            quiz_title: self.quiz.title.clone(),
            player_count: count,
            players,
        };

        self.broadcast(&msg);
    }
}
