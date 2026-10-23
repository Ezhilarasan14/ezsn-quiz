-- Seed initial demo quizzes for PostgreSQL

-- Quiz 1: Rust & Web Tech Trivia
INSERT INTO quizzes (id, title, description)
VALUES ('00000000-0000-0000-0000-000000000001', 'Rust & Modern Web Challenge', 'Test your knowledge on Rust programming, WebSockets, and modern full-stack web concepts!')
ON CONFLICT (id) DO NOTHING;

-- Questions for Quiz 1
-- Q1
INSERT INTO questions (id, quiz_id, question_text, time_limit_sec, points, order_index)
VALUES ('q1-01', '00000000-0000-0000-0000-000000000001', 'What mascot represents the Rust programming language?', 15, 1000, 0)
ON CONFLICT (id) DO NOTHING;

INSERT INTO options (id, question_id, option_text, is_correct, order_index)
VALUES 
('opt-1-1', 'q1-01', 'Ferris the Crab', true, 0),
('opt-1-2', 'q1-01', 'Gopher', false, 1),
('opt-1-3', 'q1-01', 'Tux the Penguin', false, 2),
('opt-1-4', 'q1-01', 'Duke the Java Mascot', false, 3)
ON CONFLICT (id) DO NOTHING;

-- Q2
INSERT INTO questions (id, quiz_id, question_text, time_limit_sec, points, order_index)
VALUES ('q1-02', '00000000-0000-0000-0000-000000000001', 'Which mechanism in Rust enforces memory safety without a garbage collector?', 20, 1000, 1)
ON CONFLICT (id) DO NOTHING;

INSERT INTO options (id, question_id, option_text, is_correct, order_index)
VALUES 
('opt-2-1', 'q1-02', 'Reference Counting only', false, 0),
('opt-2-2', 'q1-02', 'Ownership and Borrow Checker', true, 1),
('opt-2-3', 'q1-02', 'Generational Mark-and-Sweep', false, 2),
('opt-2-4', 'q1-02', 'Manual malloc and free', false, 3)
ON CONFLICT (id) DO NOTHING;

-- Q3
INSERT INTO questions (id, quiz_id, question_text, time_limit_sec, points, order_index)
VALUES ('q1-03', '00000000-0000-0000-0000-000000000001', 'What protocol header does an HTTP request use to upgrade to a WebSocket connection?', 15, 1000, 2)
ON CONFLICT (id) DO NOTHING;

INSERT INTO options (id, question_id, option_text, is_correct, order_index)
VALUES 
('opt-3-1', 'q1-03', 'Connection: Upgrade', true, 0),
('opt-3-2', 'q1-03', 'Protocol: Socket-Stream', false, 1),
('opt-3-3', 'q1-03', 'Transfer-Encoding: chunked', false, 2),
('opt-3-4', 'q1-03', 'Accept-Encoding: gzip', false, 3)
ON CONFLICT (id) DO NOTHING;

-- Q4
INSERT INTO questions (id, quiz_id, question_text, time_limit_sec, points, order_index)
VALUES ('q1-04', '00000000-0000-0000-0000-000000000001', 'Which Actix Web feature handles asynchronous background tasks in Rust?', 15, 1000, 3)
ON CONFLICT (id) DO NOTHING;

INSERT INTO options (id, question_id, option_text, is_correct, order_index)
VALUES 
('opt-4-1', 'q1-04', 'pthread_create wrapper', false, 0),
('opt-4-2', 'q1-04', 'Tokio runtime tasks', true, 1),
('opt-4-3', 'q1-04', 'Node.js libuv event loop', false, 2),
('opt-4-4', 'q1-04', 'Synchronous thread sleep', false, 3)
ON CONFLICT (id) DO NOTHING;

-- Quiz 2: General Tech & Computing Trivia
INSERT INTO quizzes (id, title, description)
VALUES ('00000000-0000-0000-0000-000000000002', 'General Computing Trivia', 'Fun history, protocols, and computer science fundamentals for all developers.')
ON CONFLICT (id) DO NOTHING;

-- Questions for Quiz 2
-- Q1
INSERT INTO questions (id, quiz_id, question_text, time_limit_sec, points, order_index)
VALUES ('q2-01', '00000000-0000-0000-0000-000000000002', 'Who is considered the world''s first computer programmer?', 20, 1000, 0)
ON CONFLICT (id) DO NOTHING;

INSERT INTO options (id, question_id, option_text, is_correct, order_index)
VALUES 
('opt-2-1-1', 'q2-01', 'Ada Lovelace', true, 0),
('opt-2-1-2', 'q2-01', 'Alan Turing', false, 1),
('opt-2-1-3', 'q2-01', 'Grace Hopper', false, 2),
('opt-2-1-4', 'q2-01', 'Charles Babbage', false, 3)
ON CONFLICT (id) DO NOTHING;

-- Q2
INSERT INTO questions (id, quiz_id, question_text, time_limit_sec, points, order_index)
VALUES ('q2-02', '00000000-0000-0000-0000-000000000002', 'What year was the World Wide Web introduced to the public by Tim Berners-Lee?', 20, 1000, 1)
ON CONFLICT (id) DO NOTHING;

INSERT INTO options (id, question_id, option_text, is_correct, order_index)
VALUES 
('opt-2-2-1', 'q2-02', '1989', false, 0),
('opt-2-2-2', 'q2-02', '1991', true, 1),
('opt-2-2-3', 'q2-02', '1995', false, 2),
('opt-2-2-4', 'q2-02', '1983', false, 3)
ON CONFLICT (id) DO NOTHING;

-- Q3
INSERT INTO questions (id, quiz_id, question_text, time_limit_sec, points, order_index)
VALUES ('q2-03', '00000000-0000-0000-0000-000000000002', 'Which port does standard HTTPS traffic use?', 15, 1000, 2)
ON CONFLICT (id) DO NOTHING;

INSERT INTO options (id, question_id, option_text, is_correct, order_index)
VALUES 
('opt-2-3-1', 'q2-03', '80', false, 0),
('opt-2-3-2', 'q2-03', '8080', false, 1),
('opt-2-3-3', 'q2-03', '443', true, 2),
('opt-2-3-4', 'q2-03', '8443', false, 3)
ON CONFLICT (id) DO NOTHING;
