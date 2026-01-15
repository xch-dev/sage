import { invoke } from '@tauri-apps/api/core';

async function isNdefAvailable() {
    return await invoke('plugin:sage|is_ndef_available').then((r) => r.available);
}
async function getNdefPayloads() {
    return await invoke('plugin:sage|get_ndef_payloads').then((r) => r.payloads);
}
async function testTangem() {
    return await invoke('plugin:sage|test_tangem').then((r) => r.output);
}

export { getNdefPayloads, isNdefAvailable, testTangem };
