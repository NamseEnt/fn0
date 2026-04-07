import * as pulumi from "@pulumi/pulumi";
import * as outputs from "../types/output";
/**
 * Provides details about an existing Network Manager device.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.networkmanager.getDevice({
 *     globalNetworkId: globalNetworkId,
 *     deviceId: deviceId,
 * });
 * ```
 */
export declare function getDevice(args: GetDeviceArgs, opts?: pulumi.InvokeOptions): Promise<GetDeviceResult>;
/**
 * A collection of arguments for invoking getDevice.
 */
export interface GetDeviceArgs {
    /**
     * ID of the device.
     */
    deviceId: string;
    /**
     * ID of the global network.
     */
    globalNetworkId: string;
    /**
     * Key-value tags for the device.
     */
    tags?: {
        [key: string]: string;
    };
}
/**
 * A collection of values returned by getDevice.
 */
export interface GetDeviceResult {
    /**
     * ARN of the device.
     */
    readonly arn: string;
    /**
     * AWS location of the device. Documented below.
     */
    readonly awsLocations: outputs.networkmanager.GetDeviceAwsLocation[];
    /**
     * Description of the device.
     */
    readonly description: string;
    readonly deviceId: string;
    readonly globalNetworkId: string;
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    /**
     * Location of the device. Documented below.
     */
    readonly locations: outputs.networkmanager.GetDeviceLocation[];
    /**
     * Model of device.
     */
    readonly model: string;
    /**
     * Serial number of the device.
     */
    readonly serialNumber: string;
    /**
     * ID of the site.
     */
    readonly siteId: string;
    /**
     * Key-value tags for the device.
     */
    readonly tags: {
        [key: string]: string;
    };
    /**
     * Type of device.
     */
    readonly type: string;
    /**
     * Vendor of the device.
     */
    readonly vendor: string;
}
/**
 * Provides details about an existing Network Manager device.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.networkmanager.getDevice({
 *     globalNetworkId: globalNetworkId,
 *     deviceId: deviceId,
 * });
 * ```
 */
export declare function getDeviceOutput(args: GetDeviceOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetDeviceResult>;
/**
 * A collection of arguments for invoking getDevice.
 */
export interface GetDeviceOutputArgs {
    /**
     * ID of the device.
     */
    deviceId: pulumi.Input<string>;
    /**
     * ID of the global network.
     */
    globalNetworkId: pulumi.Input<string>;
    /**
     * Key-value tags for the device.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
