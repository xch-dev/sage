import { invoke } from '@tauri-apps/api/core';

async function isNdefAvailable() {
    return await invoke('plugin:sage|is_ndef_available').then((r) => r.available);
}
async function getNdefPayloads() {
    return await invoke('plugin:sage|get_ndef_payloads').then((r) => r.payloads);
}
async function scanTangemCard() {
    return await invoke('plugin:sage|scan_tangem_card');
}

export { getNdefPayloads, isNdefAvailable, scanTangemCard };
