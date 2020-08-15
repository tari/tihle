// tihle ServiceWorker
//
// It doesn't do much; just provides offline access to assets
// and lets Chrome decide (by its mere presence) it should be
// installable.

// Take over as controller if a client asks us to while waiting
// to become active.
self.addEventListener('message', evt => {
    if (evt.data === 'skipWaiting') {
        self.skipWaiting();
    }
});

cacheManifest = cacheManifest || [];
if (!cacheManifest) {
    console.log("No files given to precache; may not work offline");
}

const cacheName = 'tihle-0.2.0'

/**
 * Block install on precaching files.
 */
self.addEventListener('install', e => {
    console.log('Handling app install');
    async function primeCache() {
        let cache = await caches.open(cacheName);

        let cachePaths = cacheManifest.filter(x => x !== null).map(x => x['path'] + '?sha256=' + x['sha256']);
        console.groupCollapsed("Cached resources");
        console.log(cachePaths);
        console.groupEnd();
        await cache.addAll(cachePaths);
    }

    e.waitUntil(primeCache());
});

/**
 * Serve fetches from the cache if present, otherwise fetch and cache the
 * response.
 */
self.addEventListener('fetch', e => {
    async function handleFetch() {
        let cacheResponse = await caches.match(e.request);
        if (cacheResponse) {
            return cacheResponse;
        }

        console.log('Resource %s not cached; fetching it', e.request.url);
        let networkResponse = await fetch(e.request);
        cache.put(e.request, networkResponse.clone());
        return networkResponse;
    }

    e.respondWith(handleFetch());
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
