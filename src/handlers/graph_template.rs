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
    <script src="https://cdn.jsdelivr.net/npm/chartjs-plugin-zoom"></script>
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
        .nav-bar {{
            display: flex;
            justify-content: center;
            gap: 20px;
            margin-bottom: 20px;
            padding: 10px;
            border-bottom: 1px solid #ddd;
        }}
        .nav-link {{
            text-decoration: none;
            color: #4CAF50;
            padding: 5px 10px;
            border-radius: 4px;
            transition: background-color 0.2s;
        }}
        .nav-link.active {{
            background-color: #4CAF50;
            color: white;
        }}
        .nav-link:hover:not(.active) {{
            background-color: #e8f5e9;
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
        <div class="nav-bar">
            <a href="/graph?url=all&days=1&offset=0" class="nav-link active">Live Graph</a>
            <a href="/time-averages-view" class="nav-link">Time Averages</a>
        </div>

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
            <div>
                <label for="showOffset">
                    <input type="checkbox" id="showOffset" onchange="loadData()">
                    Compare with last week
                </label>
            </div>
            <button onclick="loadData()">Update Graph</button>
            <button id="resetZoom" onclick="resetZoom()" style="display: none;">Reset Zoom</button>
        </div>
        <div style="text-align: center; margin-top: 5px; font-size: 0.9rem; color: #666;">
            <p>Tip: Click and drag on the graph to zoom into a specific area</p>
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
                        }},
                        zoom: {{
                            zoom: {{
                                wheel: {{
                                    enabled: true,
                                }},
                                pinch: {{
                                    enabled: true
                                }},
                                drag: {{
                                    enabled: true,
                                    backgroundColor: 'rgba(76, 175, 80, 0.1)',
                                    borderColor: 'rgba(76, 175, 80, 0.5)',
                                    borderWidth: 1
                                }},
                                mode: 'xy',
                                onZoomComplete: function() {{
                                    document.getElementById('resetZoom').style.display = 'inline-block';
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
                        }},
                        zoom: {{
                            zoom: {{
                                wheel: {{
                                    enabled: true,
                                }},
                                pinch: {{
                                    enabled: true
                                }},
                                drag: {{
                                    enabled: true,
                                    backgroundColor: 'rgba(76, 175, 80, 0.1)',
                                    borderColor: 'rgba(76, 175, 80, 0.5)',
                                    borderWidth: 1
                                }},
                                mode: 'xy',
                                onZoomComplete: function() {{
                                    document.getElementById('resetZoom').style.display = 'inline-block';
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

        // Load data from the API for a single gym
        async function loadGymData(website, days, offset = 0) {{
            try {{
                // Calculate the since timestamp based on the requested days
                const since = new Date();
                since.setDate(since.getDate() - days - offset);
                const sinceTimestamp = Math.floor(since.getTime() / 1000);

                // Calculate the until timestamp for offset data
                const until = new Date();
                if (offset > 0) {{
                    until.setDate(until.getDate() - offset);
                }}
                const untilTimestamp = Math.floor(until.getTime() / 1000);

                // Fetch data from the history endpoint with the since parameter
                let url = '/history?url=' + encodeURIComponent(website) + '&since=' + sinceTimestamp;
                if (offset > 0) {{
                    url += '&until=' + untilTimestamp;
                }}

                const response = await fetch(url);
                const result = await response.json();

                if (!result.data || !Array.isArray(result.data)) {{
                    throw new Error('Invalid data received from server');
                }}

                // Process the data into chart format
                const newDataPoints = result.data.map(record => {{
                    // Parse the timestamp to a proper date format that Chart.js can understand
                    let timestamp;

                    // Try multiple timestamp field possibilities
                    if (record.created_at) {{
                        // SQLite timestamp format from D1 database: "2025-03-29 22:50:09"
                        try {{
                            // First method: Convert SQLite format to ISO format for JavaScript Date
                            timestamp = new Date(record.created_at.replace(' ', 'T') + 'Z');

                            // Check if the date is valid
                            if (isNaN(timestamp.getTime())) {{
                                // Alternative parsing method
                                const parts = record.created_at.split(/[- :]/);
                                // parts[0] = year, parts[1] = month (0-based), parts[2] = day, 
                                // parts[3] = hours, parts[4] = minutes, parts[5] = seconds
                                if (parts.length >= 6) {{
                                    timestamp = new Date(
                                        parseInt(parts[0]), 
                                        parseInt(parts[1]) - 1, // month is 0-based
                                        parseInt(parts[2]),
                                        parseInt(parts[3]),
                                        parseInt(parts[4]),
                                        parseInt(parts[5])
                                    );
                                }}

                                if (isNaN(timestamp.getTime())) {{
                                    return null;
                                }}
                            }}
                        }} catch (e) {{
                            return null;
                        }}
                    }} else {{
                        // Try other possible time fields if available
                        if (record.timestamp) {{
                            try {{
                                timestamp = new Date(record.timestamp);
                                if (isNaN(timestamp.getTime())) {{
                                    return null;
                                }}
                            }} catch (e) {{
                                return null;
                            }}
                        }} else {{
                            return null;
                        }}
                    }}

                    // Extract percentage value, trying multiple possible field names
                    let percentValue = 0;
                    if (record.percentage !== undefined) {{
                        percentValue = parseFloat(record.percentage);
                    }} else if (record.crowd_level_percentage !== undefined) {{
                        percentValue = parseFloat(record.crowd_level_percentage);
                    }} else if (record.value !== undefined) {{
                        percentValue = parseFloat(record.value);
                    }}

                    if (isNaN(percentValue)) {{
                        return null;
                    }}

                    return {{
                        x: timestamp,
                        y: percentValue
                    }};
                }}).filter(point => point !== null);

                // Combine with existing data and sort
                const allData = [...newDataPoints].sort((a, b) => a.x - b.x);

                // Client-side filtering is now minimal since server is already filtering
                // Fix future dates by adjusting them to current time
                const currentYear = new Date().getFullYear();
                const filteredData = allData.map(point => {{
                    // Create a copy of the point to avoid modifying the original
                    const adjustedPoint = {{...point}};

                    // If the date is in the future, adjust it to current year
                    if (adjustedPoint.x.getFullYear() > currentYear) {{
                        const pointDate = new Date(adjustedPoint.x);
                        pointDate.setFullYear(currentYear);
                        adjustedPoint.x = pointDate;
                    }}

                    return adjustedPoint;
                }});

                // We don't need to filter by date range anymore since the server does it
                return {{
                    data: filteredData,
                    total: newDataPoints.length,
                    filtered: filteredData.length
                }};
            }} catch (error) {{
                console.error('Error loading data:', error);
                return {{
                    data: [],
                    total: 0,
                    error: error.message
                }};
            }}
        }}

        // Load data from API
        async function loadData() {{
            showLoading();
            // Hide reset zoom button when loading new data
            document.getElementById('resetZoom').style.display = 'none';

            const website = document.getElementById('website').value;
            const days = parseInt(document.getElementById('timeRange').value);
            const showOffset = document.getElementById('showOffset').checked;

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
                    const dataPromises = websitesData.websites.map(site => {{
                        const promises = [loadGymData(site.url, days)];
                        if (showOffset) {{
                            promises.push(loadGymData(site.url, days, 7));
                        }}
                        return Promise.all(promises);
                    }});

                    // Wait for all promises to resolve
                    const results = await Promise.all(dataPromises);

                    // Create datasets for the chart
                    const datasets = results.flatMap((result, index) => {{
                        const website = websitesData.websites[index];
                        // Generate a stable color for this gym
                        const hue = (index * 137) % 360; // Golden angle to get distinct colors

                        const currentDataset = {{
                            label: website.name,
                            data: result[0].data,
                            borderColor: 'hsl(' + hue + ', 70%, 50%)',
                            backgroundColor: 'hsla(' + hue + ', 70%, 50%, 0.1)',
                            borderWidth: 2,
                            fill: false,
                            tension: 0.2,
                            pointRadius: 3,
                            pointHoverRadius: 6
                        }};

                        if (showOffset && result[1]) {{
                            // Adjust timestamps for offset data to align with current data
                            const offsetData = result[1].data.map(point => {{
                                const adjustedDate = new Date(point.x);
                                adjustedDate.setDate(adjustedDate.getDate() + 7);
                                return {{
                                    x: adjustedDate,
                                    y: point.y
                                }};
                            }});

                            const offsetDataset = {{
                                label: website.name + ' (7 days ago)',
                                data: offsetData,
                                borderColor: 'hsl(' + hue + ', 70%, 25%)',
                                backgroundColor: 'hsla(' + hue + ', 70%, 25%, 0.1)',
                                borderWidth: 2,
                                borderDash: [5, 5],
                                fill: false,
                                tension: 0.2,
                                pointRadius: 2,
                                pointHoverRadius: 5
                            }};

                            return [currentDataset, offsetDataset];
                        }}

                        return [currentDataset];
                    }});

                    // Initialize chart with multiple datasets
                    initMultiChart(datasets, timeUnit);

                    hideLoading();
                }} catch (error) {{
                    console.error('Error loading data for all gyms:', error);
                    hideLoading();
                    alert('Error loading data. Please try again.');
                }}
            }} else {{
                // Fetch data for a single gym
                try {{
                    const currentData = await loadGymData(website, days);
                    let datasets = [];

                    if (showOffset) {{
                        const offsetData = await loadGymData(website, days, 7);

                        // Adjust timestamps for offset data to align with current data
                        const adjustedOffsetData = offsetData.data.map(point => {{
                            const adjustedDate = new Date(point.x);
                            adjustedDate.setDate(adjustedDate.getDate() + 7);
                            return {{
                                x: adjustedDate,
                                y: point.y
                            }};
                        }});

                        datasets = [
                            {{
                                label: 'Current',
                                data: currentData.data,
                                borderColor: '#4CAF50',
                                backgroundColor: 'rgba(76, 175, 80, 0.1)',
                                borderWidth: 2,
                                fill: true,
                                tension: 0.2,
                                pointRadius: 3,
                                pointHoverRadius: 6
                            }},
                            {{
                                label: '7 days ago',
                                data: adjustedOffsetData,
                                borderColor: '#2E7D32',
                                backgroundColor: 'rgba(46, 125, 50, 0.1)',
                                borderWidth: 2,
                                borderDash: [5, 5],
                                fill: true,
                                tension: 0.2,
                                pointRadius: 2,
                                pointHoverRadius: 5
                            }}
                        ];
                        initMultiChart(datasets, timeUnit);
                    }} else {{
                        initChart(currentData.data, timeUnit);
                    }}

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
            const showOffset = document.getElementById('showOffset').checked;

            const url = new URL(window.location);
            url.searchParams.set('url', website);
            url.searchParams.set('days', days);
            url.searchParams.set('offset', showOffset ? '1' : '0');
            window.history.pushState({{}},'', url);
        }}

        // Reset zoom to original scale
        function resetZoom() {{
            if (chart) {{
                chart.resetZoom();
                document.getElementById('resetZoom').style.display = 'none';
            }}
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
