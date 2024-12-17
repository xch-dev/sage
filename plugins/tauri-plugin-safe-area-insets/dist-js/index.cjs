'use strict';

var core = require('@tauri-apps/api/core');

async function getInsets() {
    return await core.invoke("plugin:window-insets|get_insets");
}

exports.getInsets = getInsets;
