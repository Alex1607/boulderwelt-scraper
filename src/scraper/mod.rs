use regex::Regex;
use scraper::{Html, Selector};
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

/// Fetches data from a specified bouldering website and extracts the crowd level information
pub async fn fetch_website_data(website: &WebsiteConfig) -> Result<ScrapedWebsiteData> {
    let url = &website.url;

    // Fetch the HTML content
    let mut resp = Fetch::Url(url.parse()?).send().await?;
    let html = resp.text().await?;

    // Parse the HTML
    let document = Html::parse_document(&html);

    // Log HTML for debugging (in production, you'd remove this or limit it)
    console_log!("Fetched HTML from {}, length: {}", url, html.len());

    // Try to find the pointer.png element and its margin-left value
    let result = find_pointer_margin_left(&document, &html)?;

    // Parse the percentage as a float for additional calculations if needed
    let percentage_float = result.parse::<f64>().unwrap_or(0.0);

    // Calculate a crowd level description based on the percentage
    let crowd_level_description = if percentage_float < 20.0 {
        "Very low".to_string()
    } else if percentage_float < 40.0 {
        "Low".to_string()
    } else if percentage_float < 60.0 {
        "Moderate".to_string()
    } else if percentage_float < 80.0 {
        "High".to_string()
    } else {
        "Very high".to_string()
    };

    // Get the scrape time, handling the Result<Option<String>>
    let scrape_time = match resp.headers().get("cf-request-time") {
        Ok(Some(time)) => time,
        _ => String::new(),
    };
    
    // Return the result as a struct, ensuring all strings are owned
    Ok(ScrapedWebsiteData {
        timestamp: Date::now().to_string(),
        url: url.clone(),
        pointer_margin_left_percentage: result.clone(),
        crowd_level_percentage: result,
        crowd_level_description: crowd_level_description,
        location: website.name.clone(),
        website_url: website.url.clone(),
        details: ScrapedDetails {
            raw_percentage: percentage_float,
            scrape_time_ms: scrape_time,
        },
    })
}

/// Fetches data specifically from Boulderwelt München Ost website (for backward compatibility)
pub async fn fetch_boulderwelt_data() -> Result<ScrapedData> {
    let websites = get_configured_websites();
    let mut all_data = Vec::new();

    for website in websites {
        match fetch_website_data(&website).await {
            Ok(data) => all_data.push(data),
            Err(e) => console_error!("Error fetching data from {}: {}", website.name, e),
        }
    }

    Ok(ScrapedData {
        timestamp: Date::now().to_string(),
        data: all_data,
    })
}

/// Finds the pointer.png element in the HTML and extracts its margin-left percentage value
fn find_pointer_margin_left(document: &Html, html_content: &str) -> Result<String> {
    // First try the exact selector pattern from the provided example
    let exact_selectors = [
        "img[src*='wp-content/plugins/cxo-crowd-level//resources/img/pointer.png']",
        "img[width='21'][height='15'][src*='pointer.png']",
        "img.entered.lazyloaded[src*='pointer.png']",
        "img[data-ll-status='loaded'][src*='pointer.png']",
    ];

    for selector_str in exact_selectors.iter() {
        if let Ok(selector) = Selector::parse(selector_str) {
            for element in document.select(&selector) {
                console_log!("Found exact match element: {:?}", element.value());

                // Check style attribute
                if let Some(style) = element.value().attr("style") {
                    let style_str = style.to_string(); // Clone to avoid pointer issues
                    if let Some(margin_left) = extract_margin_left(&style_str) {
                        console_log!("Found exact match margin-left: {}%", margin_left);
                        return Ok(margin_left);
                    }
                }
            }
        }
    }

    // Try a direct regex pattern based on the example
    let html_content_owned = html_content.to_string(); // Clone to avoid memory issues
    let exact_regex = r#"<img[^>]*src="[^"]*pointer\.png"[^>]*style="[^"]*margin-left:\s*(\d+(?:\.\d+)?)%[^"]*"[^>]*>"#;
    if let Ok(regex) = Regex::new(exact_regex) {
        if let Some(captures) = regex.captures(&html_content_owned) {
            if let Some(percentage) = captures.get(1) {
                let percentage_str = percentage.as_str().to_string(); // Ensure we own the string
                console_log!(
                    "Found margin-left via exact regex: {}%",
                    percentage_str
                );
                return Ok(percentage_str);
            }
        }
    }

    // Try to match the exact pattern provided by the user
    let user_exact_pattern = r#"<img width="21" height="15" src="[^"]*pointer\.png" style="margin-left:\s*(\d+(?:\.\d+)?)%;"[^>]*>"#;
    if let Ok(regex) = Regex::new(user_exact_pattern) {
        if let Some(captures) = regex.captures(&html_content_owned) {
            if let Some(percentage) = captures.get(1) {
                let percentage_str = percentage.as_str().to_string(); // Ensure we own the string
                console_log!(
                    "Found margin-left via user's exact pattern: {}%",
                    percentage_str
                );
                return Ok(percentage_str);
            }
        }
    }

    // Fall back to more generic approaches if the exact match fails

    // 1. Try CSS selectors for the image
    let selectors = [
        "img[src*='pointer.png']",
        "img[src*='pointer']",
        ".pointer",
        "#pointer",
        "[class*='pointer']",
        "[id*='pointer']",
        // Slider-related selectors that might contain the pointer
        ".slider-pointer",
        ".carousel-pointer",
        ".indicator",
        ".slider-indicator",
        // Navigation elements that might use pointer.png
        ".nav-pointer",
        ".navigation-indicator",
    ];

    for selector_str in selectors.iter() {
        if let Ok(selector) = Selector::parse(selector_str) {
            for element in document.select(&selector) {
                console_log!(
                    "Found element matching selector {}: {:?}",
                    selector_str,
                    element.value()
                );

                // Check style attribute
                if let Some(style) = element.value().attr("style") {
                    let style_str = style.to_string(); // Clone to avoid pointer issues
                    if let Some(margin_left) = extract_margin_left(&style_str) {
                        console_log!("Found margin-left: {}% in style attribute", margin_left);
                        return Ok(margin_left);
                    }
                }

                // Check for data attributes that might contain position info
                if let Some(position) = element.value().attr("data-position") {
                    let position_str = position.to_string(); // Clone to avoid pointer issues
                    console_log!("Found data-position attribute: {}", position_str);
                    if let Ok(value) = position_str.parse::<f64>() {
                        return Ok(value.to_string());
                    }
                }
            }
        }
    }

    // 2. Look for pointer.png in the HTML content
    let regex_patterns = [
        r#"pointer\.png[^>]*style="[^"]*margin-left:\s*(\d+(?:\.\d+)?)%"#,
        r#"pointer[^>]*style="[^"]*margin-left:\s*(\d+(?:\.\d+)?)%"#,
        r#"style="[^"]*margin-left:\s*(\d+(?:\.\d+)?)%[^"]*pointer"#,
    ];

    for pattern in regex_patterns.iter() {
        if let Ok(regex) = Regex::new(pattern) {
            if let Some(captures) = regex.captures(&html_content_owned) {
                if let Some(percentage) = captures.get(1) {
                    let percentage_str = percentage.as_str().to_string(); // Ensure we own the string
                    console_log!("Found margin-left via regex: {}%", percentage_str);
                    return Ok(percentage_str);
                }
            }
        }
    }

    // 3. Look for JavaScript variables that might set the pointer position
    let js_patterns = [
        r#"pointerPosition\s*=\s*(\d+(?:\.\d+)?)"#,
        r#"setPointerPosition\s*\(\s*(\d+(?:\.\d+)?)"#,
        r#"margin-left['"]\s*:\s*['"]([\d.]+)%"#,
    ];

    for pattern in js_patterns.iter() {
        if let Ok(regex) = Regex::new(pattern) {
            if let Some(captures) = regex.captures(&html_content_owned) {
                if let Some(percentage) = captures.get(1) {
                    let percentage_str = percentage.as_str().to_string(); // Ensure we own the string
                    console_log!("Found pointer position in JS: {}%", percentage_str);
                    return Ok(percentage_str);
                }
            }
        }
    }

    // 4. Generic search for elements with margin-left in style
    let double_quote_margin_regex =
        Regex::new(r#"style=".*?margin-left:\s*(\d+(?:\.\d+)?)%.*?""#).unwrap();
    let single_quote_margin_regex =
        Regex::new(r#"style='.*?margin-left:\s*(\d+(?:\.\d+)?)%.*?'"#).unwrap();

    // Try with double quotes
    for captures in double_quote_margin_regex.captures_iter(&html_content_owned) {
        if let Some(percentage) = captures.get(1) {
            let percentage_str = percentage.as_str().to_string(); // Ensure we own the string
            console_log!(
                "Found generic margin-left (double quotes): {}%",
                percentage_str
            );
            return Ok(percentage_str);
        }
    }

    // Try with single quotes
    for captures in single_quote_margin_regex.captures_iter(&html_content_owned) {
        if let Some(percentage) = captures.get(1) {
            let percentage_str = percentage.as_str().to_string(); // Ensure we own the string
            console_log!(
                "Found generic margin-left (single quotes): {}%",
                percentage_str
            );
            return Ok(percentage_str);
        }
    }

    // If we couldn't find anything, return an error
    Err(Error::RustError(
        "Could not find pointer.png or its margin-left value".into(),
    ))
}

/// Extracts the margin-left percentage value from a style attribute
fn extract_margin_left(style: &str) -> Option<String> {
    let margin_regex = Regex::new(r#"margin-left:\s*(\d+(?:\.\d+)?)%"#).unwrap();

    if let Some(captures) = margin_regex.captures(style) {
        if let Some(percentage) = captures.get(1) {
            return Some(percentage.as_str().to_string());
        }
    }
    None
}
