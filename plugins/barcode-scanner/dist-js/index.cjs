'use strict';

var core = require('@tauri-apps/api/core');

// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT
exports.Format = void 0;
(function (Format) {
    Format["QRCode"] = "QR_CODE";
    Format["UPC_A"] = "UPC_A";
    Format["UPC_E"] = "UPC_E";
    Format["EAN8"] = "EAN_8";
    Format["EAN13"] = "EAN_13";
    Format["Code39"] = "CODE_39";
    Format["Code93"] = "CODE_93";
    Format["Code128"] = "CODE_128";
    Format["Codabar"] = "CODABAR";
    Format["ITF"] = "ITF";
    Format["Aztec"] = "AZTEC";
    Format["DataMatrix"] = "DATA_MATRIX";
    Format["PDF417"] = "PDF_417";
})(exports.Format || (exports.Format = {}));
/**
 * Start scanning.
 * @param options
 */
async function scan(options) {
    return await core.invoke('plugin:barcode-scanner|scan', { ...options });
}
/**
 * Cancel the current scan process.
 */
async function cancel() {
    await core.invoke('plugin:barcode-scanner|cancel');
}
/**
 * Get permission state.
 */
async function checkPermissions() {
    return await core.checkPermissions('barcode-scanner').then((r) => r.camera);
}
/**
 * Request permissions to use the camera.
 */
async function requestPermissions() {
    return await core.requestPermissions('barcode-scanner').then((r) => r.camera);
}
/**
 * Open application settings. Useful if permission was denied and the user must manually enable it.
 */
async function openAppSettings() {
    await core.invoke('plugin:barcode-scanner|open_app_settings');
}
/**
 * Start scanning and listen for barcode events.
 * @param options Scan options
 * @param onDetect Callback for when a barcode is detected
 * @param onError Optional callback for errors
 */
async function startScan(options, onDetect) {
    // First start the scan
    await core.invoke('plugin:barcode-scanner|start_scan', { ...options });
    console.log('Start scanning');
    // Then register the event listener
    return await core.addPluginListener('barcode-scanner', // Need to use the full plugin name with prefix
    'barcode-detected', onDetect);
}
/**
 * Stop the current scanning process.
 */
async function stopScan() {
    await core.invoke('plugin:barcode-scanner|stop_scan');
}

exports.cancel = cancel;
exports.checkPermissions = checkPermissions;
exports.openAppSettings = openAppSettings;
exports.requestPermissions = requestPermissions;
exports.scan = scan;
exports.startScan = startScan;
exports.stopScan = stopScan;
