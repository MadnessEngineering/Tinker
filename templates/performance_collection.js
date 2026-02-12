// Performance metrics collection script
// This script collects comprehensive performance data from the browser

(function() {
    'use strict';

    const perfData = {
        coreWebVitals: {},
        navigationTiming: {},
        resourceTimings: [],
        memory: {},
        timestamp: Date.now()
    };

    // Collect Core Web Vitals
    function collectCoreWebVitals() {
        const vitals = {};

        // LCP (Largest Contentful Paint)
        if (window.PerformanceObserver && PerformanceObserver.supportedEntryTypes.includes('largest-contentful-paint')) {
            const lcpEntries = performance.getEntriesByType('largest-contentful-paint');
            if (lcpEntries.length > 0) {
                vitals.lcp = lcpEntries[lcpEntries.length - 1].renderTime || lcpEntries[lcpEntries.length - 1].loadTime;
            }
        }

        // FID (First Input Delay) - requires PerformanceObserver
        if (window.PerformanceObserver && PerformanceObserver.supportedEntryTypes.includes('first-input')) {
            const fidEntries = performance.getEntriesByType('first-input');
            if (fidEntries.length > 0) {
                vitals.fid = fidEntries[0].processingStart - fidEntries[0].startTime;
            }
        }

        // CLS (Cumulative Layout Shift)
        if (window.PerformanceObserver && PerformanceObserver.supportedEntryTypes.includes('layout-shift')) {
            let clsScore = 0;
            const layoutShiftEntries = performance.getEntriesByType('layout-shift');
            layoutShiftEntries.forEach(entry => {
                if (!entry.hadRecentInput) {
                    clsScore += entry.value;
                }
            });
            vitals.cls = clsScore;
        }

        // FCP (First Contentful Paint)
        const fcpEntries = performance.getEntriesByName('first-contentful-paint');
        if (fcpEntries.length > 0) {
            vitals.fcp = fcpEntries[0].startTime;
        }

        // TTFB (Time to First Byte)
        if (performance.timing) {
            const timing = performance.timing;
            vitals.ttfb = timing.responseStart - timing.requestStart;
        }

        return vitals;
    }

    // Collect Navigation Timing
    function collectNavigationTiming() {
        if (!performance.timing) {
            return null;
        }

        const timing = performance.timing;
        const navigation = performance.navigation || {};

        return {
            navigation_start: timing.navigationStart,
            unload_event_start: timing.unloadEventStart || null,
            unload_event_end: timing.unloadEventEnd || null,
            redirect_start: timing.redirectStart || null,
            redirect_end: timing.redirectEnd || null,
            redirect_count: navigation.redirectCount || 0,
            fetch_start: timing.fetchStart,
            domain_lookup_start: timing.domainLookupStart,
            domain_lookup_end: timing.domainLookupEnd,
            connect_start: timing.connectStart,
            connect_end: timing.connectEnd,
            secure_connection_start: timing.secureConnectionStart || null,
            request_start: timing.requestStart,
            response_start: timing.responseStart,
            response_end: timing.responseEnd,
            dom_interactive: timing.domInteractive,
            dom_content_loaded_event_start: timing.domContentLoadedEventStart,
            dom_content_loaded_event_end: timing.domContentLoadedEventEnd,
            dom_complete: timing.domComplete,
            load_event_start: timing.loadEventStart,
            load_event_end: timing.loadEventEnd
        };
    }

    // Collect Resource Timing
    function collectResourceTimings() {
        if (!performance.getEntriesByType) {
            return [];
        }

        const resources = performance.getEntriesByType('resource');
        return resources.map(resource => ({
            name: resource.name,
            initiator_type: resource.initiatorType,
            start_time: resource.startTime,
            duration: resource.duration,
            fetch_start: resource.fetchStart,
            domain_lookup_start: resource.domainLookupStart,
            domain_lookup_end: resource.domainLookupEnd,
            connect_start: resource.connectStart,
            connect_end: resource.connectEnd,
            secure_connection_start: resource.secureConnectionStart || null,
            request_start: resource.requestStart,
            response_start: resource.responseStart,
            response_end: resource.responseEnd,
            transfer_size: resource.transferSize || 0,
            encoded_body_size: resource.encodedBodySize || 0,
            decoded_body_size: resource.decodedBodySize || 0,
            from_cache: resource.transferSize === 0 && resource.decodedBodySize > 0
        }));
    }

    // Collect Memory Metrics
    function collectMemoryMetrics() {
        if (!performance.memory) {
            return {
                js_heap_size_limit: 0,
                total_js_heap_size: 0,
                used_js_heap_size: 0,
                dom_node_count: document.getElementsByTagName('*').length,
                event_listener_count: 0,
                detached_node_count: 0,
                timestamp: Date.now()
            };
        }

        const memory = performance.memory;
        return {
            js_heap_size_limit: memory.jsHeapSizeLimit,
            total_js_heap_size: memory.totalJSHeapSize,
            used_js_heap_size: memory.usedJSHeapSize,
            dom_node_count: document.getElementsByTagName('*').length,
            event_listener_count: getEventListenerCount(),
            detached_node_count: 0, // Requires additional API
            timestamp: Date.now()
        };
    }

    // Estimate event listener count
    function getEventListenerCount() {
        // This is an approximation - exact count requires Chrome DevTools Protocol
        let count = 0;
        try {
            const allElements = document.getElementsByTagName('*');
            // Sample a subset to avoid performance issues
            const sampleSize = Math.min(100, allElements.length);
            for (let i = 0; i < sampleSize; i++) {
                const elem = allElements[i];
                if (elem._getEventListeners) {
                    const listeners = elem._getEventListeners();
                    for (let type in listeners) {
                        count += listeners[type].length;
                    }
                }
            }
            // Extrapolate to full document
            return Math.round(count * (allElements.length / sampleSize));
        } catch (e) {
            return 0;
        }
    }

    // Long Task Timing (requires PerformanceObserver)
    function collectLongTasks() {
        if (!window.PerformanceObserver || !PerformanceObserver.supportedEntryTypes.includes('longtask')) {
            return [];
        }

        const longTasks = performance.getEntriesByType('longtask');
        return longTasks.map(task => ({
            name: task.name,
            entry_type: task.entryType,
            start_time: task.startTime,
            duration: task.duration,
            attribution: task.attribution ? task.attribution.map(attr => ({
                name: attr.name,
                entry_type: attr.entryType,
                start_time: attr.startTime,
                duration: attr.duration,
                container_type: attr.containerType,
                container_src: attr.containerSrc,
                container_id: attr.containerId,
                container_name: attr.containerName
            })) : []
        }));
    }

    // Paint Timing
    function collectPaintTimings() {
        if (!performance.getEntriesByType) {
            return {};
        }

        const paintEntries = performance.getEntriesByType('paint');
        const timings = {};

        paintEntries.forEach(entry => {
            timings[entry.name.replace(/-/g, '_')] = entry.startTime;
        });

        return timings;
    }

    // Collect all performance data
    perfData.coreWebVitals = collectCoreWebVitals();
    perfData.navigationTiming = collectNavigationTiming();
    perfData.resourceTimings = collectResourceTimings();
    perfData.memory = collectMemoryMetrics();
    perfData.longTasks = collectLongTasks();
    perfData.paintTimings = collectPaintTimings();

    // Return the collected data as JSON
    return JSON.stringify(perfData);
})();
