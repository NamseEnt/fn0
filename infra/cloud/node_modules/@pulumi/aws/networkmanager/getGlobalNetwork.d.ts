import * as pulumi from "@pulumi/pulumi";
/**
 * Provides details about an existing Network Manager global network.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.networkmanager.getGlobalNetwork({
 *     globalNetworkId: globalNetworkId,
 * });
 * ```
 */
export declare function getGlobalNetwork(args: GetGlobalNetworkArgs, opts?: pulumi.InvokeOptions): Promise<GetGlobalNetworkResult>;
/**
 * A collection of arguments for invoking getGlobalNetwork.
 */
export interface GetGlobalNetworkArgs {
    /**
     * ID of the specific global network to retrieve.
     */
    globalNetworkId: string;
    /**
     * Map of resource tags.
     */
    tags?: {
        [key: string]: string;
    };
}
/**
 * A collection of values returned by getGlobalNetwork.
 */
export interface GetGlobalNetworkResult {
    /**
     * ARN of the global network.
     */
    readonly arn: string;
    /**
     * Description of the global network.
     */
    readonly description: string;
    readonly globalNetworkId: string;
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    /**
     * Map of resource tags.
     */
    readonly tags: {
        [key: string]: string;
    };
}
/**
 * Provides details about an existing Network Manager global network.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.networkmanager.getGlobalNetwork({
 *     globalNetworkId: globalNetworkId,
 * });
 * ```
 */
export declare function getGlobalNetworkOutput(args: GetGlobalNetworkOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetGlobalNetworkResult>;
/**
 * A collection of arguments for invoking getGlobalNetwork.
 */
export interface GetGlobalNetworkOutputArgs {
    /**
     * ID of the specific global network to retrieve.
     */
    globalNetworkId: pulumi.Input<string>;
    /**
     * Map of resource tags.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
