import * as pulumi from "@pulumi/pulumi";
/**
 * Provides details about an existing Network Manager connection.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.networkmanager.getConnection({
 *     globalNetworkId: globalNetworkId,
 *     connectionId: connectionId,
 * });
 * ```
 */
export declare function getConnection(args: GetConnectionArgs, opts?: pulumi.InvokeOptions): Promise<GetConnectionResult>;
/**
 * A collection of arguments for invoking getConnection.
 */
export interface GetConnectionArgs {
    /**
     * ID of the specific connection to retrieve.
     */
    connectionId: string;
    /**
     * ID of the Global Network of the connection to retrieve.
     */
    globalNetworkId: string;
    /**
     * Key-value tags for the connection.
     */
    tags?: {
        [key: string]: string;
    };
}
/**
 * A collection of values returned by getConnection.
 */
export interface GetConnectionResult {
    /**
     * ARN of the connection.
     */
    readonly arn: string;
    /**
     * ID of the second device in the connection.
     */
    readonly connectedDeviceId: string;
    /**
     * ID of the link for the second device.
     */
    readonly connectedLinkId: string;
    readonly connectionId: string;
    /**
     * Description of the connection.
     */
    readonly description: string;
    /**
     * ID of the first device in the connection.
     */
    readonly deviceId: string;
    readonly globalNetworkId: string;
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    /**
     * ID of the link for the first device.
     */
    readonly linkId: string;
    /**
     * Key-value tags for the connection.
     */
    readonly tags: {
        [key: string]: string;
    };
}
/**
 * Provides details about an existing Network Manager connection.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.networkmanager.getConnection({
 *     globalNetworkId: globalNetworkId,
 *     connectionId: connectionId,
 * });
 * ```
 */
export declare function getConnectionOutput(args: GetConnectionOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetConnectionResult>;
/**
 * A collection of arguments for invoking getConnection.
 */
export interface GetConnectionOutputArgs {
    /**
     * ID of the specific connection to retrieve.
     */
    connectionId: pulumi.Input<string>;
    /**
     * ID of the Global Network of the connection to retrieve.
     */
    globalNetworkId: pulumi.Input<string>;
    /**
     * Key-value tags for the connection.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
