// tihle ServiceWorker
//
// It doesn't do much; just provides offline access to assets
// and lets Chrome decide (by its mere presence) it should be
// installable.
/* eslint-env worker */

// Take over as controller if a client asks us to while waiting
// to become active.
self.addEventListener('message', evt => {
    if (evt.data === 'skipWaiting') {
        self.skipWaiting();
    }
});

// build-web.sh computes a hash over all the app files which identifies
// the PWA version. This ensures any change to a cached file is reflected
// in a change to this file so browsers realize there's an update.
const cacheName = 'tihle-' + packageHash; /* global packageHash */

/**
 * Block install on precaching files.
 */
self.addEventListener('install', e => {
    console.log('Handling app install');
    async function primeCache() {
        let response = fetch('cache.manifest').then(async response => {
            if (!response.ok) {
                throw response;
            }
            return response.text();
        });
        let cache = await caches.open(cacheName);
        let responseText = await response;

        console.groupCollapsed("Resources to cache");
        console.log(responseText);
        console.groupEnd();
        await cache.addAll(responseText.split('\n'));
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
        let cache = caches.open(cacheName);
        let networkResponse = await fetch(e.request);
        (await cache).put(e.request, networkResponse.clone());
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
