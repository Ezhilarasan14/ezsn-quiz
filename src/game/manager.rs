use crate::db::models::QuizDetail;
use crate::db::{self, DbPool};
use crate::game::room::GameRoom;
use rand::Rng;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use uuid::Uuid;

pub struct RoomManager {
    pub rooms: Arc<RwLock<HashMap<String, Arc<RwLock<GameRoom>>>>>,
    pub db_pool: DbPool,
}

impl RoomManager {
    pub fn new(db_pool: DbPool) -> Self {
        let manager = Self {
            rooms: Arc::new(RwLock::new(HashMap::new())),
            db_pool: db_pool.clone(),
        };

        // Start background cleaner task
        manager.start_cleanup_task();
        manager
    }

    pub async fn create_room(&self, quiz: QuizDetail) -> Result<(String, String, String), String> {
        let mut rooms = self.rooms.write().await;

        // Generate unique 6-digit PIN
        let mut rng = rand::thread_rng();
        let pin = loop {
            let candidate = format!("{:06}", rng.gen_range(100000..999999));
            if !rooms.contains_key(&candidate) {
                break candidate;
            }
        };

        let game_id = Uuid::new_v4().to_string();
        let host_token = Uuid::new_v4().to_string();

        // Record game in database
        if let Err(e) = db::create_game_record(&self.db_pool, &game_id, &quiz.id, &pin, &host_token).await {
            log::error!("Failed to persist game record to DB: {}", e);
            return Err("Failed to create game in database.".to_string());
        }

        let room = GameRoom::new(pin.clone(), game_id.clone(), host_token.clone(), quiz);
        let room_arc = Arc::new(RwLock::new(room));
        rooms.insert(pin.clone(), room_arc);

        log::info!("Created game room with PIN: {}, Game ID: {}", pin, game_id);
        Ok((pin, host_token, game_id))
    }

    pub async fn get_room(&self, pin: &str) -> Option<Arc<RwLock<GameRoom>>> {
        let rooms = self.rooms.read().await;
        rooms.get(pin).cloned()
    }

    pub async fn room_exists(&self, pin: &str) -> bool {
        let rooms = self.rooms.read().await;
        rooms.contains_key(pin)
    }

    pub async fn active_rooms_count(&self) -> usize {
        let rooms = self.rooms.read().await;
        rooms.len()
    }

    fn start_cleanup_task(&self) {
        let rooms_arc = self.rooms.clone();
        let db_pool = self.db_pool.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            loop {
                interval.tick().await;

                let mut to_remove = Vec::new();
                {
                    let rooms = rooms_arc.read().await;
                    for (pin, room_arc) in rooms.iter() {
                        let room = room_arc.read().await;
                        if room.is_abandoned() {
                            to_remove.push((pin.clone(), room.game_id.clone()));
                        }
                    }
                }

                if !to_remove.is_empty() {
                    let mut rooms = rooms_arc.write().await;
                    for (pin, game_id) in to_remove {
                        rooms.remove(&pin);
                        log::info!("Cleaned up abandoned room with PIN: {}", pin);
                        let _ = db::update_game_status(&db_pool, &game_id, "abandoned", 0).await;
                    }
                }
            }
        });
    }
}
