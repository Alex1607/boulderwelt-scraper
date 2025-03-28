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
- **/history** - Retrieve historical crowd level data with pagination
  - Query parameters: `limit` (default: 100) and `offset` (default: 0)
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
```

## License

MIT 