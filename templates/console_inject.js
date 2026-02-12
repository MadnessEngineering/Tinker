(function() {
    'use strict';

    // Store original console methods
    const originalConsole = {
        log: console.log,
        info: console.info,
        warn: console.warn,
        error: console.error,
        debug: console.debug
    };

    // Buffer for storing console messages
    if (!window.__tinkerConsoleBuffer) {
        window.__tinkerConsoleBuffer = [];
    }

    // Function to get caller info from stack trace
    function getCallerInfo() {
        try {
            const stack = new Error().stack;
            if (!stack) return null;

            const lines = stack.split('\n');
            // Skip first 3 lines (Error, getCallerInfo, sendConsoleMessage)
            const callerLine = lines[4] || lines[3] || lines[2];

            // Try to match different stack trace formats
            // Chrome/Edge: "at functionName (url:line:column)"
            // Firefox: "functionName@url:line:column"
            const chromeMatch = callerLine.match(/at\s+.*?\s*\(?([^)]+):(\d+):(\d+)\)?/);
            const firefoxMatch = callerLine.match(/@(.+):(\d+):(\d+)/);

            const match = chromeMatch || firefoxMatch;
            if (match) {
                return {
                    url: match[1],
                    line: parseInt(match[2], 10),
                    column: parseInt(match[3], 10)
                };
            }
        } catch (e) {
            // Silently fail if stack parsing doesn't work
        }
        return null;
    }

    // Function to serialize arguments
    function serializeArgs(args) {
        const serialized = [];
        for (let i = 0; i < args.length; i++) {
            const arg = args[i];
            try {
                if (typeof arg === 'object' && arg !== null) {
                    // Try to stringify objects
                    serialized.push(JSON.parse(JSON.stringify(arg)));
                } else {
                    serialized.push(arg);
                }
            } catch (e) {
                // If serialization fails, convert to string
                serialized.push(String(arg));
            }
        }
        return serialized;
    }

    // Function to format message
    function formatMessage(args) {
        return args.map(arg => {
            if (typeof arg === 'object' && arg !== null) {
                try {
                    return JSON.stringify(arg, null, 2);
                } catch (e) {
                    return String(arg);
                }
            }
            return String(arg);
        }).join(' ');
    }

    // Function to send console message to Rust
    function sendConsoleMessage(level, args) {
        const message = {
            level: level,
            message: formatMessage(args),
            timestamp: Date.now(),
            args: serializeArgs(args),
            source: getCallerInfo(),
            stack_trace: null
        };

        // Store in buffer (keep last 100 for quick access)
        window.__tinkerConsoleBuffer.push(message);
        if (window.__tinkerConsoleBuffer.length > 100) {
            window.__tinkerConsoleBuffer.shift();
        }

        // Send via callback if available
        if (window.__tinkerConsoleCallback) {
            try {
                window.__tinkerConsoleCallback(message);
            } catch (e) {
                // Don't break if callback fails
                originalConsole.error('Tinker console callback error:', e);
            }
        }
    }

    // Override console methods
    console.log = function(...args) {
        originalConsole.log.apply(console, args);
        sendConsoleMessage('log', args);
    };

    console.info = function(...args) {
        originalConsole.info.apply(console, args);
        sendConsoleMessage('info', args);
    };

    console.warn = function(...args) {
        originalConsole.warn.apply(console, args);
        sendConsoleMessage('warn', args);
    };

    console.error = function(...args) {
        originalConsole.error.apply(console, args);
        sendConsoleMessage('error', args);
    };

    console.debug = function(...args) {
        originalConsole.debug.apply(console, args);
        sendConsoleMessage('debug', args);
    };

    // Capture unhandled errors
    window.addEventListener('error', function(event) {
        const error = {
            message: event.message || 'Unknown error',
            stack: event.error && event.error.stack ? event.error.stack : '',
            filename: event.filename || '',
            line: event.lineno || 0,
            column: event.colno || 0,
            timestamp: Date.now(),
            error_type: event.error && event.error.name ? event.error.name : 'Error'
        };

        if (window.__tinkerErrorCallback) {
            try {
                window.__tinkerErrorCallback(error);
            } catch (e) {
                originalConsole.error('Tinker error callback error:', e);
            }
        }
    }, true);

    // Capture unhandled promise rejections
    window.addEventListener('unhandledrejection', function(event) {
        const reason = event.reason;
        const error = {
            message: reason ? String(reason) : 'Unhandled Promise Rejection',
            stack: reason && reason.stack ? reason.stack : '',
            filename: '',
            line: 0,
            column: 0,
            timestamp: Date.now(),
            error_type: 'UnhandledRejection'
        };

        if (window.__tinkerErrorCallback) {
            try {
                window.__tinkerErrorCallback(error);
            } catch (e) {
                originalConsole.error('Tinker error callback error:', e);
            }
        }
    }, true);

    // Store reference to original methods for potential restoration
    window.__tinkerOriginalConsole = originalConsole;

    // Return success message
    return JSON.stringify({
        success: true,
        message: 'Console interceptor installed successfully',
        timestamp: Date.now()
    });
})();
