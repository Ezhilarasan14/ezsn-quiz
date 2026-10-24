-- 1. Extend admins table for Google OAuth and RBAC
ALTER TABLE admins ADD COLUMN IF NOT EXISTS email TEXT UNIQUE;
ALTER TABLE admins ADD COLUMN IF NOT EXISTS google_sub TEXT UNIQUE;
ALTER TABLE admins ADD COLUMN IF NOT EXISTS name TEXT;
ALTER TABLE admins ADD COLUMN IF NOT EXISTS avatar_url TEXT;
ALTER TABLE admins ADD COLUMN IF NOT EXISTS role TEXT NOT NULL DEFAULT 'admin';

-- Make password_hash and salt nullable so Google OAuth accounts don't need passwords
ALTER TABLE admins ALTER COLUMN password_hash DROP NOT NULL;
ALTER TABLE admins ALTER COLUMN salt DROP NOT NULL;

-- 2. Create admin_allowlist table for pre-approved Google accounts
CREATE TABLE IF NOT EXISTS admin_allowlist (
    email TEXT PRIMARY KEY,
    role TEXT NOT NULL DEFAULT 'admin',
    notes TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Seed default allowlisted administrator email
INSERT INTO admin_allowlist (email, role, notes)
VALUES ('admin@ezsnapps.in', 'superadmin', 'Default Administrator')
ON CONFLICT (email) DO NOTHING;

-- 3. Create oauth_states table for secure state validation and CSRF protection
CREATE TABLE IF NOT EXISTS oauth_states (
    state TEXT PRIMARY KEY,
    nonce TEXT NOT NULL,
    redirect_url TEXT,
    expires_at TIMESTAMP NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_oauth_states_expires ON oauth_states(expires_at);

-- 4. Enhance admin_sessions table with security and metadata columns
ALTER TABLE admin_sessions ADD COLUMN IF NOT EXISTS user_agent TEXT;
ALTER TABLE admin_sessions ADD COLUMN IF NOT EXISTS ip_address TEXT;
ALTER TABLE admin_sessions ADD COLUMN IF NOT EXISTS last_active_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP;
