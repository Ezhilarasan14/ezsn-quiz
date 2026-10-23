# EZSN Quiz - Real-Time Multiplayer Quiz Platform

A high-performance, Kahoot-inspired real-time multiplayer quiz web application written in **Rust**, with embedded frontend assets, **SQLite** persistence, and **WebSockets**.

The entire application runs as a **single standalone binary** with zero external runtime dependencies (no Node.js or frontend web server required).

---

## 🚀 Key Features

### 1. Quiz Management
- **Create, Edit, Delete, Duplicate Quizzes**: Intuitive web-based quiz editor with validation.
- **Multiple Choice Questions**: Add multiple-choice questions with 2–4 colorful answer options.
- **Configurable Timers & Points**: 5s, 10s, 15s, 20s, 30s, 60s timers; 500, 1000, 2000 base points.
- **Live Preview Modal**: Test & preview questions and options before hosting.
- **Pre-seeded Quizzes**: Comes pre-populated with ready-to-play Rust/Web and Science trivia.

### 2. Multiplayer Game Rooms
- **6-Digit PIN Generation**: Collision-free unique PINs for rooms.
- **Frictionless Player Onboarding**: Join instantly with just a PIN and nickname (no registration required).
- **Concurrent Game Sessions**: Supports multiple hosts running independent quizzes concurrently.
- **Live Player Lobby**: Real-time player avatars with host kick capabilities.
- **Reconnection Handling**: Disconnected players automatically resume with their scores and rank restored using session tokens.
- **Automated Room Sweeper**: Background task automatically cleans up inactive or abandoned rooms.

### 3. Server-Authoritative Live Gameplay
- **Kahoot Game Lifecycle**:
  `Lobby` &rarr; `Question Intro (3s countdown)` &rarr; `Active Question` &rarr; `Answer Reveal (Stats Chart)` &rarr; `Leaderboard` &rarr; `Next Question` &rarr; `Final Podium`.
- **Server-Authoritative Scoring Formula**:
  - Awards points based on speed and correctness:
    $$\text{points} = \text{round}\left(\text{max\_points} \times \left(1.0 - \frac{\text{elapsed\_ms}}{\text{duration\_ms}} \times 0.5\right)\right) + \text{streak\_bonus}$$
  - Streak bonuses (+100 for 2-streak, up to +500 for hot streaks).
  - Duplicate and late answers are strictly rejected by the server.
- **Early Question Completion**: Automatically advances when all connected players have submitted an answer.

### 4. Interactive Live Leaderboards & Final Podium
- **Rank Change Indicators**: Displays rank transitions ($+1$, $-1$, streak badges).
- **Personalized Player Controller View**: Players see their exact rank and score updates directly on their phones.
- **Podium Celebration**: Olympic-style 1st, 2nd, 3rd place animated pillars with medal icons, fanfare audio, and confetti particles.
- **Persistent Game History**: Saves completed games and player scoreboards to SQLite.

### 5. Vibrant, Responsive UI & Web Audio
- **Vibrant Kahoot Palette**: Red ($\blacktriangle$), Blue ($\blacklozenge$), Yellow ($\CIRCLE$), Green ($\blacksquare$).
- **Optimized Mobile Controller**: Big touch targets with tactile feedback and responsive layouts.
- **Synthesized Web Audio**: In-browser Web Audio API sound generator for countdown ticks, answer clicks, correct/wrong chimes, and victory fanfares (with mute toggle).
- **Canvas Confetti Particle System**: Lightweight, self-contained confetti engine.

---

## 🛠️ Technology Stack

| Layer | Technology |
|---|---|
| **Backend** | Rust 1.92+, Actix Web 4, Tokio async runtime |
| **Real-time** | WebSockets (`actix-ws`) with async unbounded channels |
| **Database** | SQLite + SQLx (with WAL mode, migrations, foreign keys) |
| **Frontend** | HTML5, Tailwind CSS, Vanilla JavaScript, Web Audio API |
| **Asset Embedding** | `rust-embed` (embedded directly into the compiled binary) |
| **Serialization** | Serde and `serde_json` |

---

## 📦 Directory Structure

```text
ezsn-quiz/
├── Cargo.toml                  # Rust dependencies and package configuration
├── migrations/                 # SQLx SQLite migrations
│   ├── 20261023000001_create_tables.sql
│   └── 20261023000002_seed_quizzes.sql
├── src/
│   ├── main.rs                 # Server entrypoint and routing
│   ├── lib.rs                  # Module re-exports for integration tests
│   ├── config.rs               # Environment and server configuration
│   ├── static_files.rs         # rust-embed static asset handler
│   ├── db/
│   │   ├── mod.rs              # SQLite connection pool and CRUD queries
│   │   └── models.rs           # Database models and DTOs
│   ├── game/
│   │   ├── mod.rs              # Game module exports
│   │   ├── manager.rs          # Thread-safe RoomManager with background sweeper
│   │   ├── room.rs             # GameRoom state machine and event broadcasts
│   │   ├── scoring.rs          # Server-side speed & streak scoring engine
│   │   └── types.rs            # Game states, player sessions, leaderboard models
│   ├── ws/
│   │   ├── mod.rs              # WebSocket router
│   │   ├── handler.rs          # Actix-ws async stream handler
│   │   └── messages.rs         # Strongly typed ClientMessage & ServerMessage
│   └── api/
│       ├── mod.rs              # REST API route setup
│       ├── quizzes.rs          # Quizzes CRUD API endpoints
│       ├── rooms.rs            # Room creation and verification endpoints
│       └── history.rs          # Game history and standings API
├── static/                     # Embedded frontend assets
│   ├── index.html              # Home landing & player join
│   ├── host.html               # Big screen presenter view
│   ├── play.html               # Mobile controller player screen
│   ├── edit.html               # Quiz creator & editor
│   ├── history.html            # Past completed games & leaderboards
│   ├── css/styles.css          # Theme styles, button shapes, podium pillars
│   └── js/
│       ├── audio.js            # Web Audio API sound synthesizer
│       ├── confetti.js         # Canvas confetti animation
│       ├── common.js           # Toast alerts, API helpers, WebSocket wrapper
│       ├── home.js             # Landing page interaction
│       ├── host.js             # Presenter screen controller
│       ├── play.js             # Player controller logic
│       ├── editor.js           # Quiz editor interactive form
│       └── history.js          # Standings and history modal
└── tests/                      # Automated test suite
    ├── scoring_test.rs         # Scoring formula & streak tests
    ├── db_test.rs              # Database CRUD & migrations tests
    ├── room_lifecycle_test.rs  # Multiplayer room transitions & reconnection tests
    └── load_test.rs            # 100 concurrent players load test
```

---

## 🏃 Build & Run Instructions

### Prerequisites
- [Rust toolchain](https://www.rust-lang.org/tools/install) (1.75+)

### 1. Build and Run in Development
```bash
cargo run
```
The server will start at `http://localhost:8081`.

### 2. Run Tests
```bash
cargo test
```
Runs the unit tests, database migration tests, multiplayer lifecycle tests, and concurrency load tests.

### 3. Build Optimized Standalone Binary
```bash
cargo build --release
```
The optimized single binary will be generated at `target/release/ezsn-quiz` (~10MB) containing all embedded frontend assets and migrations.

Run it anywhere:
```bash
./target/release/ezsn-quiz
```

---

## ⚙️ Configuration

Environment variables can be set directly or via a `.env` file:

| Variable | Default | Description |
|---|---|---|
| `PORT` | `8081` | Port for the HTTP and WebSocket server |
| `HOST` | `0.0.0.0` | Bind host address |
| `DATABASE_URL` | `sqlite:quiz.db` | SQLite database connection string |
| `RUST_LOG` | `info` | Logging verbosity (`info`, `debug`, `trace`) |

---

## 🌐 REST API Reference

### Quizzes
- `GET /api/quizzes`: List all available quizzes with question counts.
- `GET /api/quizzes/{id}`: Get full quiz details with questions and options.
- `POST /api/quizzes`: Create a new quiz with questions and answer options.
- `PUT /api/quizzes/{id}`: Update an existing quiz.
- `DELETE /api/quizzes/{id}`: Delete a quiz and its questions.
- `POST /api/quizzes/{id}/duplicate`: Duplicate an existing quiz.

### Rooms
- `POST /api/rooms`: Host creates a new room for a quiz:
  ```json
  // Request
  { "quiz_id": "00000000-0000-0000-0000-000000000001" }

  // Response
  {
    "pin": "489201",
    "host_token": "a1b2c3d4-...",
    "game_id": "e5f6g7h8-...",
    "quiz_title": "Rust & Modern Web Challenge"
  }
  ```
- `GET /api/rooms/{pin}`: Check status and player count for a given room PIN.

### Game History
- `GET /api/history`: List past completed multiplayer sessions with winners.
- `GET /api/history/{game_id}`: Retrieve final player scores and rankings.

---

## ⚡ WebSocket Protocol

WebSocket endpoint: `/ws`

### Client &rarr; Server Messages (`ClientMessage`)
```json
// Host connects with PIN and secret token
{ "type": "host_connect", "payload": { "pin": "489201", "token": "a1b2c3d4-..." } }

// Player joins or reconnects
{ "type": "player_connect", "payload": { "pin": "489201", "nickname": "Ferris", "player_id": null } }

// Host triggers game start
{ "type": "host_start_game" }

// Host advances to next step (Skip / Leaderboard / Next Question)
{ "type": "host_next_question" }

// Host removes a player from the lobby
{ "type": "host_kick_player", "payload": { "player_id": "uuid-..." } }

// Player submits answer
{ "type": "player_submit_answer", "payload": { "question_id": "", "option_id": "opt-1-1" } }

// Ping keep-alive
{ "type": "ping" }
```

### Server &rarr; Client Messages (`ServerMessage`)
```json
// Confirms connection
{ "type": "connected", "payload": { "role": "player", "pin": "489201", "player_id": "uuid", "is_reconnect": false } }

// Lobby update broadcast
{ "type": "lobby_state", "payload": { "pin": "489201", "quiz_title": "...", "player_count": 4, "players": [...] } }

// Active question broadcast
{
  "type": "question_active",
  "payload": {
    "question_index": 1,
    "total_questions": 5,
    "question_text": "What mascot represents Rust?",
    "time_limit_sec": 15,
    "options": [
      { "id": "opt-1-1", "text": "Ferris the Crab", "color_idx": 0, "shape": "triangle" },
      { "id": "opt-1-2", "text": "Gopher", "color_idx": 1, "shape": "diamond" }
    ],
    "already_answered": false
  }
}

// Question result with personalized player points
{
  "type": "question_result",
  "payload": {
    "question_index": 1,
    "correct_option_ids": ["opt-1-1"],
    "option_counts": { "opt-1-1": 3, "opt-1-2": 1 },
    "player_correct": true,
    "points_awarded": 920,
    "total_score": 920,
    "streak": 1
  }
}

// Leaderboard update broadcast
{
  "type": "leaderboard_update",
  "payload": {
    "rankings": [
      { "player_id": "...", "nickname": "Ferris", "score": 920, "rank": 1, "rank_change": 0, "streak": 1, "is_connected": true }
    ],
    "my_rank": 1,
    "my_score": 920
  }
}

// Final podium celebration
{
  "type": "podium",
  "payload": {
    "podium": [
      { "player_id": "...", "nickname": "Ferris", "score": 4500, "rank": 1, "correct_count": 5, "total_answered": 5 }
    ],
    "full_results": [...],
    "my_rank": 1
  }
}
```

---

## 📈 Concurrency & Load Testing Plan

The platform is designed to effortlessly handle dozens of simultaneous game rooms with hundreds of concurrent players.

1. **Automated Concurrency Test**: Run `tests/load_test.rs` to verify that 100 concurrent players can join, answer questions asynchronously, and receive accurate rankings without thread blocking or state corruption:
   ```bash
   cargo test --test load_test
   ```
2. **Resource Efficiency**: Active game sessions are managed with thread-safe `Arc<RwLock<GameRoom>>` structures in memory.
3. **Database Scalability**: SQLite runs with Write-Ahead Logging (`WAL`) mode and non-blocking asynchronous queries via `sqlx`.

---

## 🚢 Production Deployment

### Docker Deployment
```dockerfile
FROM rust:1.92-slim as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app
COPY --from=builder /app/target/release/ezsn-quiz /app/ezsn-quiz
EXPOSE 8081
ENV PORT=8081
ENV HOST=0.0.0.0
CMD ["/app/ezsn-quiz"]
```

Build and run container:
```bash
docker build -t ezsn-quiz .
docker run -p 8081:8081 ezsn-quiz
```

---

## 📄 License
MIT License.
