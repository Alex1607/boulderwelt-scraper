# Boulderwelt Crowd Level Scraper

A Cloudflare Worker written in Rust that scrapes bouldering websites every 10 minutes, extracts the margin-left percentage of a pointer.png element (which represents the current crowd level at the climbing gym), and stores it in a Cloudflare D1 database for historical tracking.

## Features

- Runs every 10 minutes as a scheduled Cloudflare Worker
- Supports multiple bouldering websites with the same pointer.png pattern
- Fetches HTML from configured websites
- Identifies the pointer element and extracts its margin-left percentage 
- Calculates crowd level based on the percentage
- Stores data in a Cloudflare D1 database
- Provides API endpoints for real-time and historical data
- Logs results to the Cloudflare console in a structured format

## Endpoints

- **/** - Simple health check
- **/scrape** - Manually trigger a scrape operation and get results
  - Add `?save=true` to store the result in the database
  - Add `?url=https://example.com` to scrape a specific website from the configured list
- **/history** - Retrieve historical crowd level data with timestamp-based pagination
  - Query parameter `since`: Unix timestamp to retrieve data older than (before) the specified time
  - Add `?url=https://example.com` to filter results for a specific website
- **/history/latest** - Get the most recent crowd level data from the database
  - Add `?url=https://example.com` to get the latest data for a specific website
- **/websites** - List all configured websites that can be scraped

## JSON Response

```json
{
  "timestamp": "2025-03-28T17:00:00.000Z",
  "url": "https://www.boulderwelt-muenchen-ost.de/",
  "pointer_margin_left_percentage": "38",
  "crowd_level_percentage": "38",
  "crowd_level_description": "Low",
  "location": "Boulderwelt München Ost",
  "website_url": "https://www.boulderwelt-muenchen-ost.de/",
  "details": {
    "raw_percentage": 38.0,
    "scrape_time_ms": "123.45"
  }
}
```

## Database Setup

The application uses Cloudflare D1 as its database. You need to manually set up the database schema using the following SQL:

```sql
CREATE TABLE IF NOT EXISTS crowd_levels (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TEXT NOT NULL,
    percentage TEXT NOT NULL,
    description TEXT NOT NULL,
    website_url TEXT NOT NULL,
    website_name TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes for performance optimization
CREATE INDEX IF NOT EXISTS idx_crowd_levels_website_url_created_at ON crowd_levels(website_url, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_created_at_desc ON crowd_levels(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_website_url_desc ON crowd_levels(website_url DESC);

-- Daily averages table for storing crowd levels by day of week
CREATE TABLE IF NOT EXISTS daily_averages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    website_url TEXT NOT NULL,
    website_name TEXT NOT NULL,
    day_of_week INTEGER NOT NULL, -- 0 = Sunday, 1 = Monday, etc.
    average_percentage REAL NOT NULL,
    sample_count INTEGER NOT NULL,
    last_updated TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(website_url, day_of_week)
);

-- Create indexes for the daily_averages table
CREATE INDEX IF NOT EXISTS idx_daily_averages_website_url ON daily_averages(website_url);
CREATE INDEX IF NOT EXISTS idx_daily_averages_day_of_week ON daily_averages(day_of_week);
```

To set up the database:

1. Create a D1 database in your Cloudflare account:
   ```bash
   wrangler d1 create boulderwelt_crowd_levels
   ```

2. Copy the database ID from the output and update it in your wrangler.toml file:
   ```toml
   [[d1_databases]]
   binding = "DB"
   database_name = "boulderwelt_crowd_levels"
   database_id = "YOUR_DATABASE_ID"
   ```

3. Execute the SQL to create the table schema:
   ```bash
   wrangler d1 execute boulderwelt_crowd_levels --file=./schema.sql
   ```
   
   Where schema.sql contains the CREATE TABLE statement above.

### Database Indexes

The schema includes the following carefully targeted indexes based on the application's query patterns:

1. `idx_crowd_levels_website_url_timestamp` - A composite index that optimizes the two most common query patterns:
   - Filtering by website_url and ordering by timestamp DESC (used in both `/history` and `/history/latest` endpoints)
   - This index serves both website-specific historical queries and latest-record lookups

2. `idx_timestamp_desc` - Optimizes queries that fetch records ordered by timestamp regardless of website:
   - Used when retrieving historical data across all websites 
   - Supports the global `/history` and `/history/latest` endpoints

These indexes were chosen by analyzing the actual SQL queries in the application code:
- `SELECT * FROM crowd_levels WHERE website_url = ? AND strftime('%s', timestamp) > ? ORDER BY timestamp DESC`
- `SELECT * FROM crowd_levels WHERE strftime('%s', timestamp) > ? ORDER BY timestamp DESC`
- `SELECT * FROM crowd_levels WHERE website_url = ? ORDER BY timestamp DESC`
- `SELECT * FROM crowd_levels ORDER BY timestamp DESC`
- `SELECT * FROM crowd_levels WHERE website_url = ? ORDER BY timestamp DESC LIMIT 1`
- `SELECT * FROM crowd_levels ORDER BY timestamp DESC LIMIT 1`

This minimal set of indexes provides optimal performance while avoiding redundant indexes that would increase storage requirements and slow down write operations.

## How the Pointer Works

The websites use an image element with a margin-left CSS property to indicate the current crowd level:

```html
<img width="21" height="15" src="https://www.boulderwelt-muenchen-ost.de/wp-content/plugins/cxo-crowd-level//resources/img/pointer.png" style="margin-left: 38%;" data-lazy-src="https://www.boulderwelt-muenchen-ost.de/wp-content/plugins/cxo-crowd-level//resources/img/pointer.png" data-ll-status="loaded" class="entered lazyloaded">
```

The `margin-left` percentage value represents how busy the gym is:
- 0-20%: Very low
- 20-40%: Low
- 40-60%: Moderate
- 60-80%: High
- 80-100%: Very high

## Adding New Websites

To add new websites that use the same pointer.png pattern, edit the `get_configured_websites` function in `src/scraper/mod.rs`:

```rust
pub fn get_configured_websites() -> Vec<WebsiteConfig> {
    vec![
        WebsiteConfig {
            url: "https://www.boulderwelt-muenchen-ost.de/".to_string(),
            name: "Boulderwelt München Ost".to_string(),
        },
        WebsiteConfig {
            url: "https://www.your-new-website.com/".to_string(),
            name: "Your New Boulder Gym".to_string(),
        },
        // Add more websites here
    ]
}
```

## Development

### Prerequisites

- [Wrangler CLI](https://developers.cloudflare.com/workers/wrangler/install-and-update/) (`npm install -g wrangler`)
- [Rust](https://www.rust-lang.org/tools/install)
- [Node.js](https://nodejs.org/)
- Cloudflare account with Workers and D1 access

### Setup

1. Create a D1 database in your Cloudflare account:
   ```bash
   wrangler d1 create boulderwelt_crowd_levels
   ```

2. Copy the database ID from the output and update it in your wrangler.toml file:
   ```toml
   [[d1_databases]]
   binding = "DB"
   database_name = "boulderwelt_crowd_levels"
   database_id = "YOUR_DATABASE_ID"
   ```

3. Deploy the worker:
   ```bash
   wrangler deploy
   ```

4. Apply the database schema:
   ```bash
   wrangler d1 execute boulderwelt_crowd_levels --file=./schema.sql
   ```

### Local Development

To run the Worker locally with a local D1 database:

```bash
wrangler dev --local
```

To run the Worker locally but use the production database (for testing with real data):

```bash
wrangler dev --env dev
```

This configuration is defined in the `wrangler.toml` file, where the development environment is set to use the same database as production.

### Deployment

To deploy to Cloudflare Workers:

1. Login to Cloudflare:
   ```bash
   wrangler login
   ```

2. Deploy:
   ```bash
   wrangler deploy
   ```

## How it Works

1. The worker is triggered every 10 minutes using Cloudflare's CRON triggers
2. It fetches the HTML from all configured bouldering websites
3. Using specific CSS selectors and regex patterns, it locates the pointer.png element on each site
4. When found, it extracts the margin-left percentage value
5. It categorizes the crowd level based on the percentage value
6. It stores the data in the D1 database for historical tracking
7. Results are logged and can be retrieved via the API endpoints

## Querying Historical Data

You can use the API to query historical crowd level data:

```bash
# Get the latest record (from any website)
curl https://your-worker-url.workers.dev/history/latest

# Get the latest record for a specific website
curl https://your-worker-url.workers.dev/history/latest?url=https://www.boulderwelt-muenchen-ost.de/

# Get the last 10 records (from any website)
curl https://your-worker-url.workers.dev/history?limit=10

# Get the last 10 records for a specific website
curl https://your-worker-url.workers.dev/history?limit=10&url=https://www.boulderwelt-muenchen-ost.de/

# List all configured websites
curl https://your-worker-url.workers.dev/websites

# Get historical data older than a specific timestamp (March 25, 2023)
curl https://your-worker-url.workers.dev/history?since=1679731200

# Get historical data for a specific website older than a specific timestamp
curl https://your-worker-url.workers.dev/history?since=1679731200&url=https://www.boulderwelt-muenchen-ost.de/
```