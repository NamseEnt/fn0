import * as pulumi from "@pulumi/pulumi";
/**
 * Provides details about existing Network Manager connections.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.networkmanager.getConnections({
 *     globalNetworkId: globalNetworkId,
 *     tags: {
 *         Env: "test",
 *     },
 * });
 * ```
 */
export declare function getConnections(args: GetConnectionsArgs, opts?: pulumi.InvokeOptions): Promise<GetConnectionsResult>;
/**
 * A collection of arguments for invoking getConnections.
 */
export interface GetConnectionsArgs {
    /**
     * ID of the device of the connections to retrieve.
     */
    deviceId?: string;
    /**
     * ID of the Global Network of the connections to retrieve.
     */
    globalNetworkId: string;
    /**
     * Restricts the list to the connections with these tags.
     */
    tags?: {
        [key: string]: string;
    };
}
/**
 * A collection of values returned by getConnections.
 */
export interface GetConnectionsResult {
    readonly deviceId?: string;
    readonly globalNetworkId: string;
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    /**
     * IDs of the connections.
     */
    readonly ids: string[];
    readonly tags?: {
        [key: string]: string;
    };
}
/**
 * Provides details about existing Network Manager connections.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.networkmanager.getConnections({
 *     globalNetworkId: globalNetworkId,
 *     tags: {
 *         Env: "test",
 *     },
 * });
 * ```
 */
export declare function getConnectionsOutput(args: GetConnectionsOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetConnectionsResult>;
/**
 * A collection of arguments for invoking getConnections.
 */
export interface GetConnectionsOutputArgs {
    /**
     * ID of the device of the connections to retrieve.
     */
    deviceId?: pulumi.Input<string>;
    /**
     * ID of the Global Network of the connections to retrieve.
     */
    globalNetworkId: pulumi.Input<string>;
    /**
     * Restricts the list to the connections with these tags.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
