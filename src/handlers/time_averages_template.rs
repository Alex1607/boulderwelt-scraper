use serde_json::Value;

pub fn get_time_averages_html(data: Value) -> String {
    let html = format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Crowd Level Time Averages</title>
    <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
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
        .controls {{
            display: flex;
            justify-content: space-between;
            margin-bottom: 20px;
            gap: 10px;
        }}
        select {{
            padding: 8px 12px;
            border-radius: 4px;
            border: 1px solid #ddd;
        }}
        .chart-container {{
            height: 500px;
            margin-top: 20px;
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
        </div>

        <div class="chart-container">
            <canvas id="averagesChart"></canvas>
        </div>
    </div>

    <script>
        const rawData = {data};
        const weekdays = ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"];
        const displayWeekdays = ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"];
        let chart = null;
        
        // Populate gym select
        const gymSelect = document.getElementById('gymSelect');
        Object.keys(rawData.data).forEach(gym => {{
            const option = document.createElement('option');
            option.value = gym;
            option.textContent = gym;
            gymSelect.appendChild(option);
        }});

        function getChartData() {{
            const selectedGym = gymSelect.value;
            const selectedDay = daySelect.value;
            const datasets = [];
            const hours = Array.from({{length: 24}}, (_, i) => i);
            
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
                                data[parseInt(hour)] = value.average;
                            }});
                            datasets.push({{
                                label: `${{gym}} - ${{day}}`,
                                data: data,
                                borderColor: getRandomColor(),
                                fill: false,
                                tension: 0.4
                            }});
                        }});
                    }} else {{
                        // Show only selected day for each gym
                        const day = weekdays[parseInt(selectedDay)];
                        if (gymData[day]) {{
                            const data = new Array(24).fill(null);
                            Object.entries(gymData[day]).forEach(([hour, value]) => {{
                                data[parseInt(hour)] = value.average;
                            }});
                            datasets.push({{
                                label: gym,
                                data: data,
                                borderColor: getRandomColor(),
                                fill: false,
                                tension: 0.4
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
                        sortedDays.forEach(([day, dayData]) => {{
                            const data = new Array(24).fill(null);
                            Object.entries(dayData).forEach(([hour, value]) => {{
                                data[parseInt(hour)] = value.average;
                            }});
                            datasets.push({{
                                label: day,
                                data: data,
                                borderColor: getRandomColor(),
                                fill: false,
                                tension: 0.4
                            }});
                        }});
                    }} else {{
                        // Show only selected day for selected gym
                        const day = weekdays[parseInt(selectedDay)];
                        if (gymData[day]) {{
                            const data = new Array(24).fill(null);
                            Object.entries(gymData[day]).forEach(([hour, value]) => {{
                                data[parseInt(hour)] = value.average;
                            }});
                            datasets.push({{
                                label: day,
                                data: data,
                                borderColor: getRandomColor(),
                                fill: false,
                                tension: 0.4
                            }});
                        }}
                    }}
                }}
            }}

            return {{
                labels: hours.map(h => `${{h}}:00`),
                datasets: datasets
            }};
        }}

        function getRandomColor() {{
            const letters = '0123456789ABCDEF';
            let color = '#';
            for (let i = 0; i < 6; i++) {{
                color += letters[Math.floor(Math.random() * 16)];
            }}
            return color;
        }}

        function updateChart() {{
            if (chart) {{
                chart.destroy();
            }}

            const ctx = document.getElementById('averagesChart').getContext('2d');
            const data = getChartData();

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
                        }}
                    }}
                }}
            }});
        }}

        // Initial chart
        updateChart();
    </script>
</body>
</html>"##,
        data = serde_json::to_string(&data).unwrap()
    );
    
    html
} 