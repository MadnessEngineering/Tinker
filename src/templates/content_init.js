// Injected into every page loaded in the content view. Reports page lifecycle
// (title + load) back to the browser engine so the chrome (tab labels, address
// bar) can stay in sync. Kept deliberately minimal — this runs in the context
// of arbitrary pages under test, so it must not throw or pollute the page.
(function () {
    function send(msg) {
        try {
            window.ipc.postMessage(JSON.stringify(msg));
        } catch (e) {
            // window.ipc may be unavailable in some contexts; ignore.
        }
    }

    function report() {
        send({ type: 'pageLoaded', url: location.href });
        send({ type: 'titleChanged', title: document.title });
    }

    if (document.readyState === 'complete') {
        report();
    }
    window.addEventListener('load', report);
})();
