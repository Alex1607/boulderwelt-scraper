use worker::*;
use serde_json::Value;

const WEEKDAYS: [&str; 7] = ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"];

pub fn get_daily_averages_html(data: Value, selected_day: Option<usize>) -> String {
    let mut websites_data = std::collections::HashMap::new();
    
    // Process the data into a more usable format
    if let Some(entries) = data["data"].as_array() {
        for entry in entries {
            let website_name = entry["website_name"].as_str().unwrap_or("Unknown");
            let day = entry["day_of_week"].as_i64().unwrap_or(0) as usize;
            let avg = entry["average_percentage"].as_f64().unwrap_or(0.0);
            let count = entry["sample_count"].as_i64().unwrap_or(0);
            
            websites_data
                .entry(website_name.to_string())
                .or_insert_with(Vec::new)
                .push((day, avg, count));
        }
    }

    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Crowd Level Daily Averages</title>
    <style>
        :root {{
            --primary-color: #2196F3;
            --secondary-color: #FFC107;
            --background-color: #f5f5f5;
            --card-background: #ffffff;
            --text-color: #333333;
            --border-radius: 8px;
        }}

        body {{
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            line-height: 1.6;
            margin: 0;
            padding: 20px;
            background-color: var(--background-color);
            color: var(--text-color);
        }}

        .container {{
            max-width: 1200px;
            margin: 0 auto;
        }}

        h1 {{
            color: var(--primary-color);
            text-align: center;
            margin-bottom: 30px;
        }}

        .controls {{
            display: flex;
            justify-content: center;
            gap: 15px;
            margin-bottom: 30px;
        }}

        .btn {{
            padding: 10px 20px;
            border: none;
            border-radius: var(--border-radius);
            background-color: var(--primary-color);
            color: white;
            cursor: pointer;
            transition: background-color 0.3s;
            text-decoration: none;
        }}

        .btn:hover {{
            background-color: #1976D2;
        }}

        .btn.active {{
            background-color: var(--secondary-color);
        }}

        .website-card {{
            background-color: var(--card-background);
            border-radius: var(--border-radius);
            padding: 20px;
            margin-bottom: 20px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }}

        .website-name {{
            color: var(--primary-color);
            margin-top: 0;
            margin-bottom: 15px;
            font-size: 1.5em;
        }}

        .crowd-level {{
            display: flex;
            align-items: center;
            margin-bottom: 10px;
            padding: 10px;
            border-radius: var(--border-radius);
            background-color: #f8f9fa;
        }}

        .day-name {{
            width: 100px;
            font-weight: bold;
        }}

        .percentage-bar {{
            flex-grow: 1;
            height: 20px;
            background-color: #e9ecef;
            border-radius: var(--border-radius);
            overflow: hidden;
            margin: 0 15px;
        }}

        .percentage-fill {{
            height: 100%;
            background-color: var(--primary-color);
            transition: width 0.3s ease;
        }}

        .percentage-text {{
            width: 150px;
            text-align: right;
        }}

        .sample-count {{
            color: #666;
            font-size: 0.9em;
            margin-left: 10px;
        }}
    </style>
</head>
<body>
    <div class="container">
        <h1>Crowd Level Daily Averages</h1>
        
        <div class="controls">
            <a href="/daily-averages-view" class="btn{}" role="button">Full Week</a>
            {}
        </div>

        {}
    </div>
</body>
</html>"#,
        // Active state for Full Week button
        if selected_day.is_none() { " active" } else { "" },
        
        // Generate day selection buttons
        WEEKDAYS
            .iter()
            .enumerate()
            .map(|(i, day)| {
                format!(
                    r#"<a href="/daily-averages-view?day={}" class="btn{}" role="button">{}</a>"#,
                    i,
                    if selected_day == Some(i) { " active" } else { "" },
                    day
                )
            })
            .collect::<Vec<_>>()
            .join(""),
        
        // Generate website cards
        websites_data
            .iter()
            .map(|(name, data)| {
                let mut sorted_data = data.clone();
                sorted_data.sort_by_key(|(day, _, _)| *day);
                
                format!(
                    r#"<div class="website-card">
                        <h2 class="website-name">{}</h2>
                        {}</div>"#,
                    name,
                    sorted_data
                        .iter()
                        .filter(|(day, _, _)| selected_day.map_or(true, |d| d == *day))
                        .map(|(day, avg, count)| {
                            format!(
                                r#"<div class="crowd-level">
                                    <span class="day-name">{}</span>
                                    <div class="percentage-bar">
                                        <div class="percentage-fill" style="width: {}%"></div>
                                    </div>
                                    <span class="percentage-text">{:.1}%</span>
                                    <span class="sample-count">({} samples)</span>
                                </div>"#,
                                WEEKDAYS[*day], avg, avg, count
                            )
                        })
                        .collect::<Vec<_>>()
                        .join("\n")
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    );
    
    html
} 