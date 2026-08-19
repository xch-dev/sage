export declare function isNdefAvailable(): Promise<boolean>;
export declare function getNdefPayloads(): Promise<number[][]>;
export interface WebviewBounds {
    label: string;
    x: number;
    y: number;
    width: number;
    height: number;
}
export declare function setWebviewBounds(bounds: WebviewBounds): Promise<void>;
export declare function snapshotWebview(label: string, width?: number): Promise<string>;
