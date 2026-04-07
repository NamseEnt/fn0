declare type Exports = string | {
    [key: string]: SubExports;
};
declare type SubExports = string | {
    [key: string]: SubExports;
} | null;
declare type PackageDefinition = {
    name: string;
    exports?: Exports;
};
declare type RequireOpts = {
    isRequire?: boolean;
    isImport?: boolean;
};
export declare function getModuleFromPath(path: string, packageDefinition?: PackageDefinition, opts?: RequireOpts): string;
export {};
