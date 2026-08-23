import { invoke } from '@tauri-apps/api/core';

async function isNdefAvailable() {
    return await invoke('plugin:sage|is_ndef_available').then((r) => r.available);
}
async function getNdefPayloads() {
    return await invoke('plugin:sage|get_ndef_payloads').then((r) => r.payloads);
}
async function setWebviewBounds(bounds) {
    await invoke('plugin:sage|set_webview_bounds', { request: bounds });
}
async function snapshotWebview(label, width = 360) {
    return await invoke('plugin:sage|snapshot_webview', {
        request: { label, width },
    }).then((response) => response.dataUrl);
}

export { getNdefPayloads, isNdefAvailable, setWebviewBounds, snapshotWebview };
