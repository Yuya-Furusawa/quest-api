CREATE TABLE quests
(
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    description TEXT,
    price INTEGER,
    difficulty TEXT,
    num_participate INTEGER,
    num_clear INTEGER
);
