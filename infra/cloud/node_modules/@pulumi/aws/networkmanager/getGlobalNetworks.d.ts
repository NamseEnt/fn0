import * as pulumi from "@pulumi/pulumi";
/**
 * Provides details about existing Network Manager global networks.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.networkmanager.getGlobalNetworks({
 *     tags: {
 *         Env: "test",
 *     },
 * });
 * ```
 */
export declare function getGlobalNetworks(args?: GetGlobalNetworksArgs, opts?: pulumi.InvokeOptions): Promise<GetGlobalNetworksResult>;
/**
 * A collection of arguments for invoking getGlobalNetworks.
 */
export interface GetGlobalNetworksArgs {
    /**
     * Restricts the list to the global networks with these tags.
     */
    tags?: {
        [key: string]: string;
    };
}
/**
 * A collection of values returned by getGlobalNetworks.
 */
export interface GetGlobalNetworksResult {
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    /**
     * IDs of the global networks.
     */
    readonly ids: string[];
    readonly tags?: {
        [key: string]: string;
    };
}
/**
 * Provides details about existing Network Manager global networks.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.networkmanager.getGlobalNetworks({
 *     tags: {
 *         Env: "test",
 *     },
 * });
 * ```
 */
export declare function getGlobalNetworksOutput(args?: GetGlobalNetworksOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetGlobalNetworksResult>;
/**
 * A collection of arguments for invoking getGlobalNetworks.
 */
export interface GetGlobalNetworksOutputArgs {
    /**
     * Restricts the list to the global networks with these tags.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
