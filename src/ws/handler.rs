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
        let conn_id = uuid::Uuid::new_v4().to_string();
        let mut heartbeat = tokio::time::interval(Duration::from_secs(15));
        let mut last_client_msg = std::time::Instant::now();

        loop {
            tokio::select! {
                _ = heartbeat.tick() => {
                    // Check if client has gone completely silent (no frames/pongs for 60s)
                    if last_client_msg.elapsed() > Duration::from_secs(60) {
                        log::warn!("WebSocket client timed out (no message/pong for 60s). Closing connection {}", conn_id);
                        break;
                    }

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

                    last_client_msg = std::time::Instant::now();

                    match msg {
                        actix_ws::Message::Text(text) => {
                            let client_msg: Result<ClientMessage, _> = serde_json::from_str(&text);
                            match client_msg {
                                Ok(ClientMessage::HostConnect { pin, token }) => {
                                    if let Some(room) = handle_host_connect(&manager, &pin, &token, tx.clone(), Some(conn_id.clone())).await {
                                        is_host = true;
                                        current_room = Some(room);
                                    }
                                }
                                Ok(ClientMessage::PlayerConnect { pin, nickname, player_id: pid_opt, email }) => {
                                    if let Some((room, pid)) = handle_player_connect(&manager, &pin, &nickname, pid_opt, email, tx.clone(), Some(conn_id.clone())).await {
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
                                    if let Some(ref room_arc) = current_room {
                                        let mut r = room_arc.write().await;
                                        r.touch();
                                        if let Some(ref pid) = player_id {
                                            if let Some(p) = r.players.get_mut(pid) {
                                                p.last_seen = std::time::Instant::now();
                                            }
                                        }
                                    }
                                }
                                Err(err) => {
                                    log::warn!("Invalid client message payload: {}", err);
                                }
                            }
                        }
                        actix_ws::Message::Ping(bytes) => {
                            if session.pong(&bytes).await.is_err() { break; }
                        }
                        actix_ws::Message::Pong(_) => {
                            if let Some(ref room_arc) = current_room {
                                let mut r = room_arc.write().await;
                                r.touch();
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

        // Cleanup on disconnect: only disconnect if this connection is still active for this role/player
        if let Some(room_arc) = current_room {
            let mut room = room_arc.write().await;
            if is_host {
                room.disconnect_host_conn(Some(&conn_id));
                log::info!("Host disconnected from room {} (conn: {})", room.pin, conn_id);
            } else if let Some(ref pid) = player_id {
                room.disconnect_player_conn(pid, Some(&conn_id));
                log::info!("Player {} disconnected from room {} (conn: {})", pid, room.pin, conn_id);
            }
        }
    });

    Ok(response)
}
