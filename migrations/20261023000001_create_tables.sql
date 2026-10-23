-- PostgreSQL Schema for quizzes, questions, options, games, and results

CREATE TABLE IF NOT EXISTS quizzes (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    description TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS questions (
    id TEXT PRIMARY KEY,
    quiz_id TEXT NOT NULL REFERENCES quizzes(id) ON DELETE CASCADE,
    question_text TEXT NOT NULL,
    time_limit_sec BIGINT NOT NULL DEFAULT 20,
    points BIGINT NOT NULL DEFAULT 1000,
    order_index BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_questions_quiz_id ON questions(quiz_id);

CREATE TABLE IF NOT EXISTS options (
    id TEXT PRIMARY KEY,
    question_id TEXT NOT NULL REFERENCES questions(id) ON DELETE CASCADE,
    option_text TEXT NOT NULL,
    is_correct BOOLEAN NOT NULL DEFAULT false,
    order_index BIGINT NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_options_question_id ON options(question_id);

CREATE TABLE IF NOT EXISTS games (
    id TEXT PRIMARY KEY,
    quiz_id TEXT NOT NULL REFERENCES quizzes(id) ON DELETE CASCADE,
    room_pin TEXT NOT NULL,
    host_token TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'lobby', -- 'lobby', 'in_progress', 'completed', 'abandoned'
    total_players BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    ended_at TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_games_room_pin ON games(room_pin);
CREATE INDEX IF NOT EXISTS idx_games_status ON games(status);

CREATE TABLE IF NOT EXISTS game_player_results (
    id TEXT PRIMARY KEY,
    game_id TEXT NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    player_id TEXT NOT NULL,
    nickname TEXT NOT NULL,
    total_score BIGINT NOT NULL DEFAULT 0,
    final_rank BIGINT NOT NULL DEFAULT 1,
    correct_count BIGINT NOT NULL DEFAULT 0,
    total_answered BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_player_results_game ON game_player_results(game_id);

CREATE TABLE IF NOT EXISTS game_answers (
    id TEXT PRIMARY KEY,
    game_id TEXT NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    question_id TEXT NOT NULL REFERENCES questions(id) ON DELETE CASCADE,
    player_id TEXT NOT NULL,
    option_id TEXT NOT NULL,
    is_correct BOOLEAN NOT NULL,
    points_awarded BIGINT NOT NULL DEFAULT 0,
    response_time_ms BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_game_answers_game_question ON game_answers(game_id, question_id);
