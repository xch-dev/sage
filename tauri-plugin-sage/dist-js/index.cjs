'use strict';

var core = require('@tauri-apps/api/core');

async function isNdefAvailable() {
    return await core.invoke('plugin:sage|is_ndef_available').then((r) => r.available);
}
async function getNdefPayloads() {
    return await core.invoke('plugin:sage|get_ndef_payloads').then((r) => r.payloads);
}
async function setWebviewBounds(bounds) {
    await core.invoke('plugin:sage|set_webview_bounds', { request: bounds });
}
async function snapshotWebview(label, width = 360) {
    return await core.invoke('plugin:sage|snapshot_webview', {
        request: { label, width },
    }).then((response) => response.dataUrl);
}

exports.getNdefPayloads = getNdefPayloads;
exports.isNdefAvailable = isNdefAvailable;
exports.setWebviewBounds = setWebviewBounds;
exports.snapshotWebview = snapshotWebview;
