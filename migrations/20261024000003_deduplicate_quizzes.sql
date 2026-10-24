-- Deduplicate quizzes and clean up test runner artifacts

-- 1. Remove automated test artifact quizzes
DELETE FROM quizzes
WHERE title IN (
    'Concurrency Load Test Quiz',
    'Multiplayer Unit Test Quiz',
    'Review Flow Integration Quiz',
    'Test Rust Concurrency Quiz (Copy)',
    '5s Answer Time Test Quiz'
);

-- 2. Deduplicate any remaining quizzes by title, keeping the earliest created quiz
DELETE FROM quizzes
WHERE id NOT IN (
    SELECT DISTINCT ON (title) id
    FROM quizzes
    ORDER BY title, created_at ASC
);
