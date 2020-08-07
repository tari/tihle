// tihle ServiceWorker
//
// It doesn't do much; just provides offline access to assets
// and lets Chrome decide (by its mere presence) it should be
// installable.
const cacheName = 'tihle-0.3.0a';
const cachedFiles = [
    'index.html',
    'tihle.js',
    'tihle.wasm',
    'tihle.data',
    'tihle.webmanifest',
    'maskable_icon.png',
];

/**
 * Block install on precaching files.
 */
self.addEventListener('install', e => {
    console.log('Handling app install');
    e.waitUntil(
        caches.open(cacheName).then(cache => {
            console.log('Caching app files');
            return cache.addAll(cachedFiles);
        })
    );
});

/**
 * Serve fetches from the cache if present, otherwise fetch and cache the
 * response.
 */
self.addEventListener('fetch', e => {
    e.respondWith(
        caches.match(e.request).then(r => {
            console.log('Fetching resource: '+e.request.url);
            return r || fetch(e.request).then(response => {
                return caches.open(cacheName).then(cache => {
                    console.log('Caching new resource: '+e.request.url);
                    cache.put(e.request, response.clone());
                    return response;
                });
            });
        })
    );
});

/**
 * On update, remove old data from the cache.
 */
self.addEventListener('activate', e => {
    e.waitUntil(
        caches.keys().then(keyList => {
            return Promise.all(keyList.map(key => {
                if(key !== cacheName) {
                    return caches.delete(key);
                }
            }));
        })
    );
});
