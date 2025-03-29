use crate::scraper::WebsiteConfig;

/// Generate HTML for the graph visualization
pub fn generate_html(websites: &[WebsiteConfig], selected_website: Option<&str>, days: u32) -> String {
    // Generate website options HTML
    let mut website_options = String::new();
    
    // Add an "All Gyms" option at the top
    website_options.push_str("<option value=\"all\" ");
    if selected_website.is_none() || selected_website == Some("all") {
        website_options.push_str("selected");
    }
    website_options.push_str(">All Gyms</option>");
    
    for website in websites {
        let selected = selected_website.is_some_and(|s| s == website.url);
        website_options.push_str(&format!(
            "<option value=\"{}\" {}>{}</option>",
            website.url,
            if selected { "selected" } else { "" },
            website.name
        ));
    }
    
    // Generate time range options HTML
    let time_options = [(1, "1 Day"), (3, "3 Days"), (7, "7 Days"), (14, "14 Days"), (30, "30 Days")];
    let mut time_options_html = String::new();
    for &(value, label) in &time_options {
        time_options_html.push_str(&format!(
            "<option value=\"{}\" {}>{}</option>",
            value,
            if value == days { "selected" } else { "" },
            label
        ));
    }
    
    // Build the HTML
    format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Climbing Gym Crowd Levels</title>
    <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
    <script src="https://cdn.jsdelivr.net/npm/chartjs-adapter-date-fns"></script>
    <style>
        body {{
            font-family: Arial, sans-serif;
            margin: 0;
            padding: 20px;
            background-color: #f5f5f5;
        }}
        .container {{
            max-width: 1200px;
            margin: 0 auto;
            background-color: white;
            padding: 20px;
            border-radius: 8px;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
        }}
        h1 {{
            color: #333;
            text-align: center;
            margin-bottom: 30px;
        }}
        .chart-container {{
            height: 500px;
            margin-top: 20px;
        }}
        .controls {{
            display: flex;
            justify-content: space-between;
            margin-bottom: 20px;
            gap: 10px;
        }}
        select, button {{
            padding: 8px 12px;
            border-radius: 4px;
            border: 1px solid #ddd;
        }}
        button {{
            background-color: #4CAF50;
            color: white;
            border: none;
            cursor: pointer;
        }}
        button:hover {{
            background-color: #45a049;
        }}
        .loading-overlay {{
            position: absolute;
            top: 0;
            left: 0;
            right: 0;
            bottom: 0;
            background-color: rgba(255, 255, 255, 0.7);
            display: flex;
            justify-content: center;
            align-items: center;
            z-index: 1000;
            font-size: 1.2rem;
            visibility: hidden;
        }}
        .chart-wrapper {{
            position: relative;
        }}
        .pagination-info {{
            text-align: center;
            margin-top: 10px;
            color: #666;
            font-size: 0.9rem;
        }}
        .spinner {{
            border: 4px solid #f3f3f3;
            border-top: 4px solid #4CAF50;
            border-radius: 50%;
            width: 30px;
            height: 30px;
            animation: spin 1s linear infinite;
            margin-right: 10px;
        }}
        @keyframes spin {{
            0% {{ transform: rotate(0deg); }}
            100% {{ transform: rotate(360deg); }}
        }}
        @media (max-width: 768px) {{
            .controls {{
                flex-direction: column;
            }}
        }}
    </style>
</head>
<body>
    <div class="container">
        <h1>Climbing Gym Crowd Levels</h1>
        
        <div class="controls">
            <div>
                <label for="website">Gym Location:</label>
                <select id="website">
                    {website_options}
                </select>
            </div>
            <div>
                <label for="timeRange">Time Range:</label>
                <select id="timeRange">
                    {time_options_html}
                </select>
            </div>
            <button onclick="loadData()">Update Graph</button>
        </div>
        
        <div class="chart-wrapper">
            <div class="loading-overlay" id="loadingOverlay">
                <div class="spinner"></div>
                <span>Loading data...</span>
            </div>
            <div class="chart-container">
                <canvas id="crowdChart"></canvas>
            </div>
            <div class="pagination-info" id="paginationInfo"></div>
        </div>
    </div>

    <script>
        // Initial selected values
        const initialWebsite = document.getElementById('website').value;
        const initialDays = {days};
        
        // Chart reference
        let chart = null;
        
        // Initialize empty chart for a single gym
        function initChart(data, timeUnit = 'hour') {{
            const ctx = document.getElementById('crowdChart').getContext('2d');
            
            if (chart) {{
                chart.destroy();
            }}
            
            // Filter out invalid data points
            const validData = data.filter(point => 
                point.x instanceof Date && !isNaN(point.x) && 
                !isNaN(point.y) && point.y >= 0 && point.y <= 100
            );
            
            if (validData.length === 0) {{
                document.getElementById('paginationInfo').textContent = 'No valid data points found for the selected criteria';
                hideLoading();
                return;
            }}
            
            chart = new Chart(ctx, {{
                type: 'line',
                data: {{
                    datasets: [{{
                        label: 'Crowd Level Percentage',
                        data: validData,
                        borderColor: '#4CAF50',
                        backgroundColor: 'rgba(76, 175, 80, 0.1)',
                        borderWidth: 2,
                        fill: true,
                        tension: 0.2,
                        pointRadius: 3,
                        pointHoverRadius: 6
                    }}]
                }},
                options: {{
                    responsive: true,
                    maintainAspectRatio: false,
                    scales: {{
                        x: {{
                            type: 'time',
                            time: {{
                                unit: timeUnit,
                                displayFormats: {{
                                    hour: 'MMM d, HH:mm',
                                    day: 'MMM d'
                                }}
                            }},
                            title: {{
                                display: true,
                                text: 'Time'
                            }}
                        }},
                        y: {{
                            min: 0,
                            max: 100,
                            title: {{
                                display: true,
                                text: 'Crowd Level (%)'
                            }}
                        }}
                    }},
                    plugins: {{
                        tooltip: {{
                            callbacks: {{
                                title: function(tooltipItems) {{
                                    const date = new Date(tooltipItems[0].parsed.x);
                                    return date.toLocaleString();
                                }},
                                label: function(context) {{
                                    return 'Crowd Level: ' + context.parsed.y + '%';
                                }}
                            }}
                        }}
                    }}
                }}
            }});
        }}
        
        // Initialize chart for multiple gyms
        function initMultiChart(datasets, timeUnit = 'hour') {{
            const ctx = document.getElementById('crowdChart').getContext('2d');
            
            if (chart) {{
                chart.destroy();
            }}
            
            if (datasets.length === 0 || datasets.every(d => d.data.length === 0)) {{
                document.getElementById('paginationInfo').textContent = 'No valid data points found for the selected criteria';
                hideLoading();
                return;
            }}
            
            chart = new Chart(ctx, {{
                type: 'line',
                data: {{
                    datasets: datasets
                }},
                options: {{
                    responsive: true,
                    maintainAspectRatio: false,
                    scales: {{
                        x: {{
                            type: 'time',
                            time: {{
                                unit: timeUnit,
                                displayFormats: {{
                                    hour: 'MMM d, HH:mm',
                                    day: 'MMM d'
                                }}
                            }},
                            title: {{
                                display: true,
                                text: 'Time'
                            }}
                        }},
                        y: {{
                            min: 0,
                            max: 100,
                            title: {{
                                display: true,
                                text: 'Crowd Level (%)'
                            }}
                        }}
                    }},
                    plugins: {{
                        tooltip: {{
                            callbacks: {{
                                title: function(tooltipItems) {{
                                    const date = new Date(tooltipItems[0].parsed.x);
                                    return date.toLocaleString();
                                }},
                                label: function(context) {{
                                    return context.dataset.label + ': ' + context.parsed.y + '%';
                                }}
                            }}
                        }}
                    }}
                }}
            }});
        }}
        
        // Show loading indicator
        function showLoading() {{
            document.getElementById('loadingOverlay').style.visibility = 'visible';
        }}
        
        // Hide loading indicator
        function hideLoading() {{
            document.getElementById('loadingOverlay').style.visibility = 'hidden';
        }}
        
        // Update pagination info
        function updatePaginationInfo(total, loaded) {{
            const info = document.getElementById('paginationInfo');
            info.textContent = 'Showing ' + loaded + ' of ' + total + ' data points';
        }}
        
        // Load data from the API for a single gym
        async function loadGymData(website, days, offset = 0, existingData = []) {{
            // Calculate a reasonable batch size (1 day worth of data at 10-minute intervals)
            const batchSize = 144; // 6 records per hour * 24 hours
            
            try {{
                // Fetch data from the history endpoint with pagination
                const url = '/history?url=' + encodeURIComponent(website) + 
                            '&limit=' + batchSize + 
                            '&offset=' + offset;
                
                const response = await fetch(url);
                const result = await response.json();
                
                if (!result.data || !Array.isArray(result.data)) {{
                    throw new Error('Invalid data received from server');
                }}
                
                // Process the data into chart format
                const newDataPoints = result.data.map(record => {{
                    // Parse the timestamp to a proper date format that Chart.js can understand
                    let timestamp = new Date(record.timestamp);
                    
                    return {{
                        x: timestamp,
                        y: parseFloat(record.percentage) || 0
                    }};
                }});
                
                // Sort data by timestamp (newest first for consistent display)
                const allData = [...existingData, ...newDataPoints].sort((a, b) => a.x - b.x);
                
                // Determine if we need to fetch more data
                const totalPointsNeeded = days * 24 * 6; // days * hours * data points per hour
                
                if (allData.length < Math.min(totalPointsNeeded, result.pagination.total) && 
                    result.pagination.has_more) {{
                    // Continue loading more data
                    return await loadGymData(website, days, offset + batchSize, allData);
                }} else {{
                    return {{
                        data: allData,
                        total: result.pagination.total
                    }};
                }}
            }} catch (error) {{
                console.error('Error loading data:', error);
                return {{
                    data: existingData,
                    total: existingData.length,
                    error: error.message
                }};
            }}
        }}
        
        // Load data from API
        async function loadData() {{
            showLoading();
            
            const website = document.getElementById('website').value;
            const days = parseInt(document.getElementById('timeRange').value);
            
            // Determine appropriate time unit based on days selected
            let timeUnit = 'hour';
            if (days > 7) {{
                timeUnit = 'day';
            }}
            
            if (website === 'all') {{
                // Fetch data for all gyms
                try {{
                    // First get the list of websites
                    const websitesResponse = await fetch('/websites');
                    const websitesData = await websitesResponse.json();
                    
                    if (!websitesData.websites || !Array.isArray(websitesData.websites)) {{
                        throw new Error('Invalid websites data received');
                    }}
                    
                    // Create an array of promises to fetch data for all gyms in parallel
                    const dataPromises = websitesData.websites.map(site => 
                        loadGymData(site.url, days)
                    );
                    
                    // Wait for all promises to resolve
                    const results = await Promise.all(dataPromises);
                    
                    // Create datasets for the chart
                    const datasets = results.map((result, index) => {{
                        const website = websitesData.websites[index];
                        // Generate a stable color for this gym
                        const hue = (index * 137) % 360; // Golden angle to get distinct colors
                        
                        return {{
                            label: website.name,
                            data: result.data,
                            borderColor: 'hsl(' + hue + ', 70%, 50%)',
                            backgroundColor: 'hsla(' + hue + ', 70%, 50%, 0.1)',
                            borderWidth: 2,
                            fill: false,
                            tension: 0.2,
                            pointRadius: 3,
                            pointHoverRadius: 6
                        }};
                    }});
                    
                    // Initialize chart with multiple datasets
                    initMultiChart(datasets, timeUnit);
                    
                    // Update pagination info
                    const totalDataPoints = results.reduce((sum, result) => sum + result.total, 0);
                    const loadedDataPoints = results.reduce((sum, result) => sum + result.data.length, 0);
                    updatePaginationInfo(totalDataPoints, loadedDataPoints);
                    
                    hideLoading();
                }} catch (error) {{
                    console.error('Error loading data for all gyms:', error);
                    hideLoading();
                    alert('Error loading data. Please try again.');
                }}
            }} else {{
                // Fetch data for a single gym
                try {{
                    const result = await loadGymData(website, days);
                    
                    // Update chart with the data
                    initChart(result.data, timeUnit);
                    
                    // Update pagination info
                    updatePaginationInfo(result.total, result.data.length);
                    
                    hideLoading();
                }} catch (error) {{
                    console.error('Error loading data:', error);
                    hideLoading();
                    alert('Error loading data. Please try again.');
                }}
            }}
            
            // Update URL
            updateURL();
        }}
        
        // Update URL without reloading the page
        function updateURL() {{
            const website = document.getElementById('website').value;
            const days = document.getElementById('timeRange').value;
            
            const url = new URL(window.location);
            url.searchParams.set('url', website);
            url.searchParams.set('days', days);
            window.history.pushState({{}},'', url);
        }}
        
        // Event listeners for controls
        document.getElementById('website').addEventListener('change', function() {{
            loadData();
            updateURL();
        }});
        
        document.getElementById('timeRange').addEventListener('change', function() {{
            loadData();
            updateURL();
        }});
        
        // Initialize the chart when page loads
        document.addEventListener('DOMContentLoaded', function() {{
            loadData();
        }});
    </script>
</body>
</html>"#,
        website_options = website_options,
        time_options_html = time_options_html,
        days = days
    )
} 