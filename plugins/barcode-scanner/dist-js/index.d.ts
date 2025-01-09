import { PermissionState, PluginListener } from '@tauri-apps/api/core';
export declare enum Format {
    QRCode = "QR_CODE",
    UPC_A = "UPC_A",
    UPC_E = "UPC_E",
    EAN8 = "EAN_8",
    EAN13 = "EAN_13",
    Code39 = "CODE_39",
    Code93 = "CODE_93",
    Code128 = "CODE_128",
    Codabar = "CODABAR",
    ITF = "ITF",
    Aztec = "AZTEC",
    DataMatrix = "DATA_MATRIX",
    PDF417 = "PDF_417"
}
export interface ScanOptions {
    cameraDirection?: 'back' | 'front';
    formats?: Format[];
    windowed?: boolean;
}
export interface Scanned {
    content: string;
    format: Format;
    bounds: unknown;
}
/**
 * Start scanning.
 * @param options
 */
export declare function scan(options?: ScanOptions): Promise<Scanned>;
/**
 * Cancel the current scan process.
 */
export declare function cancel(): Promise<void>;
/**
 * Get permission state.
 */
export declare function checkPermissions(): Promise<PermissionState>;
/**
 * Request permissions to use the camera.
 */
export declare function requestPermissions(): Promise<PermissionState>;
/**
 * Open application settings. Useful if permission was denied and the user must manually enable it.
 */
export declare function openAppSettings(): Promise<void>;
/**
 * Start scanning and listen for barcode events.
 * @param options Scan options
 * @param onDetect Callback for when a barcode is detected
 * @param onError Optional callback for errors
 */
export declare function startScan(options: ScanOptions, onDetect: (scanned: Scanned) => void): Promise<PluginListener>;
/**
 * Stop the current scanning process.
 */
export declare function stopScan(): Promise<void>;
