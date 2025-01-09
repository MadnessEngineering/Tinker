// Store tab data
const tabs = new Map();
let activeTabId = null;

function createTabElement(id, title, url) {
    const tab = document.createElement('div');
    tab.className = 'tab';
    tab.dataset.id = id;
    tab.innerHTML = `
        <div class="tab-title" title="${url}">${title}</div>
        <div class="tab-close" onclick="closeTab(${id})">&times;</div>
    `;
    tab.onclick = (e) => {
        if (!e.target.classList.contains('tab-close')) {
            switchTab(id);
        }
    };
    return tab;
}

function addTab(id, title, url) {
    const tabBar = document.getElementById('tab-bar');
    const newTabButton = document.getElementById('new-tab');

    // Create tab element
    const tab = document.createElement('div');
    tab.className = 'tab';
    tab.setAttribute('data-id', id);
    tab.onclick = () => switchTab(id);

    // Create title element
    const titleSpan = document.createElement('span');
    titleSpan.className = 'tab-title';
    titleSpan.textContent = title;
    tab.appendChild(titleSpan);

    // Create close button
    const closeButton = document.createElement('div');
    closeButton.className = 'tab-close';
    closeButton.innerHTML = '×';
    closeButton.onclick = (e) =>
    {
        e.stopPropagation();
        closeTab(id);
    };
    tab.appendChild(closeButton);

    // Insert before the new tab button
    tabBar.insertBefore(tab, newTabButton);

    // Set as active
    setActiveTab(id);
}

function updateTab(id, title, url) {
    const tab = document.querySelector(`.tab[data-id="${id}"]`);
    if (tab) {
        const titleElement = tab.querySelector('.tab-title');
        titleElement.textContent = title;
        titleElement.title = url;
        tabs.set(id, { title, url });
    }
}

function removeTab(id) {
    const tab = document.querySelector(`.tab[data-id="${id}"]`);
    if (tab) {
        tab.remove();
    }
}

function setActiveTab(id) {
    // Remove active class from all tabs
    document.querySelectorAll('.tab').forEach(tab =>
    {
        tab.classList.remove('active');
    });

    // Add active class to selected tab
    const tab = document.querySelector(`.tab[data-id="${id}"]`);
    if (tab)
    {
        tab.classList.add('active');
    }
}

// Event handlers that will send messages to Rust
function createNewTab() {
    // Send message to Rust to create a new tab
    window.ipc.postMessage(JSON.stringify({
        type: 'TabCreated',
        url: 'https://github.com/DanEdens/Tinker'
    }));
}

function closeTab(id) {
    window.ipc.postMessage(JSON.stringify({
        type: 'TabClosed',
        id: id
    }));
}

function switchTab(id) {
    window.ipc.postMessage(JSON.stringify({
        type: 'TabSwitched',
        id: id
    }));
}
