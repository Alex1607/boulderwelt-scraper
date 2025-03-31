CREATE TABLE IF NOT EXISTS crowd_levels (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    percentage TEXT NOT NULL,
    description TEXT NOT NULL,
    website_url TEXT NOT NULL,
    website_name TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
); 