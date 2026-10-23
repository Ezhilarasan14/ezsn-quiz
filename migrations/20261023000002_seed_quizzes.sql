-- Seed initial demo quizzes

-- Quiz 1: Rust & Web Tech Trivia
INSERT OR IGNORE INTO quizzes (id, title, description)
VALUES ('00000000-0000-0000-0000-000000000001', 'Rust & Modern Web Challenge', 'Test your knowledge on Rust programming, WebSockets, and modern full-stack web concepts!');

-- Questions for Quiz 1
-- Q1
INSERT OR IGNORE INTO questions (id, quiz_id, question_text, time_limit_sec, points, order_index)
VALUES ('q1-01', '00000000-0000-0000-0000-000000000001', 'What mascot represents the Rust programming language?', 15, 1000, 0);

INSERT OR IGNORE INTO options (id, question_id, option_text, is_correct, order_index)
VALUES 
('opt-1-1', 'q1-01', 'Ferris the Crab', 1, 0),
('opt-1-2', 'q1-01', 'Gopher', 0, 1),
('opt-1-3', 'q1-01', 'Tux the Penguin', 0, 2),
('opt-1-4', 'q1-01', 'Duke the Java Mascot', 0, 3);

-- Q2
INSERT OR IGNORE INTO questions (id, quiz_id, question_text, time_limit_sec, points, order_index)
VALUES ('q1-02', '00000000-0000-0000-0000-000000000001', 'Which mechanism in Rust enforces memory safety without a garbage collector?', 20, 1000, 1);

INSERT OR IGNORE INTO options (id, question_id, option_text, is_correct, order_index)
VALUES 
('opt-2-1', 'q1-02', 'Reference Counting only', 0, 0),
('opt-2-2', 'q1-02', 'Ownership and Borrow Checker', 1, 1),
('opt-2-3', 'q1-02', 'Generational Mark-and-Sweep', 0, 2),
('opt-2-4', 'q1-02', 'Manual malloc and free', 0, 3);

-- Q3
INSERT OR IGNORE INTO questions (id, quiz_id, question_text, time_limit_sec, points, order_index)
VALUES ('q1-03', '00000000-0000-0000-0000-000000000001', 'What protocol header does an HTTP request use to upgrade to a WebSocket connection?', 15, 1000, 2);

INSERT OR IGNORE INTO options (id, question_id, option_text, is_correct, order_index)
VALUES 
('opt-3-1', 'q1-03', 'Connection: Upgrade', 1, 0),
('opt-3-2', 'q1-03', 'Protocol: Socket-Stream', 0, 1),
('opt-3-3', 'q1-03', 'Transfer-Encoding: chunked', 0, 2),
('opt-3-4', 'q1-03', 'Accept-Encoding: gzip', 0, 3);

-- Q4
INSERT OR IGNORE INTO questions (id, quiz_id, question_text, time_limit_sec, points, order_index)
VALUES ('q1-04', '00000000-0000-0000-0000-000000000001', 'Which Actix Web feature handles asynchronous background tasks in Rust?', 15, 1000, 3);

INSERT OR IGNORE INTO options (id, question_id, option_text, is_correct, order_index)
VALUES 
('opt-4-1', 'q1-04', 'pthread_create wrapper', 0, 0),
('opt-4-2', 'q1-04', 'Tokio runtime tasks', 1, 1),
('opt-4-3', 'q1-04', 'Node.js libuv event loop', 0, 2),
('opt-4-4', 'q1-04', 'Synchronous thread sleep', 0, 3);

-- Quiz 2: General Science & World Trivia
INSERT OR IGNORE INTO quizzes (id, title, description)
VALUES ('00000000-0000-0000-0000-000000000002', 'World & Science Wonders', 'Quick-fire fun trivia on planets, nature, and world history!');

-- Q1
INSERT OR IGNORE INTO questions (id, quiz_id, question_text, time_limit_sec, points, order_index)
VALUES ('q2-01', '00000000-0000-0000-0000-000000000002', 'Which planet is known as the "Red Planet"?', 15, 1000, 0);

INSERT OR IGNORE INTO options (id, question_id, option_text, is_correct, order_index)
VALUES 
('opt-21-1', 'q2-01', 'Venus', 0, 0),
('opt-21-2', 'q2-01', 'Jupiter', 0, 1),
('opt-21-3', 'q2-01', 'Mars', 1, 2),
('opt-21-4', 'q2-01', 'Saturn', 0, 3);

-- Q2
INSERT OR IGNORE INTO questions (id, quiz_id, question_text, time_limit_sec, points, order_index)
VALUES ('q2-02', '00000000-0000-0000-0000-000000000002', 'What is the chemical symbol for Gold?', 15, 1000, 1);

INSERT OR IGNORE INTO options (id, question_id, option_text, is_correct, order_index)
VALUES 
('opt-22-1', 'q2-02', 'Au', 1, 0),
('opt-22-2', 'q2-02', 'Ag', 0, 1),
('opt-22-3', 'q2-02', 'Fe', 0, 2),
('opt-22-4', 'q2-02', 'Gd', 0, 3);

-- Q3
INSERT OR IGNORE INTO questions (id, quiz_id, question_text, time_limit_sec, points, order_index)
VALUES ('q2-03', '00000000-0000-0000-0000-000000000002', 'How many minutes does sunlight take to reach Earth?', 15, 1000, 2);

INSERT OR IGNORE INTO options (id, question_id, option_text, is_correct, order_index)
VALUES 
('opt-23-1', 'q2-03', 'About 30 seconds', 0, 0),
('opt-23-2', 'q2-03', 'About 8 minutes', 1, 1),
('opt-23-3', 'q2-03', 'About 1 hour', 0, 2),
('opt-23-4', 'q2-03', 'Instantaneous', 0, 3);
