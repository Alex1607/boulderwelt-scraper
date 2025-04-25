use serde_json::Value;

pub fn get_time_averages_html(data: Value) -> String {
    let data_str = serde_json::to_string(&data).unwrap();
    format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Crowd Level Time Averages</title>
    <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
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
        .chart-container {{
            height: 500px;
            margin-top: 20px;
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
            <a href="/graph?url=all&days=1&offset=0" class="nav-link">Live Graph</a>
            <a href="/time-averages-view" class="nav-link active">Time Averages</a>
        </div>

        <h1>Crowd Level Time Averages</h1>

        <div class="controls">
            <div>
                <label for="gymSelect">Gym Location:</label>
                <select id="gymSelect" onchange="updateChart()">
                    <option value="all">All Gyms</option>
                </select>
            </div>
            <div>
                <label for="daySelect">Day of Week:</label>
                <select id="daySelect" onchange="updateChart()">
                    <option value="all">All Days</option>
                    <option value="1">Monday</option>
                    <option value="2">Tuesday</option>
                    <option value="3">Wednesday</option>
                    <option value="4">Thursday</option>
                    <option value="5">Friday</option>
                    <option value="6">Saturday</option>
                    <option value="0">Sunday</option>
                </select>
            </div>
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
                <canvas id="averagesChart"></canvas>
            </div>
        </div>
    </div>

    <script>
        const rawData = {0};
        const weekdays = ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"];
        const displayWeekdays = ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"];
        let chart = null;

        // Color management for consistent gym colors
        const gymColors = new Map();
        const baseColors = [
            '#d92626', // Red
            '#26d959', // Green
            '#8b26d9', // Purple
            '#4BC0C0', // Teal
            '#FF9F40', // Orange
            '#EC932F', // Dark Orange
            '#5DA5DA', // Light Blue
            '#FAA43A'  // Light Orange
        ];
        let nextColorIndex = 0;

        function getGymColor(gym) {{
            if (!gymColors.has(gym)) {{
                gymColors.set(gym, baseColors[nextColorIndex % baseColors.length]);
                nextColorIndex++;
            }}
            return gymColors.get(gym);
        }}

        function adjustColorBrightness(color, factor) {{
            // Convert hex to RGB
            const r = parseInt(color.slice(1, 3), 16);
            const g = parseInt(color.slice(3, 5), 16);
            const b = parseInt(color.slice(5, 7), 16);

            // Adjust brightness
            const adjustedR = Math.min(255, Math.max(0, Math.round(r * factor)));
            const adjustedG = Math.min(255, Math.max(0, Math.round(g * factor)));
            const adjustedB = Math.min(255, Math.max(0, Math.round(b * factor)));

            // Convert back to hex
            return '#' + 
                adjustedR.toString(16).padStart(2, '0') +
                adjustedG.toString(16).padStart(2, '0') +
                adjustedB.toString(16).padStart(2, '0');
        }}

        function getColorForGymAndDay(gym, day) {{
            const baseColor = getGymColor(gym);
            const currentSelectedDay = daySelect.value;
            if (currentSelectedDay === 'all') {{
                // For "all days" view, use brightness to distinguish days
                const dayIndex = displayWeekdays.indexOf(day);
                const brightnessFactor = 0.6 + (dayIndex * 0.1); // Vary from 60% to 130% brightness
                return adjustColorBrightness(baseColor, brightnessFactor);
            }}
            return baseColor;
        }}

        // Function to get URL parameters
        function getUrlParams() {{
            const params = new URLSearchParams(window.location.search);
            return {{
                gym: params.get('gym') || 'all',
                day: params.get('day') || 'all'
            }};
        }}

        // Function to update URL parameters
        function updateUrlParams(gym, day) {{
            const params = new URLSearchParams(window.location.search);
            params.set('gym', gym);
            params.set('day', day);
            const newUrl = window.location.pathname + '?' + params.toString();
            window.history.pushState({{}},'', newUrl);
        }}

        // Get timezone offset in hours
        const timezoneOffset = new Date().getTimezoneOffset() / 60;

        // Convert UTC hour to local hour
        function utcToLocal(hour) {{
            let localHour = hour - timezoneOffset;
            if (localHour < 0) localHour += 24;
            if (localHour >= 24) localHour -= 24;
            return localHour;
        }}

        // Show loading indicator
        function showLoading() {{
            document.getElementById('loadingOverlay').style.visibility = 'visible';
        }}

        // Hide loading indicator
        function hideLoading() {{
            document.getElementById('loadingOverlay').style.visibility = 'hidden';
        }}

        // Populate gym select and set initial values from URL
        const gymSelect = document.getElementById('gymSelect');
        Object.keys(rawData.data).forEach(gym => {{
            const option = document.createElement('option');
            option.value = gym;
            option.textContent = gym;
            gymSelect.appendChild(option);
        }});

        // Set initial values from URL parameters
        const urlParams = getUrlParams();
        gymSelect.value = urlParams.gym;
        daySelect.value = urlParams.day;

        function getChartData() {{
            const selectedGym = gymSelect.value;
            const selectedDay = daySelect.value;
            const datasets = [];

            if (selectedGym === 'all') {{
                // Show all gyms
                Object.entries(rawData.data).forEach(([gym, gymData]) => {{
                    if (selectedDay === 'all') {{
                        // Show all days for each gym
                        const sortedDays = Object.entries(gymData).sort((a, b) => {{
                            const dayA = displayWeekdays.indexOf(a[0]);
                            const dayB = displayWeekdays.indexOf(b[0]);
                            return dayA - dayB;
                        }});
                        sortedDays.forEach(([day, dayData]) => {{
                            const data = new Array(24).fill(null);
                            Object.entries(dayData).forEach(([hour, value]) => {{
                                const localHour = utcToLocal(parseInt(hour));
                                data[localHour] = value.average;
                            }});
                            datasets.push({{
                                label: `${{gym}} - ${{day}}`,
                                data: data,
                                borderColor: getColorForGymAndDay(gym, day),
                                fill: false,
                                tension: 0.4,
                                segment: {{
                                    borderColor: ctx => ctx.p0.skip || ctx.p1.skip ? 'transparent' : undefined,
                                }},
                                spanGaps: false
                            }});
                        }});
                    }} else {{
                        // Show only selected day for each gym
                        const day = weekdays[parseInt(selectedDay)];
                        if (gymData[day]) {{
                            const data = new Array(24).fill(null);
                            Object.entries(gymData[day]).forEach(([hour, value]) => {{
                                const localHour = utcToLocal(parseInt(hour));
                                data[localHour] = value.average;
                            }});
                            datasets.push({{
                                label: gym,
                                data: data,
                                borderColor: getGymColor(gym),
                                fill: false,
                                tension: 0.4,
                                segment: {{
                                    borderColor: ctx => ctx.p0.skip || ctx.p1.skip ? 'transparent' : undefined,
                                }},
                                spanGaps: false
                            }});
                        }}
                    }}
                }});
            }} else {{
                // Show only selected gym
                const gymData = rawData.data[selectedGym];
                if (gymData) {{
                    if (selectedDay === 'all') {{
                        // Show all days for selected gym
                        const sortedDays = Object.entries(gymData).sort((a, b) => {{
                            const dayA = displayWeekdays.indexOf(a[0]);
                            const dayB = displayWeekdays.indexOf(b[0]);
                            return dayA - dayB;
                        }});
                        const baseColor = getGymColor(selectedGym);
                        sortedDays.forEach(([day, dayData], index) => {{
                            const data = new Array(24).fill(null);
                            Object.entries(dayData).forEach(([hour, value]) => {{
                                const localHour = utcToLocal(parseInt(hour));
                                data[localHour] = value.average;
                            }});
                            datasets.push({{
                                label: day,
                                data: data,
                                borderColor: getColorForGymAndDay(selectedGym, day),
                                fill: false,
                                tension: 0.4,
                                segment: {{
                                    borderColor: ctx => ctx.p0.skip || ctx.p1.skip ? 'transparent' : undefined,
                                }},
                                spanGaps: false
                            }});
                        }});
                    }} else {{
                        // Show only selected day for selected gym
                        const day = weekdays[parseInt(selectedDay)];
                        if (gymData[day]) {{
                            const data = new Array(24).fill(null);
                            Object.entries(gymData[day]).forEach(([hour, value]) => {{
                                const localHour = utcToLocal(parseInt(hour));
                                data[localHour] = value.average;
                            }});
                            datasets.push({{
                                label: day,
                                data: data,
                                borderColor: getGymColor(selectedGym),
                                fill: false,
                                tension: 0.4,
                                segment: {{
                                    borderColor: ctx => ctx.p0.skip || ctx.p1.skip ? 'transparent' : undefined,
                                }},
                                spanGaps: false
                            }});
                        }}
                    }}
                }}
            }}

            // Format hours in 24-hour format with leading zeros
            const hourLabels = Array.from({{length: 24}}, (_, i) => 
                i.toString().padStart(2, '0') + ':00'
            );

            return {{
                labels: hourLabels,
                datasets: datasets
            }};
        }}

        function updateChart() {{
            showLoading();

            // Hide reset zoom button when loading new data
            document.getElementById('resetZoom').style.display = 'none';

            // Update URL parameters when chart is updated
            updateUrlParams(gymSelect.value, daySelect.value);

            if (chart) {{
                chart.destroy();
            }}

            const ctx = document.getElementById('averagesChart').getContext('2d');
            const data = getChartData();

            // Hide loading as soon as data is ready
            hideLoading();

            chart = new Chart(ctx, {{
                type: 'line',
                data: data,
                options: {{
                    responsive: true,
                    maintainAspectRatio: false,
                    scales: {{
                        y: {{
                            beginAtZero: true,
                            max: 100,
                            title: {{
                                display: true,
                                text: 'Average Crowd Level (%)'
                            }}
                        }},
                        x: {{
                            title: {{
                                display: true,
                                text: 'Hour of Day'
                            }},
                            ticks: {{
                                maxRotation: 0,
                                autoSkip: false
                            }}
                        }}
                    }},
                    plugins: {{
                        title: {{
                            display: true,
                            text: 'Average Crowd Levels by Hour',
                            font: {{
                                size: 16
                            }}
                        }},
                        tooltip: {{
                            callbacks: {{
                                label: function(context) {{
                                    const label = context.dataset.label || '';
                                    const value = context.parsed.y;
                                    return `${{label}}: ${{value.toFixed(1)}}%`;
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

        // Reset zoom to original scale
        function resetZoom() {{
            if (chart) {{
                chart.resetZoom();
                document.getElementById('resetZoom').style.display = 'none';
            }}
        }}

        // Initial chart
        updateChart();
    </script>
</body>
</html>"##,
        data_str
    )
} 
