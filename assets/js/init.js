// WebView initialization script

// Initialize IPC communication
window.ipc = {
    postMessage: (message) => {
        window.external.invoke(JSON.stringify(message));
    }
};

// Add custom styles
const style = document.createElement('style');
style.textContent = `
    ::-webkit-scrollbar {
        width: 12px;
        height: 12px;
    }
    ::-webkit-scrollbar-track {
        background: #f1f1f1;
    }
    ::-webkit-scrollbar-thumb {
        background: #888;
        border-radius: 6px;
    }
    ::-webkit-scrollbar-thumb:hover {
        background: #555;
    }
`;
document.head.appendChild(style);

// Listen for navigation events
document.addEventListener('click', (e) => {
    const link = e.target.closest('a');
    if (link && link.href) {
        window.ipc.postMessage({
            type: 'navigation',
            url: link.href
        });
    }
});

// Report page load complete
window.addEventListener('load', () => {
    window.ipc.postMessage({
        type: 'loaded',
        title: document.title,
        url: window.location.href
    });
});

// Monitor page events
document.addEventListener('DOMContentLoaded', () =>
{
    // Send page load event
    window.ipc.postMessage({
        type: 'pageLoaded',
        url: window.location.href
    });

    // Monitor title changes
    const observer = new MutationObserver(() =>
    {
        window.ipc.postMessage({
            type: 'titleChanged',
            title: document.title
        });
    });

    observer.observe(
        document.querySelector('title'),
        { childList: true, characterData: true, subtree: true }
    );
});

// Handle navigation events
window.addEventListener('popstate', () =>
{
    window.ipc.postMessage({
        type: 'navigation',
        url: window.location.href
    });
}); 
