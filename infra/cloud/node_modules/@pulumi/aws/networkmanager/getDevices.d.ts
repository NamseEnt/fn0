import * as pulumi from "@pulumi/pulumi";
/**
 * Provides details about existing Network Manager devices.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.networkmanager.getDevices({
 *     globalNetworkId: globalNetworkId,
 *     tags: {
 *         Env: "test",
 *     },
 * });
 * ```
 */
export declare function getDevices(args: GetDevicesArgs, opts?: pulumi.InvokeOptions): Promise<GetDevicesResult>;
/**
 * A collection of arguments for invoking getDevices.
 */
export interface GetDevicesArgs {
    /**
     * ID of the Global Network of the devices to retrieve.
     */
    globalNetworkId: string;
    /**
     * ID of the site of the devices to retrieve.
     */
    siteId?: string;
    /**
     * Restricts the list to the devices with these tags.
     */
    tags?: {
        [key: string]: string;
    };
}
/**
 * A collection of values returned by getDevices.
 */
export interface GetDevicesResult {
    readonly globalNetworkId: string;
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    /**
     * IDs of the devices.
     */
    readonly ids: string[];
    readonly siteId?: string;
    readonly tags?: {
        [key: string]: string;
    };
}
/**
 * Provides details about existing Network Manager devices.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.networkmanager.getDevices({
 *     globalNetworkId: globalNetworkId,
 *     tags: {
 *         Env: "test",
 *     },
 * });
 * ```
 */
export declare function getDevicesOutput(args: GetDevicesOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetDevicesResult>;
/**
 * A collection of arguments for invoking getDevices.
 */
export interface GetDevicesOutputArgs {
    /**
     * ID of the Global Network of the devices to retrieve.
     */
    globalNetworkId: pulumi.Input<string>;
    /**
     * ID of the site of the devices to retrieve.
     */
    siteId?: pulumi.Input<string>;
    /**
     * Restricts the list to the devices with these tags.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
