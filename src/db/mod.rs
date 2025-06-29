use worker::*;
use serde_json::json;
use crate::scraper;

/// Stores a crowd level record in the database
pub async fn store_crowd_level(env: &Env, percentage: &str, description: &str, website_url: &str, website_name: &str) -> Result<()> {
    // Get the D1 database
    let d1 = match env.d1("DB") {
        Ok(db) => db,
        Err(e) => {
            console_error!("Error getting D1 database: {}", e);
            return Err(e);
        }
    };

    // Insert a new record
    let stmt = "INSERT INTO crowd_levels (percentage, description, website_url, website_name) VALUES (?, ?, ?, ?)";
    let prepared_stmt = d1.prepare(stmt);

    let _result = prepared_stmt
        .bind(&[percentage.into(), description.into(), website_url.into(), website_name.into()])?
        .run()
        .await?;

    console_log!("Inserted record successfully");

    Ok(())
}

/// Retrieves historical crowd level data with pagination
pub async fn get_crowd_level_history(env: &Env, since_timestamp: Option<i64>, until_timestamp: Option<i64>, website_url: Option<&str>) -> Result<serde_json::Value> {
    // Get the D1 database
    let d1 = match env.d1("DB") {
        Ok(db) => db,
        Err(e) => {
            console_error!("Error getting D1 database: {}", e);
            return Err(e);
        }
    };

    // Build the query based on parameters
    let mut conditions = Vec::new();
    let mut params = Vec::new();

    if let Some(url) = website_url {
        conditions.push("website_url = ?");
        params.push(url.into());
    }

    if let Some(ts) = since_timestamp {
        conditions.push("created_at > DATETIME(?, 'unixepoch')");
        params.push(ts.to_string().into());
    }

    if let Some(ts) = until_timestamp {
        conditions.push("created_at < DATETIME(?, 'unixepoch')");
        params.push(ts.to_string().into());
    }

    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };

    let stmt = format!("SELECT * FROM crowd_levels {} ORDER BY created_at DESC", where_clause);

    let prepared_stmt = d1.prepare(&stmt);

    let result = prepared_stmt
        .bind(&params)?
        .all()
        .await?;

    let records = result.results::<serde_json::Value>()?;

    Ok(json!({
        "data": records
    }))
}

/// Retrieves the latest crowd level record from the database for a specific website
pub async fn get_latest_crowd_level(env: &Env, website_url: Option<&str>) -> Result<serde_json::Value> {
    // Get the D1 database
    let d1 = match env.d1("DB") {
        Ok(db) => db,
        Err(e) => {
            console_error!("Error getting D1 database: {}", e);
            return Err(e);
        }
    };

    // Query the latest record
    let (stmt, params) = if let Some(url) = website_url {
        (
            "SELECT * FROM crowd_levels WHERE website_url = ? ORDER BY created_at DESC LIMIT 1",
            vec![url.into()]
        )
    } else {
        (
            "SELECT * FROM crowd_levels ORDER BY created_at DESC LIMIT 1",
            vec![]
        )
    };

    let result = d1.prepare(stmt)
        .bind(&params)?
        .all()
        .await?;

    let records = result.results::<serde_json::Value>()?;

    if records.is_empty() {
        return Ok(json!({
            "error": "No records found"
        }));
    }

    let record = &records[0];

    // Calculate additional fields based on the percentage
    let percentage = record["percentage"].as_str().unwrap_or("0");
    let percentage_float = percentage.parse::<f64>().unwrap_or(0.0);

    Ok(json!({
        "record": record,
        "crowd_level_percentage": percentage,
        "crowd_level_description": record["description"],
        "location": record["website_name"],
        "website_url": record["website_url"],
        "details": {
            "raw_percentage": percentage_float,
            "created_at": record["created_at"]
        }
    }))
}

/// Calculates and stores time-based averages for crowd levels
pub async fn update_time_averages(env: &Env) -> Result<()> {
    // Get the D1 database
    let d1 = match env.d1("DB") {
        Ok(db) => db,
        Err(e) => {
            console_error!("Error getting D1 database: {}", e);
            return Err(e);
        }
    };

    // Get all unique website URLs
    let websites = scraper::get_configured_websites();

    for website in websites {
        let website_url = website.url;
        let website_name = website.name;

        // Calculate averages for each day and hour combination for the last 4 weeks
        let avg_stmt = "
            SELECT 
                CAST(strftime('%w', created_at) AS INTEGER) as day_of_week,
                CAST(strftime('%H', created_at) AS INTEGER) as hour,
                ROUND(AVG(CAST(REPLACE(percentage, '%', '') AS FLOAT)), 2) as avg_percentage,
                COUNT(*) as sample_count
            FROM crowd_levels 
            WHERE website_url = ?
            AND created_at >= datetime('now', '-28 days')
            GROUP BY day_of_week, hour
            ORDER BY day_of_week, hour ASC
        ";

        let averages = d1.prepare(avg_stmt)
            .bind(&[website_url.as_str().into()])?
            .all()
            .await?
            .results::<serde_json::Value>()?;

        // Update time_averages table for each day/hour combination
        for avg in averages {
            let day_of_week = avg["day_of_week"].as_i64().unwrap_or(0);
            let hour = avg["hour"].as_i64().unwrap_or(0);
            let avg_percentage = avg["avg_percentage"].as_f64().unwrap_or(0.0);
            let sample_count = avg["sample_count"].as_i64().unwrap_or(0);

            let upsert_stmt = "
                INSERT INTO time_averages 
                    (website_url, website_name, day_of_week, hour, average_percentage, sample_count, last_updated)
                VALUES (?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)
                ON CONFLICT(website_url, day_of_week, hour)
                DO UPDATE SET 
                    average_percentage = excluded.average_percentage,
                    sample_count = excluded.sample_count,
                    last_updated = CURRENT_TIMESTAMP
            ";

            d1.prepare(upsert_stmt)
                .bind(&[
                    website_url.as_str().into(),
                    website_name.as_str().into(),
                    (day_of_week as i32).into(),
                    (hour as i32).into(),
                    avg_percentage.into(),
                    (sample_count as i32).into(),
                ])?
                .run()
                .await?;

            console_log!(
                "Updated average for {} on day {} at hour {}: {}% (samples: {})",
                website_name, day_of_week, hour, avg_percentage, sample_count
            );
        }
    }

    Ok(())
}

/// Retrieves the time-based averages for a specific website
pub async fn get_time_averages(env: &Env, website_url: Option<&str>) -> Result<serde_json::Value> {
    // Get the D1 database
    let d1 = match env.d1("DB") {
        Ok(db) => db,
        Err(e) => {
            console_error!("Error getting D1 database: {}", e);
            return Err(e);
        }
    };

    let (stmt, params) = if let Some(url) = website_url {
        (
            "SELECT * FROM time_averages WHERE website_url = ? ORDER BY day_of_week, hour ASC",
            vec![url.into()]
        )
    } else {
        (
            "SELECT * FROM time_averages ORDER BY website_url, day_of_week, hour ASC",
            vec![]
        )
    };

    let result = d1.prepare(stmt)
        .bind(&params)?
        .all()
        .await?;

    let records = result.results::<serde_json::Value>()?;

    // Process the data into a more structured format
    let mut processed_data = std::collections::HashMap::new();
    let weekdays = ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"];

    for record in records {
        let website_name = record["website_name"].as_str().unwrap_or("Unknown");
        let day_idx = record["day_of_week"].as_i64().unwrap_or(0) as usize;
        let hour = record["hour"].as_i64().unwrap_or(0);
        let avg = record["average_percentage"].as_f64().unwrap_or(0.0);
        let count = record["sample_count"].as_i64().unwrap_or(0);

        let website_data = processed_data
            .entry(website_name.to_string())
            .or_insert_with(|| std::collections::HashMap::new());

        let day_data = website_data
            .entry(weekdays[day_idx].to_string())
            .or_insert_with(|| std::collections::HashMap::new());

        day_data.insert(
            hour.to_string(),
            json!({
                "average": avg,
                "samples": count
            })
        );
    }

    Ok(json!({
        "data": processed_data
    }))
} 
