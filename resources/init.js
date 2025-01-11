// Initialize UI with dark theme and modern controls
document.body.style.margin = '0';
document.body.style.padding = '0';
document.body.style.backgroundColor = '#1e1e1e';
document.body.style.fontFamily = '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Oxygen-Sans, Ubuntu, Cantarell, "Helvetica Neue", sans-serif';

// Create toolbar
const toolbar = document.createElement('div');
toolbar.style.cssText = `
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  height: 40px;
  background: #2d2d2d;
  display: flex;
  align-items: center;
  padding: 0 10px;
  gap: 10px;
  border-bottom: 1px solid #3d3d3d;
`;

// Navigation buttons
const createButton = (text, onClick) => {
  const button = document.createElement('button');
  button.textContent = text;
  button.style.cssText = `
    background: #3d3d3d;
    border: none;
    color: #fff;
    padding: 5px 10px;
    border-radius: 4px;
    cursor: pointer;
    font-size: 14px;
  `;
  button.onclick = onClick;
  return button;
};

toolbar.appendChild(createButton('←', () => window.history.back()));
toolbar.appendChild(createButton('→', () => window.history.forward()));
toolbar.appendChild(createButton('⟳', () => window.location.reload()));

// URL bar
const urlBar = document.createElement('input');
urlBar.type = 'text';
urlBar.value = window.location.href;
urlBar.style.cssText = `
  flex: 1;
  background: #1e1e1e;
  border: 1px solid #3d3d3d;
  color: #fff;
  padding: 5px 10px;
  border-radius: 4px;
  font-size: 14px;
`;
urlBar.onkeydown = (e) => {
  if (e.key === 'Enter') {
    window.location.href = urlBar.value;
  }
};
toolbar.appendChild(urlBar);

// Content container
const content = document.createElement('div');
content.style.cssText = `
  margin-top: 40px;
  height: calc(100vh - 40px);
  overflow: auto;
`;

// Move all body content to container
while (document.body.firstChild) {
  content.appendChild(document.body.firstChild);
}

document.body.appendChild(toolbar);
document.body.appendChild(content);

// Export API for native code
window.tinker = {
  updateUrl: (url) => {
    urlBar.value = url;
  },
  updateTitle: (title) => {
    document.title = title;
  }
}; 