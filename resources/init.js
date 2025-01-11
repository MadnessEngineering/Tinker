// Tinker Browser UI Initialization
(function() {
    // Constants
    const TOOLBAR_HEIGHT = 40;
    const TAB_HEIGHT = 32;
    
    // Styles
    const styles = `
        :root {
            --toolbar-height: ${TOOLBAR_HEIGHT}px;
            --tab-height: ${TAB_HEIGHT}px;
            --primary-color: #2b2b2b;
            --secondary-color: #3c3c3c;
            --text-color: #e0e0e0;
            --border-color: #505050;
            --hover-color: #505050;
            --active-color: #707070;
        }

        body {
            margin: 0;
            padding: 0;
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
            background: var(--primary-color);
            color: var(--text-color);
            overflow: hidden;
        }

        #toolbar {
            position: fixed;
            top: 0;
            left: 0;
            right: 0;
            height: var(--toolbar-height);
            background: var(--primary-color);
            border-bottom: 1px solid var(--border-color);
            display: flex;
            align-items: center;
            padding: 0 8px;
            z-index: 1000;
        }

        #tab-bar {
            height: var(--tab-height);
            background: var(--secondary-color);
            display: flex;
            align-items: center;
            padding: 0 4px;
            overflow-x: auto;
            scrollbar-width: none;
        }

        #tab-bar::-webkit-scrollbar {
            display: none;
        }

        .tab {
            height: calc(var(--tab-height) - 4px);
            min-width: 120px;
            max-width: 200px;
            display: flex;
            align-items: center;
            padding: 0 12px;
            margin: 0 2px;
            border-radius: 4px;
            background: var(--primary-color);
            cursor: pointer;
            user-select: none;
            transition: background 0.2s;
            white-space: nowrap;
            overflow: hidden;
            text-overflow: ellipsis;
        }

        .tab:hover {
            background: var(--hover-color);
        }

        .tab.active {
            background: var(--active-color);
        }

        .tab-close {
            margin-left: 8px;
            padding: 2px;
            border-radius: 50%;
            opacity: 0.7;
            transition: opacity 0.2s;
        }

        .tab-close:hover {
            opacity: 1;
            background: rgba(255, 255, 255, 0.1);
        }

        #nav-controls {
            display: flex;
            align-items: center;
            gap: 8px;
            padding: 0 12px;
        }

        .nav-button {
            padding: 6px;
            border-radius: 4px;
            background: transparent;
            border: none;
            color: var(--text-color);
            cursor: pointer;
            transition: background 0.2s;
        }

        .nav-button:hover {
            background: var(--hover-color);
        }

        .nav-button:disabled {
            opacity: 0.5;
            cursor: not-allowed;
        }

        #url-bar {
            flex: 1;
            height: 28px;
            margin: 0 12px;
            padding: 0 8px;
            background: var(--secondary-color);
            border: 1px solid var(--border-color);
            border-radius: 4px;
            color: var(--text-color);
            font-size: 14px;
        }

        #url-bar:focus {
            outline: none;
            border-color: var(--active-color);
        }

        #content {
            position: fixed;
            top: var(--toolbar-height);
            left: 0;
            right: 0;
            bottom: 0;
            background: white;
        }
    `;

    // Create and inject styles
    const styleSheet = document.createElement('style');
    styleSheet.textContent = styles;
    document.head.appendChild(styleSheet);

    // Create toolbar structure
    const toolbar = document.createElement('div');
    toolbar.id = 'toolbar';
    toolbar.innerHTML = `
        <div id="tab-bar"></div>
        <div id="nav-controls">
            <button class="nav-button" id="back-button">←</button>
            <button class="nav-button" id="forward-button">→</button>
            <button class="nav-button" id="refresh-button">↻</button>
        </div>
        <input type="text" id="url-bar" placeholder="Enter URL">
        <div id="menu-controls">
            <button class="nav-button" id="menu-button">☰</button>
        </div>
    `;
    document.body.appendChild(toolbar);

    // Create content container
    const content = document.createElement('div');
    content.id = 'content';
    document.body.appendChild(content);

    // Initialize event listeners
    document.getElementById('back-button').addEventListener('click', () => {
        window.history.back();
    });

    document.getElementById('forward-button').addEventListener('click', () => {
        window.history.forward();
    });

    document.getElementById('refresh-button').addEventListener('click', () => {
        window.location.reload();
    });

    const urlBar = document.getElementById('url-bar');
    urlBar.addEventListener('keypress', (e) => {
        if (e.key === 'Enter') {
            const url = urlBar.value;
            if (url) {
                window.location.href = url.includes('://') ? url : `https://${url}`;
            }
        }
    });

    // Update URL bar when location changes
    const updateUrlBar = () => {
        urlBar.value = window.location.href;
    };
    window.addEventListener('load', updateUrlBar);
    window.addEventListener('popstate', updateUrlBar);

    // Export API for native code
    window.tinkerAPI = {
        updateToolbarHeight: (height) => {
            document.documentElement.style.setProperty('--toolbar-height', `${height}px`);
        },
        addTab: (id, title, isActive) => {
            const tab = document.createElement('div');
            tab.className = `tab${isActive ? ' active' : ''}`;
            tab.dataset.id = id;
            tab.innerHTML = `
                <span class="tab-title">${title || 'New Tab'}</span>
                <span class="tab-close">×</span>
            `;
            document.getElementById('tab-bar').appendChild(tab);
        },
        removeTab: (id) => {
            const tab = document.querySelector(`.tab[data-id="${id}"]`);
            if (tab) tab.remove();
        },
        activateTab: (id) => {
            document.querySelectorAll('.tab').forEach(tab => {
                tab.classList.toggle('active', tab.dataset.id === id);
            });
        }
    };
})(); 