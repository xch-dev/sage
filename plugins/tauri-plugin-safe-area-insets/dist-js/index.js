import { invoke } from '@tauri-apps/api/core';

async function getInsets() {
    return await invoke("plugin:window-insets|get_insets");
}

export { getInsets };
