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
    const newTab = document.getElementById('new-tab');
    const tab = createTabElement(id, title, url);

    tabBar.insertBefore(tab, newTab);
    tabs.set(id, { title, url });

    if (activeTabId === null) {
        setActiveTab(id);
    }
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
        tabs.delete(id);

        // If we removed the active tab, activate another one
        if (activeTabId === id) {
            const remainingTabs = Array.from(tabs.keys());
            if (remainingTabs.length > 0) {
                setActiveTab(remainingTabs[0]);
            } else {
                activeTabId = null;
            }
        }
    }
}

function setActiveTab(id) {
    // Remove active class from current active tab
    const currentActive = document.querySelector('.tab.active');
    if (currentActive) {
        currentActive.classList.remove('active');
    }

    // Add active class to new active tab
    const newActive = document.querySelector(`.tab[data-id="${id}"]`);
    if (newActive) {
        newActive.classList.add('active');
        activeTabId = id;
    }
}

// Event handlers that will send messages to Rust
function createNewTab() {
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
