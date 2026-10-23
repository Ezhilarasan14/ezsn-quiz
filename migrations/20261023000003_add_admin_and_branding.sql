-- Organizations table
CREATE TABLE IF NOT EXISTS organizations (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    logo_url TEXT,
    primary_color TEXT NOT NULL DEFAULT '#6366f1',
    secondary_color TEXT NOT NULL DEFAULT '#ec4899',
    background_color TEXT NOT NULL DEFAULT '#0f172a',
    button_color TEXT NOT NULL DEFAULT '#3b82f6',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Seed default organization
INSERT OR IGNORE INTO organizations (id, name, logo_url, primary_color, secondary_color, background_color, button_color)
VALUES ('org_default', 'EZSN Quiz Platform', '', '#6366f1', '#ec4899', '#0f172a', '#3b82f6');

-- Admins table
CREATE TABLE IF NOT EXISTS admins (
    id TEXT PRIMARY KEY,
    organization_id TEXT REFERENCES organizations(id) ON DELETE SET NULL,
    username TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    salt TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Admin sessions table
CREATE TABLE IF NOT EXISTS admin_sessions (
    token TEXT PRIMARY KEY,
    admin_id TEXT NOT NULL REFERENCES admins(id) ON DELETE CASCADE,
    expires_at TIMESTAMP NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_admin_sessions_token ON admin_sessions(token);
CREATE INDEX IF NOT EXISTS idx_admin_sessions_expires ON admin_sessions(expires_at);

-- Add branding and organization columns to quizzes
ALTER TABLE quizzes ADD COLUMN organization_id TEXT REFERENCES organizations(id) ON DELETE SET NULL;
ALTER TABLE quizzes ADD COLUMN theme_logo_url TEXT;
ALTER TABLE quizzes ADD COLUMN theme_primary_color TEXT;
ALTER TABLE quizzes ADD COLUMN theme_secondary_color TEXT;
ALTER TABLE quizzes ADD COLUMN theme_bg_color TEXT;
ALTER TABLE quizzes ADD COLUMN theme_btn_color TEXT;

UPDATE quizzes SET organization_id = 'org_default' WHERE organization_id IS NULL;
