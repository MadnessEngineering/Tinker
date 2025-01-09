// Store tab data
const tabs = new Map();
let activeTabId = null;

function createTabElement(id, title, url) {
    const tab = document.createElement('div');
    tab.className = 'tab';
    tab.setAttribute('data-tab-id', id);
    tab.setAttribute('data-url', url);

    const titleSpan = document.createElement('span');
    titleSpan.className = 'tab-title';
    titleSpan.textContent = title;
    tab.appendChild(titleSpan);

    const closeButton = document.createElement('div');
    closeButton.className = 'tab-close';
    closeButton.innerHTML = '×';
    closeButton.onclick = (e) =>
    {
        e.stopPropagation();
        window.ipc.postMessage(JSON.stringify({
            type: 'close',
            id: parseInt(id)
        }));
    };
    tab.appendChild(closeButton);

    tab.onclick = () =>
    {
        window.ipc.postMessage(JSON.stringify({
            type: 'switch',
            id: parseInt(id)
        }));
    };

    return tab;
}

function addTab(id, title, url)
{
    const tabBar = document.getElementById('tab-bar');
    const newTabButton = document.getElementById('new-tab');
    const tab = createTabElement(id, title, url);
    tabBar.insertBefore(tab, newTabButton);
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
    const tab = document.querySelector(`[data-tab-id="${id}"]`);
    if (tab) {
        tab.remove();
    }
}

function setActiveTab(id) {
    const tabs = document.querySelectorAll('.tab');
    tabs.forEach(tab =>
    {
        if (tab.getAttribute('data-tab-id') === id.toString())
        {
            tab.classList.add('active');
        } else
        {
            tab.classList.remove('active');
        }
    });
}

document.getElementById('new-tab').onclick = () =>
{
    window.ipc.postMessage(JSON.stringify({
        type: 'create',
        url: 'about:blank'
    }));
};
