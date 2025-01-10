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
                case 'navigate':
                    navigateToUrl(data.url);
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

// Navigation functions
function navigateToUrl(url)
{
    const urlInput = document.getElementById('url-input');
    urlInput.value = url;

    window.ipc.postMessage({
        type: 'navigate',
        url: url
    });
}

// Event listeners
document.getElementById('url-input').addEventListener('keypress', (e) =>
{
    if (e.key === 'Enter')
    {
        let url = e.target.value.trim();
        if (!url.startsWith('http://') && !url.startsWith('https://'))
        {
            url = 'https://' + url;
        }
        navigateToUrl(url);
    }
});

document.getElementById('back-button').addEventListener('click', () =>
{
    window.ipc.postMessage({
        type: 'navigate_back'
    });
});

document.getElementById('forward-button').addEventListener('click', () =>
{
    window.ipc.postMessage({
        type: 'navigate_forward'
    });
});

document.getElementById('reload-button').addEventListener('click', () =>
{
    window.ipc.postMessage({
        type: 'reload'
    });
}); 
