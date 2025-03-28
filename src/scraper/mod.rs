use serde::{Deserialize, Serialize};
use worker::*;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct WebsiteConfig {
    pub url: String,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ScrapedDetails {
    pub raw_percentage: f64,
    pub scrape_time_ms: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ScrapedWebsiteData {
    pub timestamp: String,
    pub url: String,
    pub pointer_margin_left_percentage: String,
    pub crowd_level_percentage: String,
    pub crowd_level_description: String,
    pub location: String,
    pub website_url: String,
    pub details: ScrapedDetails,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ScrapedData {
    pub timestamp: String,
    pub data: Vec<ScrapedWebsiteData>,
}

/// Returns a list of configured websites to scrape
pub fn get_configured_websites() -> Vec<WebsiteConfig> {
    vec![
        WebsiteConfig {
            url: "https://www.boulderwelt-muenchen-ost.de/".to_string(),
            name: "Boulderwelt München Ost".to_string(),
        },
        WebsiteConfig {
            url: "https://www.boulderwelt-muenchen-west.de/".to_string(),
            name: "Boulderwelt München West".to_string(),
        },
        WebsiteConfig {
            url: "https://www.boulderwelt-muenchen-sued.de/".to_string(),
            name: "Boulderwelt München Süd".to_string(),
        },
        // Add more websites here as needed
    ]
}

/// Fetches crowd level data directly from the AJAX endpoint
pub async fn fetch_crowd_data(website: &WebsiteConfig) -> Result<ScrapedWebsiteData> {
    let site_url = &website.url;
    
    // Construct the AJAX post URL - we'll use the WordPress AJAX endpoint
    let ajax_url = format!("{}wp-admin/admin-ajax.php?action=cxo_get_crowd_indicator", site_url);
    
    // Fetch the JSON data - using GET with action parameter in the URL
    console_log!("Fetching AJAX data from {}", ajax_url);
    let mut resp = Fetch::Url(ajax_url.parse()?).send().await?;
    
    // Check if the response is successful
    if resp.status_code() != 200 {
        return Err(Error::from(format!("AJAX request failed with status: {}", resp.status_code())));
    }
    
    // Parse the JSON response
    let json_text = resp.text().await?;
    console_log!("Received AJAX response: {}", json_text);
    
    // Parse the JSON
    let response: serde_json::Value = match serde_json::from_str(&json_text) {
        Ok(json) => json,
        Err(e) => return Err(Error::from(format!("Failed to parse JSON: {}", e))),
    };
    
    // Extract the level from the JSON response
    let level = match response.get("level") {
        Some(serde_json::Value::Number(level)) => {
            match level.as_f64() {
                Some(val) => val,
                None => return Err(Error::from("Failed to parse level as number")),
            }
        },
        _ => {
            console_log!("Full response: {:?}", response);
            return Err(Error::from("Failed to extract level from response"));
        }
    };
    
    // Convert the level to a string percentage
    let percentage = format!("{}", level);
    
    // Calculate a crowd level description based on the percentage
    let crowd_level_description = if level < 20.0 {
        "Very low".to_string()
    } else if level < 40.0 {
        "Low".to_string()
    } else if level < 60.0 {
        "Moderate".to_string()
    } else if level < 80.0 {
        "High".to_string()
    } else {
        "Very high".to_string()
    };

    // Get the scrape time, handling the Result<Option<String>>
    let scrape_time = match resp.headers().get("cf-request-time") {
        Ok(Some(time)) => time,
        _ => String::new(),
    };
    
    // Return the result as a struct
    Ok(ScrapedWebsiteData {
        timestamp: Date::now().to_string(),
        url: site_url.clone(),
        pointer_margin_left_percentage: percentage.clone(),
        crowd_level_percentage: percentage,
        crowd_level_description,
        location: website.name.clone(),
        website_url: website.url.clone(),
        details: ScrapedDetails {
            raw_percentage: level,
            scrape_time_ms: scrape_time,
        },
    })
}

/// Fetches data from all websites
pub async fn fetch_all_data() -> Result<ScrapedData> {
    let websites = get_configured_websites();
    let mut all_data = Vec::new();

    for website in websites {
        match fetch_crowd_data(&website).await {
            Ok(data) => all_data.push(data),
            Err(e) => console_error!("Error fetching data from {}: {}", website.name, e),
        }
    }

    Ok(ScrapedData {
        timestamp: Date::now().to_string(),
        data: all_data,
    })
}
