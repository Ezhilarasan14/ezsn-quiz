use actix_web::{web, HttpRequest, HttpResponse};
use futures_util::StreamExt;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, RwLock};

use crate::db::DbPool;
use crate::game::manager::RoomManager;
use crate::game::room::GameRoom;
use crate::ws::messages::{ClientMessage, ServerMessage};

pub async fn ws_route(
    req: HttpRequest,
    body: web::Payload,
    manager: web::Data<Arc<RoomManager>>,
    db_pool: web::Data<DbPool>,
) -> Result<HttpResponse, actix_web::Error> {
    let (response, mut session, mut msg_stream) = actix_ws::handle(&req, body)?;

    actix_web::rt::spawn(async move {
        let (tx, mut rx) = mpsc::unbounded_channel::<ServerMessage>();

        let mut current_room: Option<Arc<RwLock<GameRoom>>> = None;
        let mut is_host = false;
        let mut player_id: Option<String> = None;

        let mut heartbeat = tokio::time::interval(Duration::from_secs(20));

        loop {
            tokio::select! {
                _ = heartbeat.tick() => {
                    if session.ping(b"heartbeat").await.is_err() {
                        break;
                    }
                }
                Some(srv_msg) = rx.recv() => {
                    if let Ok(json_str) = serde_json::to_string(&srv_msg) {
                        if session.text(json_str).await.is_err() {
                            break;
                        }
                    }
                }
                msg = msg_stream.next() => {
                    let msg = match msg {
                        Some(Ok(m)) => m,
                        _ => break,
                    };

                    match msg {
                        actix_ws::Message::Text(text) => {
                            let client_msg: Result<ClientMessage, _> = serde_json::from_str(&text);
                            match client_msg {
                                Ok(ClientMessage::HostConnect { pin, token }) => {
                                    if let Some(room_arc) = manager.get_room(&pin).await {
                                        let mut room = room_arc.write().await;
                                        if room.host_token == token {
                                            is_host = true;
                                            room.register_host(tx.clone());
                                            drop(room);
                                            current_room = Some(room_arc);
                                        } else {
                                            let _ = tx.send(ServerMessage::Error {
                                                message: "Unauthorized: invalid host token for this game.".to_string(),
                                            });
                                        }
                                    } else {
                                        let _ = tx.send(ServerMessage::Error {
                                            message: format!("Game room with PIN {} not found.", pin),
                                        });
                                    }
                                }
                                Ok(ClientMessage::PlayerConnect { pin, nickname, player_id: pid_opt }) => {
                                    if let Some(room_arc) = manager.get_room(&pin).await {
                                        let mut room = room_arc.write().await;
                                        match room.register_player(pid_opt, nickname, tx.clone()) {
                                            Ok(pid) => {
                                                player_id = Some(pid);
                                                is_host = false;
                                                drop(room);
                                                current_room = Some(room_arc);
                                            }
                                            Err(err_msg) => {
                                                let _ = tx.send(ServerMessage::Error {
                                                    message: err_msg,
                                                });
                                            }
                                        }
                                    } else {
                                        let _ = tx.send(ServerMessage::Error {
                                            message: format!("Game room with PIN {} not found.", pin),
                                        });
                                    }
                                }
                                Ok(ClientMessage::HostStartGame) => {
                                    if is_host {
                                        if let Some(ref room_arc) = current_room {
                                            GameRoom::start_game(room_arc.clone());
                                        }
                                    }
                                }
                                Ok(ClientMessage::HostNextQuestion) => {
                                    if is_host {
                                        if let Some(ref room_arc) = current_room {
                                            GameRoom::host_advance_step(room_arc.clone(), db_pool.get_ref().clone());
                                        }
                                    }
                                }
                                Ok(ClientMessage::HostKickPlayer { player_id: kick_pid }) => {
                                    if is_host {
                                        if let Some(ref room_arc) = current_room {
                                            let mut room = room_arc.write().await;
                                            room.kick_player(&kick_pid);
                                        }
                                    }
                                }
                                Ok(ClientMessage::PlayerSubmitAnswer { question_id, option_id }) => {
                                    if let Some(ref pid) = player_id {
                                        if let Some(ref room_arc) = current_room {
                                            if let Err(e) = GameRoom::submit_answer(room_arc.clone(), pid, &question_id, &option_id).await {
                                                let _ = tx.send(ServerMessage::AnswerRejected { reason: e });
                                            }
                                        }
                                    }
                                }
                                Ok(ClientMessage::Ping) => {
                                    let _ = tx.send(ServerMessage::Pong);
                                }
                                Err(err) => {
                                    log::warn!("Invalid client message payload: {}", err);
                                }
                            }
                        }
                        actix_ws::Message::Ping(bytes) => {
                            if session.pong(&bytes).await.is_err() {
                                break;
                            }
                        }
                        actix_ws::Message::Close(reason) => {
                            let _ = session.close(reason).await;
                            break;
                        }
                        _ => {}
                    }
                }
            }
        }

        // Cleanup on disconnect
        if let Some(room_arc) = current_room {
            let mut room = room_arc.write().await;
            if is_host {
                room.disconnect_host();
                log::info!("Host disconnected from room {}", room.pin);
            } else if let Some(ref pid) = player_id {
                room.disconnect_player(pid);
                log::info!("Player {} disconnected from room {}", pid, room.pin);
            }
        }
    });

    Ok(response)
}
