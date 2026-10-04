(function() {
    var scripts = [
        '/simulator/src/00_globals.js',
        '/simulator/src/01_audio.js',
        '/simulator/src/02_core.js',
        '/simulator/src/03_world.js',
        '/simulator/src/04_buildings.js',
        '/simulator/src/05_agents.js',
        '/simulator/src/06_controls.js',
        '/simulator/src/07_ui.js',
        '/simulator/src/08_main.js',
        '/simulator/src/09_network.js'
    ];

    // Buffer the initial container in case view.rs calls initSandbox3D before scripts load.
    var pendingContainer = null;
    window.initSandbox3D = function(container) {
        pendingContainer = container;
    };

    function loadScript(src, cb) {
        var existing = document.querySelector('script[src="' + src + '"]');
        if (existing) { cb(); return; }
        var s = document.createElement('script');
        s.src = src;
        s.onload = cb;
        document.head.appendChild(s);
    }

    function loadScripts(list, index, cb) {
        if (index >= list.length) { cb(); return; }
        loadScript(list[index], function() {
            loadScripts(list, index + 1, cb);
        });
    }

    loadScripts(scripts, 0, function() {
        console.log("Modular Sandbox successfully loaded!");
        // Now 02_core.js has overwritten window.initSandbox3D with the real one.
        if (pendingContainer && typeof window.initSandbox3D === 'function') {
            window.initSandbox3D(pendingContainer);
        }
    });
})();
