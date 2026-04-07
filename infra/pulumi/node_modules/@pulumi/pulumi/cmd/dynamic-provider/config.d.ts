import * as dynamic from "../../dynamic";
export declare class Config implements dynamic.Config {
    private rawConfig;
    private projectName;
    constructor(rawConfig: Record<string, any>, projectName: string);
    get(key: string): string | undefined;
    require(key: string): string;
}
