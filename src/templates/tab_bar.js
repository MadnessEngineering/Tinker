class TabManager {
    constructor() {
        this.tabs = new Map();
        this.activeTabId = null;
        
        // Platform-specific styling
        this.isPlatformMac = navigator.platform.toLowerCase().includes('mac');
        if (this.isPlatformMac) {
            document.body.classList.add('platform-mac');
        }
    }

    createTab(id, url = 'about:blank', title = 'New Tab') {
        const tab = document.createElement('div');
        tab.className = 'tab';
        tab.dataset.tabId = id;
        
        // Platform-specific tab styling
        if (this.isPlatformMac) {
            tab.classList.add('mac-style-tab');
        }

        const titleSpan = document.createElement('span');
        titleSpan.className = 'tab-title';
        titleSpan.textContent = title;

        const closeButton = document.createElement('span');
        closeButton.className = 'tab-close';
        closeButton.textContent = '×';
        closeButton.onclick = (e) =>
        {
            e.stopPropagation();
            window.ipc.postMessage({
                type: 'close_tab',
                id: parseInt(id)
            });
        };

        tab.appendChild(titleSpan);
        tab.appendChild(closeButton);

        tab.onclick = () =>
        {
            window.ipc.postMessage({
                type: 'switch_tab',
                id: parseInt(id)
            });
        };

        const newTabButton = document.getElementById('new-tab-button');
        document.getElementById('tab-bar').insertBefore(tab, newTabButton);
        this.tabs.set(id, tab);

        return tab;
    }
}

// Initialize tab manager
window.tabManager = new TabManager();

// IPC setup
window.ipc = {
    postMessage: (msg) => window.ipc.external.invoke(JSON.stringify(msg)),
    handleMessage: (msg) =>
    {
        console.log('Message from Rust:', msg);
        try
        {
            const data = JSON.parse(msg);
            switch (data.type)
            {
                case 'updateUrl':
                    updateTabUrl(data.id, data.url);
                    break;
                case 'updateTitle':
                    updateTabTitle(data.id, data.title);
                    break;
                default:
                    console.warn('Unknown message type:', data.type);
            }
        } catch (e)
        {
            console.error('Failed to parse message:', e);
        }
    }
};

function updateTabUrl(id, url)
{
    const tab = window.tabs.get(id);
    if (tab) {
        tab.dataset.url = url;
    }
}

function updateTabTitle(id, title)
{
    const tab = window.tabs.get(id);
    if (tab) {
        const titleSpan = tab.querySelector('.tab-title');
        if (titleSpan)
        {
            titleSpan.textContent = title;
        }
    }
}

function setActiveTab(id) {
    window.tabs.forEach((tab) =>
    {
        tab.classList.remove('active');
    });
    const tab = window.tabs.get(id);
    if (tab)
    {
        tab.classList.add('active');
    }
}

// Event listeners
document.getElementById('new-tab-button').onclick = () =>
{
    window.ipc.postMessage({
        type: 'create_tab',
        url: 'about:blank'
    });
};
