use actix_web::{web, HttpRequest, HttpResponse};
use futures_util::StreamExt;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, RwLock};

use crate::db::DbPool;
use crate::game::manager::RoomManager;
use crate::game::room::GameRoom;
use crate::ws::actions::*;
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
                    if session.ping(b"heartbeat").await.is_err() { break; }
                }
                Some(srv_msg) = rx.recv() => {
                    if let Ok(json_str) = serde_json::to_string(&srv_msg) {
                        if session.text(json_str).await.is_err() { break; }
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
                                    if let Some(room) = handle_host_connect(&manager, &pin, &token, tx.clone()).await {
                                        is_host = true;
                                        current_room = Some(room);
                                    }
                                }
                                Ok(ClientMessage::PlayerConnect { pin, nickname, player_id: pid_opt, email }) => {
                                    if let Some((room, pid)) = handle_player_connect(&manager, &pin, &nickname, pid_opt, email, tx.clone()).await {
                                        is_host = false;
                                        player_id = Some(pid);
                                        current_room = Some(room);
                                    }
                                }
                                Ok(ClientMessage::HostStartGame) => {
                                    if is_host {
                                        if let Some(ref room) = current_room {
                                            GameRoom::start_game(room.clone());
                                        }
                                    }
                                }
                                Ok(ClientMessage::HostNextQuestion) => {
                                    if is_host {
                                        if let Some(ref room) = current_room {
                                            GameRoom::host_advance_step(room.clone(), db_pool.get_ref().clone());
                                        }
                                    }
                                }
                                Ok(ClientMessage::HostKickPlayer { player_id: kick_pid }) => {
                                    if is_host {
                                        if let Some(ref room) = current_room {
                                            let mut r = room.write().await;
                                            r.kick_player(&kick_pid);
                                        }
                                    }
                                }
                                Ok(ClientMessage::PlayerSubmitAnswer { question_id, option_id }) => {
                                    handle_player_answer(
                                        current_room.as_ref(),
                                        player_id.as_deref(),
                                        &question_id,
                                        &option_id,
                                        &tx,
                                    ).await;
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
                            if session.pong(&bytes).await.is_err() { break; }
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
