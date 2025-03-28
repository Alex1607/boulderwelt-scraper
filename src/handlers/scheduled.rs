use worker::*;

use crate::db;
use crate::scraper;

/// Handler for scheduled CRON events
pub async fn scheduled_handler(_event: ScheduledEvent, env: Env, cron: String) -> Result<()> {
    console_log!("Scheduled task triggered at {} with cron '{}'", Date::now().to_string(), cron);
    
    // Get all configured websites
    let websites = scraper::get_configured_websites();
    let timestamp = Date::now().to_string();
    
    // Track overall success
    let mut success_count = 0;
    
    // Fetch data for all websites
    for website in websites {
        match scraper::fetch_crowd_data(&website).await {
            Ok(data) => {
                // Log the data in a structured format
                console_log!(
                    "CROWD_LEVEL_RECORD|{}|{}|{}|{}|{}",
                    timestamp,
                    data.crowd_level_percentage,
                    data.crowd_level_description,
                    website.name,
                    website.url
                );
                
                // Store data in D1 database
                match db::store_crowd_level(
                    &env, 
                    &timestamp, 
                    &data.crowd_level_percentage,
                    &data.crowd_level_description,
                    website.url.as_str(),
                    website.name.as_str()
                ).await {
                    Ok(_) => {
                        console_log!("Successfully stored data for {} in DB", website.name);
                        success_count += 1;
                    },
                    Err(e) => console_error!("Error storing data for {} in DB: {}", website.name, e),
                }
                
                // Log the full data for debugging
                console_log!("Successfully fetched data for {}: {:?}", website.name, data);
            },
            Err(e) => {
                console_error!("Error fetching data for {}: {}", website.name, e);
            }
        }
    }
    
    console_log!("Scheduled task completed, processed {} websites successfully", success_count);
    
    Ok(())
} 