use worker::*;
use serde_json::json;

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
pub async fn get_crowd_level_history(env: &Env, since_timestamp: Option<i64>, website_url: Option<&str>) -> Result<serde_json::Value> {
    // Get the D1 database
    let d1 = match env.d1("DB") {
        Ok(db) => db,
        Err(e) => {
            console_error!("Error getting D1 database: {}", e);
            return Err(e);
        }
    };
    
    // Query the history based on timestamp, filtering by website_url if provided
    let (stmt, params) = if let Some(url) = website_url {
        match since_timestamp {
            Some(ts) => (
                "SELECT * FROM crowd_levels WHERE website_url = ? AND created_at < DATETIME(?, 'unixepoch') ORDER BY created_at DESC",
                vec![url.into(), ts.to_string().into()]
            ),
            None => (
                "SELECT * FROM crowd_levels WHERE website_url = ? ORDER BY created_at DESC",
                vec![url.into()]
            )
        }
    } else {
        match since_timestamp {
            Some(ts) => (
                "SELECT * FROM crowd_levels WHERE created_at < DATETIME(?, 'unixepoch') ORDER BY created_at DESC",
                vec![ts.to_string().into()]
            ),
            None => (
                "SELECT * FROM crowd_levels ORDER BY created_at DESC",
                vec![]
            )
        }
    };
    
    let prepared_stmt = d1.prepare(stmt);
    
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