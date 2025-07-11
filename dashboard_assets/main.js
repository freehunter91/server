const cpuUsageElem = document.getElementById('cpu-usage');
const memoryUsageElem = document.getElementById('memory-usage');
const networkUsageElem = document.getElementById('network-usage');
const ipListElem = document.getElementById('ip-list');

const socket = new WebSocket(`ws://${window.location.host}/ws`);

socket.onmessage = (event) => {
    const data = JSON.parse(event.data);

    // Update metrics
    cpuUsageElem.textContent = `${data.cpu_usage.toFixed(2)} %`;
    memoryUsageElem.textContent = `${(data.memory_usage / 1024 / 1024).toFixed(2)} MB`;
    networkUsageElem.textContent = `${(data.network_usage / 1024).toFixed(2)} KB`;

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