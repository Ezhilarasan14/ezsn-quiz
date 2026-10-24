-- Add branch_name to organizations
ALTER TABLE organizations ADD COLUMN IF NOT EXISTS branch_name text DEFAULT 'Main Campus';
UPDATE organizations SET branch_name = 'Main Campus' WHERE branch_name IS NULL;

-- Add branch_name and host_name to quizzes for dynamic theme overriding
ALTER TABLE quizzes ADD COLUMN IF NOT EXISTS branch_name text;
ALTER TABLE quizzes ADD COLUMN IF NOT EXISTS host_name text;
