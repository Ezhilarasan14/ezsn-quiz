-- Add email column to game_player_results for tracking user history across quizzes
ALTER TABLE game_player_results ADD COLUMN IF NOT EXISTS email TEXT;

CREATE INDEX IF NOT EXISTS idx_player_results_email ON game_player_results(email);
