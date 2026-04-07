/**
 * Returns a copy of the full configuration map.
 */
export declare function allConfig(): {
    [key: string]: string;
};
/**
 * Overwrites the configuration map.
 */
export declare function setAllConfig(c: {
    [key: string]: string;
}, secretKeys?: string[]): void;
/**
 * Sets a configuration variable.
 */
export declare function setConfig(k: string, v: string): void;
/**
 * Returns a configuration variable's value, or `undefined` if it is unset.
 */
export declare function getConfig(k: string): string | undefined;
