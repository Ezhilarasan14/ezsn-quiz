use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

use crate::game::manager::RoomManager;
use crate::game::room::GameRoom;
use crate::ws::messages::ServerMessage;

pub async fn handle_host_connect(
    manager: &RoomManager,
    pin: &str,
    token: &str,
    tx: mpsc::UnboundedSender<ServerMessage>,
) -> Option<Arc<RwLock<GameRoom>>> {
    if let Some(room_arc) = manager.get_room(pin).await {
        let mut room = room_arc.write().await;
        if room.host_token == token {
            room.register_host(tx);
            drop(room);
            Some(room_arc)
        } else {
            let _ = tx.send(ServerMessage::Error {
                message: "Unauthorized: invalid host token for this game.".to_string(),
            });
            None
        }
    } else {
        let _ = tx.send(ServerMessage::Error {
            message: format!("Game room with PIN {} not found.", pin),
        });
        None
    }
}

pub async fn handle_player_connect(
    manager: &RoomManager,
    pin: &str,
    nickname: &str,
    pid_opt: Option<String>,
    email: Option<String>,
    tx: mpsc::UnboundedSender<ServerMessage>,
) -> Option<(Arc<RwLock<GameRoom>>, String)> {
    if let Some(room_arc) = manager.get_room(pin).await {
        let mut room = room_arc.write().await;
        match room.register_player(pid_opt, nickname.to_string(), email, tx.clone()) {
            Ok(pid) => {
                drop(room);
                Some((room_arc, pid))
            }
            Err(err_msg) => {
                let _ = tx.send(ServerMessage::Error { message: err_msg });
                None
            }
        }
    } else {
        let _ = tx.send(ServerMessage::Error {
            message: format!("Game room with PIN {} not found.", pin),
        });
        None
    }
}

pub async fn handle_player_answer(
    room_arc: Option<&Arc<RwLock<GameRoom>>>,
    player_id: Option<&str>,
    question_id: &str,
    option_id: &str,
    tx: &mpsc::UnboundedSender<ServerMessage>,
) {
    if let (Some(room), Some(pid)) = (room_arc, player_id) {
        if let Err(e) = GameRoom::submit_answer(room.clone(), pid, question_id, option_id).await {
            let _ = tx.send(ServerMessage::AnswerRejected { reason: e });
        }
    }
}
