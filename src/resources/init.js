// Initialize UI styling and functionality
document.addEventListener('DOMContentLoaded', () => {
  // Add dark theme styles
  const style = document.createElement('style');
  style.textContent = `
    :root {
      --bg-color: #1a1a1a;
      --text-color: #ffffff;
      --toolbar-bg: #2d2d2d;
      --button-hover: #404040;
      --border-color: #404040;
      --input-bg: #333333;
    }
    
    body {
      margin: 0;
      padding: 0;
      background-color: var(--bg-color);
      color: var(--text-color);
      font-family: system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
    }

    #toolbar {
      position: fixed;
      top: 0;
      left: 0;
      right: 0;
      height: 40px;
      background-color: var(--toolbar-bg);
      display: flex;
      align-items: center;
      padding: 0 10px;
      gap: 5px;
      border-bottom: 1px solid var(--border-color);
    }

    .toolbar-button {
      background: none;
      border: none;
      color: var(--text-color);
      padding: 8px;
      border-radius: 4px;
      cursor: pointer;
      font-size: 16px;
    }

    .toolbar-button:hover {
      background-color: var(--button-hover);
    }

    #urlbar {
      flex: 1;
      background-color: var(--input-bg);
      border: 1px solid var(--border-color);
      border-radius: 4px;
      color: var(--text-color);
      padding: 6px 10px;
      margin: 0 5px;
      font-size: 14px;
    }

    #content {
      margin-top: 40px;
      height: calc(100vh - 40px);
      overflow: auto;
    }
  `;
  document.head.appendChild(style);

  // Create toolbar
  const toolbar = document.createElement('div');
  toolbar.id = 'toolbar';
  document.body.insertBefore(toolbar, document.body.firstChild);

  // Create navigation buttons
  const createButton = (text, onClick) => {
    const button = document.createElement('button');
    button.className = 'toolbar-button';
    button.textContent = text;
    button.onclick = onClick;
    return button;
  };

  toolbar.appendChild(createButton('←', () => window.tinker.goBack()));
  toolbar.appendChild(createButton('→', () => window.tinker.goForward()));
  toolbar.appendChild(createButton('⟳', () => window.tinker.reload()));

  // Create URL bar
  const urlbar = document.createElement('input');
  urlbar.id = 'urlbar';
  urlbar.type = 'text';
  urlbar.placeholder = 'Enter URL';
  urlbar.addEventListener('keypress', (e) => {
    if (e.key === 'Enter') {
      window.tinker.navigate(urlbar.value);
    }
  });
  toolbar.appendChild(urlbar);

  // Create content container
  const content = document.createElement('div');
  content.id = 'content';
  
  // Move existing body content to container
  while (document.body.firstChild !== toolbar) {
    content.appendChild(document.body.firstChild);
  }
  document.body.appendChild(content);
});

// Export API for native code
window.tinker = {
  updateUrl: (url) => {
    document.getElementById('urlbar').value = url;
  },
  updateTitle: (title) => {
    document.title = title;
  }
}; 