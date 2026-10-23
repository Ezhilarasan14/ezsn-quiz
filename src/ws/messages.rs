use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::game::types::{LeaderboardEntry, OptionView, PodiumEntry, ThemeConfig};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LobbyPlayer {
    pub player_id: String,
    pub nickname: String,
    pub is_connected: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum ClientMessage {
    HostConnect {
        pin: String,
        token: String,
    },
    PlayerConnect {
        pin: String,
        nickname: String,
        player_id: Option<String>,
        #[serde(default)]
        email: Option<String>,
    },
    HostStartGame,
    HostNextQuestion,
    HostKickPlayer {
        player_id: String,
    },
    PlayerSubmitAnswer {
        question_id: String,
        option_id: String,
    },
    Ping,
}

impl<'de> Deserialize<'de> for ClientMessage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawMsg {
            #[serde(rename = "type")]
            msg_type: String,
            #[serde(default)]
            payload: serde_json::Value,
        }

        let raw = RawMsg::deserialize(deserializer)?;
        match raw.msg_type.as_str() {
            "host_connect" => {
                #[derive(Deserialize)]
                struct Payload {
                    pin: String,
                    token: String,
                }
                let p: Payload = serde_json::from_value(raw.payload).map_err(serde::de::Error::custom)?;
                Ok(ClientMessage::HostConnect {
                    pin: p.pin,
                    token: p.token,
                })
            }
            "player_connect" => {
                #[derive(Deserialize)]
                struct Payload {
                    pin: String,
                    nickname: String,
                    #[serde(default)]
                    player_id: Option<String>,
                    #[serde(default)]
                    email: Option<String>,
                }
                let p: Payload = serde_json::from_value(raw.payload).map_err(serde::de::Error::custom)?;
                Ok(ClientMessage::PlayerConnect {
                    pin: p.pin,
                    nickname: p.nickname,
                    player_id: p.player_id,
                    email: p.email,
                })
            }
            "host_start_game" => Ok(ClientMessage::HostStartGame),
            "host_next_question" => Ok(ClientMessage::HostNextQuestion),
            "host_kick_player" => {
                #[derive(Deserialize)]
                struct Payload {
                    player_id: String,
                }
                let p: Payload = serde_json::from_value(raw.payload).map_err(serde::de::Error::custom)?;
                Ok(ClientMessage::HostKickPlayer {
                    player_id: p.player_id,
                })
            }
            "player_submit_answer" => {
                #[derive(Deserialize)]
                struct Payload {
                    #[serde(default)]
                    question_id: String,
                    option_id: String,
                }
                let p: Payload = serde_json::from_value(raw.payload).map_err(serde::de::Error::custom)?;
                Ok(ClientMessage::PlayerSubmitAnswer {
                    question_id: p.question_id,
                    option_id: p.option_id,
                })
            }
            "ping" => Ok(ClientMessage::Ping),
            other => Err(serde::de::Error::custom(format!("Unknown client message type: {}", other))),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum ServerMessage {
    Connected {
        role: String, // "host" or "player"
        pin: String,
        player_id: Option<String>,
        nickname: Option<String>,
        is_reconnect: bool,
    },
    LobbyState {
        pin: String,
        quiz_title: String,
        player_count: usize,
        players: Vec<LobbyPlayer>,
    },
    PlayerJoined {
        player_id: String,
        nickname: String,
        player_count: usize,
    },
    PlayerLeft {
        player_id: String,
        nickname: String,
        player_count: usize,
    },
    PlayerKicked {
        reason: String,
    },
    GameStarting {
        countdown_secs: u64,
    },
    QuestionIntro {
        question_index: usize,
        total_questions: usize,
        question_text: String,
        countdown_secs: u64,
    },
    QuestionActive {
        question_index: usize,
        total_questions: usize,
        question_text: String,
        time_limit_sec: u64,
        options: Vec<OptionView>,
        already_answered: bool,
    },
    AnswerAccepted {
        question_index: usize,
    },
    AnswerRejected {
        reason: String,
    },
    HostAnswerCountUpdate {
        answered_count: usize,
        total_players: usize,
    },
    QuestionResult {
        question_index: usize,
        correct_option_ids: Vec<String>,
        option_counts: HashMap<String, u32>,
        player_correct: Option<bool>,
        points_awarded: Option<i64>,
        total_score: Option<i64>,
        streak: Option<u32>,
    },
    LeaderboardUpdate {
        rankings: Vec<LeaderboardEntry>,
        my_rank: Option<usize>,
        my_score: Option<i64>,
    },
    Podium {
        podium: Vec<PodiumEntry>,
        full_results: Vec<LeaderboardEntry>,
        my_rank: Option<usize>,
    },
    ThemeConfig {
        theme: ThemeConfig,
    },
    Error {
        message: String,
    },
    Pong,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_host_start_game_with_map_payload() {
        let json = r#"{"type":"host_start_game","payload":{}}"#;
        let msg: ClientMessage = serde_json::from_str(json).unwrap();
        assert_eq!(msg, ClientMessage::HostStartGame);
    }

    #[test]
    fn test_deserialize_host_start_game_without_payload() {
        let json = r#"{"type":"host_start_game"}"#;
        let msg: ClientMessage = serde_json::from_str(json).unwrap();
        assert_eq!(msg, ClientMessage::HostStartGame);
    }

    #[test]
    fn test_deserialize_host_next_question_with_payload() {
        let json = r#"{"type":"host_next_question","payload":{}}"#;
        let msg: ClientMessage = serde_json::from_str(json).unwrap();
        assert_eq!(msg, ClientMessage::HostNextQuestion);
    }

    #[test]
    fn test_deserialize_ping_with_payload() {
        let json = r#"{"type":"ping","payload":{}}"#;
        let msg: ClientMessage = serde_json::from_str(json).unwrap();
        assert_eq!(msg, ClientMessage::Ping);
    }

    #[test]
    fn test_deserialize_player_submit_answer() {
        let json = r#"{"type":"player_submit_answer","payload":{"question_id":"q1","option_id":"opt1"}}"#;
        let msg: ClientMessage = serde_json::from_str(json).unwrap();
        assert_eq!(
            msg,
            ClientMessage::PlayerSubmitAnswer {
                question_id: "q1".to_string(),
                option_id: "opt1".to_string(),
            }
        );
    }
}
