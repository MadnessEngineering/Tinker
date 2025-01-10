// Window control functionality
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
                case 'navigate':
                    window.location.href = data.url;
                    break;
                case 'updateTitle':
                    document.title = data.title;
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

// Window control functions
function minimizeWindow()
{
    window.ipc.postMessage(JSON.stringify({
        type: 'WindowControl',
        action: 'minimize'
    }));
}

function maximizeWindow()
{
    window.ipc.postMessage(JSON.stringify({
        type: 'WindowControl',
        action: 'maximize'
    }));
}

function closeWindow()
{
    window.ipc.postMessage(JSON.stringify({
        type: 'WindowControl',
        action: 'close'
    }));
}

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

    if (document.querySelector('title'))
    {
        observer.observe(
            document.querySelector('title'),
            { childList: true, characterData: true, subtree: true }
        );
    }
});

// Handle navigation events
window.addEventListener('popstate', () =>
{
    window.ipc.postMessage({
        type: 'navigation',
        url: window.location.href
    });
}); 
