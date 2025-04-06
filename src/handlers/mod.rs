use worker::*;
use serde_json::json;

use crate::db;
use crate::scraper;

// Include the scheduled module
pub mod scheduled;
mod graph_template;

/// Handler for the /scrape endpoint
pub async fn scrape_handler(req: Request, env: Env) -> Result<Response> {
    // Parse the query parameters to get the website URL (optional)
    let url = req.url()?;
    let query_params: Vec<(String, String)> = url.query_pairs().map(|(k, v)| (k.to_string(), v.to_string())).collect();
    
    let website_url = query_params.iter()
        .find(|(k, _)| k == "url")
        .map(|(_, v)| v.as_str());
    
    // If a specific URL is provided, scrape that website
    let data = if let Some(url) = website_url {
        let found_website = scraper::get_configured_websites().into_iter()
            .find(|site| site.url == url);
        
        match found_website {
            Some(website) => {
                let mut data = Vec::new();
                data.push(scraper::fetch_crowd_data(&website).await?);
                data
            },
            None => {
                // If website not found in predefined list, return error
                return Response::error("Website not in configured list", 400);
            }
        }
    } else {
        scraper::fetch_all_data().await?.data
    };
    
    // If query param save=true, store in DB
    if url.query().unwrap_or("").contains("save=true") {
        for x in &data {
            match db::store_crowd_level(
                &env,
                x.crowd_level_percentage.as_str(),
                x.crowd_level_description.as_str(),
                x.website_url.as_str(),
                x.location.as_str()
            ).await {
                Ok(_) => console_log!("Successfully stored data in DB from scrape endpoint"),
                Err(e) => console_error!("Error storing data in DB from scrape endpoint: {}", e),
            }   
        }
    }
    
    Response::from_json(&data)
}

/// Handler for the /history endpoint
pub async fn history_handler(req: Request, env: Env) -> Result<Response> {
    // Get query parameters for timestamp and website_url
    let url = req.url()?;
    let query_params: Vec<(String, String)> = url.query_pairs().map(|(k, v)| (k.to_string(), v.to_string())).collect();
    
    let since_timestamp = query_params.iter()
        .find(|(k, _)| k == "since")
        .and_then(|(_, v)| v.parse::<i64>().ok());
    
    let until_timestamp = query_params.iter()
        .find(|(k, _)| k == "until")
        .and_then(|(_, v)| v.parse::<i64>().ok());
    
    let website_url = query_params.iter()
        .find(|(k, _)| k == "url")
        .map(|(_, v)| v.as_str());
    
    match db::get_crowd_level_history(&env, since_timestamp, until_timestamp, website_url).await {
        Ok(data) => {
            // Create response with JSON data
            let mut response = Response::from_json(&data)?;
            
            // Add cache control headers for Cloudflare (10 minutes = 600 seconds)
            response.headers_mut().set("Cache-Control", "public, max-age=600")?;
            
            Ok(response)
        },
        Err(e) => Response::error(format!("Error retrieving history: {}", e), 500)
    }
}

/// Handler for the /history/latest endpoint
pub async fn latest_handler(req: Request, env: Env) -> Result<Response> {
    // Get website_url parameter if provided
    let url = req.url()?;
    let query_params: Vec<(String, String)> = url.query_pairs().map(|(k, v)| (k.to_string(), v.to_string())).collect();
    
    let website_url = query_params.iter()
        .find(|(k, _)| k == "url")
        .map(|(_, v)| v.as_str());
    
    match db::get_latest_crowd_level(&env, website_url).await {
        Ok(data) => Response::from_json(&data),
        Err(e) => Response::error(format!("Error retrieving latest record: {}", e), 500)
    }
}

/// Handler for the /websites endpoint - returns list of configured websites
pub async fn websites_handler(_req: Request, _env: Env) -> Result<Response> {
    let websites = scraper::get_configured_websites();
    let websites_json = json!({
        "websites": websites
    });
    Response::from_json(&websites_json)
}

/// Handler for the /graph endpoint - returns HTML with interactive graph visualization
pub async fn graph_handler(req: Request, _env: Env) -> Result<Response> {
    // Parse query parameters
    let url = req.url()?;
    let query_params: Vec<(String, String)> = url.query_pairs().map(|(k, v)| (k.to_string(), v.to_string())).collect();
    
    let website_url = query_params.iter()
        .find(|(k, _)| k == "url")
        .map(|(_, v)| v.as_str());
    
    // Handle the "all" option
    let normalized_website_url = if website_url == Some("all") {
        None // None will show all gyms, as modified in the graph_template
    } else {
        website_url
    };
    
    let days = query_params.iter()
        .find(|(k, _)| k == "days")
        .map(|(_, v)| v.parse::<u32>().unwrap_or(7))
        .unwrap_or(7);
    
    // Get list of available websites for the dropdown
    let websites = scraper::get_configured_websites();

    // Create HTML with the graph
    let html = graph_template::generate_html(&websites, normalized_website_url, days);
    
    // Return the HTML response
    Response::from_html(&html)
}