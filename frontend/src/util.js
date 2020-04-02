import { get } from 'svelte/store';

import { signedIn, userId, evtSource } from './stores.js';

async function getSseToken() {
    const response = await fetch(
        "/api/sse-auth-token",
        {
            method: 'GET',
            credentials: 'same-origin'
        }
    );

    return response.text();
}

async function setupSse(userId, token) {
    evtSource.set(new EventSource(
        'http://localhost:3000/push/' + userId + '?' + token
    ));
}

async function updateStoresAfterLogin(userInfo, sseToken) {
    signedIn.set(true);
    userId.set(userInfo.user_id);
    setupSse(userInfo.user_id, sseToken);
}

async function updateStoresAfterLogout() {
    signedIn.set(false);
    userId.set(-1);
    var e = get(evtSource);
    e.close();
    evtSource.set(null);
}

export { getSseToken, setupSse, updateStoresAfterLogin, updateStoresAfterLogout }
