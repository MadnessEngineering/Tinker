window.tabs = new Map();

// Send a message to Rust. wry injects a frozen window.ipc whose postMessage
// forwards a single string to the host (read via TryGetWebMessageAsString), so
// the payload must be stringified here. Do not reassign window.ipc (frozen).
function send(msg) {
    window.ipc.postMessage(JSON.stringify(msg));
}

// Tab management functions
function createTab(id, url = 'about:blank', title = 'New Tab')
{
    const tab = document.createElement('div');
    tab.className = 'tab';
    tab.dataset.tabId = id;
    tab.dataset.url = url;

    const titleSpan = document.createElement('span');
    titleSpan.className = 'tab-title';
    titleSpan.textContent = title;

    const closeButton = document.createElement('span');
    closeButton.className = 'tab-close';
    closeButton.textContent = '×';
    closeButton.onclick = (e) =>
    {
        e.stopPropagation();
        send({
            type: 'close',
            id: parseInt(id)
        });
    };

    tab.appendChild(titleSpan);
    tab.appendChild(closeButton);

    tab.onclick = () =>
    {
        send({
            type: 'switch',
            id: parseInt(id)
        });
    };

    const newTabButton = document.getElementById('new-tab-button');
    document.getElementById('tab-bar').insertBefore(tab, newTabButton);
    window.tabs.set(id, tab);

    return tab;
}

function removeTab(id)
{
    const tab = window.tabs.get(id);
    if (tab) {
        tab.remove();
        window.tabs.delete(id);
    }
}

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

// Address-bar helpers (Rust -> page)
function updateAddressBar(url)
{
    const input = document.getElementById('url-input');
    if (input && document.activeElement !== input) {
        input.value = url;
    }
}

function normalizeUrl(input)
{
    let url = input.trim();
    if (!url) return null;
    if (!/^https?:\/\//i.test(url)) {
        if (url.includes('.') && !url.includes(' ')) {
            url = 'https://' + url;
        } else {
            url = 'https://www.google.com/search?q=' + encodeURIComponent(url);
        }
    }
    return url;
}

// Event listeners
document.getElementById('new-tab-button').onclick = () =>
{
    send({
        type: 'create',
        url: 'about:blank'
    });
};

document.getElementById('url-input').addEventListener('keypress', (e) =>
{
    if (e.key === 'Enter') {
        const url = normalizeUrl(e.target.value);
        if (url) send({ type: 'navigate', url: url });
    }
});

document.getElementById('back-button').onclick = () => send({ type: 'back' });
document.getElementById('forward-button').onclick = () => send({ type: 'forward' });
document.getElementById('reload-button').onclick = () => send({ type: 'reload' });
