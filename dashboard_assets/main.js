const cpuUsageElem = document.getElementById('cpu-usage');
const memoryUsageElem = document.getElementById('memory-usage');
const networkUsageElem = document.getElementById('network-usage');
const ipListElem = document.getElementById('ip-list');
const networkChartCanvas = document.getElementById('networkChart');

// Function to format bytes into human-readable units
function formatBytes(bytes, decimals = 2) {
    if (bytes === 0) return '0 Bytes';

    const k = 1024;
    const dm = decimals < 0 ? 0 : decimals;
    const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB', 'PB', 'EB', 'ZB', 'YB'];

    const i = Math.floor(Math.log(bytes) / Math.log(k));

    return parseFloat((bytes / Math.pow(k, i)).toFixed(dm)) + ' ' + sizes[i];
}

const socket = new WebSocket(`wss://${window.location.host}/ws`);

socket.onmessage = (event) => {
    const data = JSON.parse(event.data);

    // Update metrics
    cpuUsageElem.textContent = `${data.cpu_usage.toFixed(2)} %`;
    memoryUsageElem.textContent = `${(data.memory_usage / 1024 / 1024).toFixed(2)} MB`;
    networkUsageElem.textContent = formatBytes(data.network_current_rate) + '/s'; // Changed to network_current_rate and added /s

    // Update IP list
    if (data.connected_ips && data.connected_ips.length > 0) {
        ipListElem.innerHTML = ''; // Clear the list
        data.connected_ips.forEach(ip => {
            const li = document.createElement('li');
            li.textContent = ip;
            ipListElem.appendChild(li);
        });
    } else {
        ipListElem.innerHTML = '<li>No connections yet.</li>';
    }
};

socket.onopen = () => {
    console.log('WebSocket connection established.');
};

socket.onclose = () => {
    console.log('WebSocket connection closed. Attempting to reconnect...');
    // Optional: implement a reconnect logic here
};

socket.onerror = (error) => {
    console.error('WebSocket error:', error);
};

// Network Usage Chart
let networkChart;

async function fetchNetworkHistory() {
    try {
        const response = await fetch('/api/network_history');
        const history = await response.json();

        const labels = history.map(entry => entry.date);
        const data = history.map(entry => entry.usage);

        if (networkChart) {
            networkChart.data.labels = labels;
            networkChart.data.datasets[0].data = data;
            networkChart.update();
        } else {
            networkChart = new Chart(networkChartCanvas, {
                type: 'line',
                data: {
                    labels: labels,
                    datasets: [{
                        label: 'Daily Network Usage',
                        data: data,
                        borderColor: 'rgb(75, 192, 192)',
                        tension: 0.1
                    }]
                },
                options: {
                    responsive: true,
                    scales: {
                        y: {
                            beginAtZero: true,
                            ticks: {
                                callback: function(value) {
                                    return formatBytes(value) + '/s';
                                }
                            }
                        }
                    }
                }
            });
        }
    } catch (error) {
        console.error('Error fetching network history:', error);
    }
}

// Fetch history initially and then every 5 seconds
fetchNetworkHistory();
setInterval(fetchNetworkHistory, 5000);