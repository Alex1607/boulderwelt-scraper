CREATE TABLE IF NOT EXISTS crowd_levels (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    percentage TEXT NOT NULL,
    description TEXT NOT NULL,
    website_url TEXT NOT NULL,
    website_name TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
); 

-- Time-based averages table for storing crowd levels by day and hour
CREATE TABLE IF NOT EXISTS time_averages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    website_url TEXT NOT NULL,
    website_name TEXT NOT NULL,
    day_of_week INTEGER NOT NULL,
    hour INTEGER NOT NULL,
    average_percentage REAL NOT NULL,
    sample_count INTEGER NOT NULL,
    last_updated TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(website_url, day_of_week, hour)
);

-- Create indexes for the time_averages table
CREATE INDEX IF NOT EXISTS idx_time_averages_website_url ON time_averages(website_url);
CREATE INDEX IF NOT EXISTS idx_time_averages_day_hour ON time_averages(day_of_week, hour); 