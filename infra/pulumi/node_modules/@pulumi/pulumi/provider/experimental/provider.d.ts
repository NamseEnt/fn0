import { ComponentResource, ComponentResourceOptions } from "../../resource";
import { ConstructResult, Provider } from "../provider";
import { Inputs } from "../../output";
export declare type ComponentResourceConstructor = {
    new (name: string, args: any, opts?: ComponentResourceOptions): ComponentResource;
};
/**
 * Get all Pulumi Component constructors from a module's exports.
 * @param moduleExports The exports object of the module to check.
 * @returns Array of Pulumi Component constructors found in the exports.
 */
export declare function getPulumiComponents(moduleExports: any): ComponentResourceConstructor[];
export interface ComponentProviderOptions {
    components: ComponentResourceConstructor[];
    dirname?: string;
    name: string;
    namespace?: string;
}
export declare class ComponentProvider implements Provider {
    readonly options: ComponentProviderOptions & {
        version?: string;
    };
    private packageJSON;
    private path;
    private componentConstructors;
    private name;
    private namespace?;
    private componentDefinitions?;
    private cachedSchema?;
    version: string;
    static validateResourceType(packageName: string, resourceType: string): void;
    constructor(options: ComponentProviderOptions & {
        version?: string;
    });
    getSchema(): Promise<string>;
    construct(name: string, type: string, inputs: Inputs, options: ComponentResourceOptions): Promise<ConstructResult>;
}
export declare function componentProviderHost(options: ComponentProviderOptions): Promise<void>;
