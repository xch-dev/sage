'use strict';

var core = require('@tauri-apps/api/core');

async function isNdefAvailable() {
    return await core.invoke('plugin:sage|is_ndef_available').then((r) => r.available);
}
async function getNdefPayloads() {
    return await core.invoke('plugin:sage|get_ndef_payloads').then((r) => r.payloads);
}

exports.getNdefPayloads = getNdefPayloads;
exports.isNdefAvailable = isNdefAvailable;
