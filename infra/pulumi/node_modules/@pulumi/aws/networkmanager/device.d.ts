import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages a Network Manager Device.
 *
 * Use this resource to create a device in a global network. If you specify both a site ID and a location, the location of the site is used for visualization in the Network Manager console.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.networkmanager.Device("example", {
 *     globalNetworkId: exampleAwsNetworkmanagerGlobalNetwork.id,
 *     siteId: exampleAwsNetworkmanagerSite.id,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_networkmanager_device` using the device ARN. For example:
 *
 * ```sh
 * $ pulumi import aws:networkmanager/device:Device example arn:aws:networkmanager::123456789012:device/global-network-0d47f6t230mz46dy4/device-07f6fd08867abc123
 * ```
 */
export declare class Device extends pulumi.CustomResource {
    /**
     * Get an existing Device resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: DeviceState, opts?: pulumi.CustomResourceOptions): Device;
    /**
     * Returns true if the given object is an instance of Device.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Device;
    /**
     * ARN of the device.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * AWS location of the device. Documented below.
     */
    readonly awsLocation: pulumi.Output<outputs.networkmanager.DeviceAwsLocation | undefined>;
    /**
     * Description of the device.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * ID of the global network.
     *
     * The following arguments are optional:
     */
    readonly globalNetworkId: pulumi.Output<string>;
    /**
     * Location of the device. Documented below.
     */
    readonly location: pulumi.Output<outputs.networkmanager.DeviceLocation | undefined>;
    /**
     * Model of device.
     */
    readonly model: pulumi.Output<string | undefined>;
    /**
     * Serial number of the device.
     */
    readonly serialNumber: pulumi.Output<string | undefined>;
    /**
     * ID of the site.
     */
    readonly siteId: pulumi.Output<string | undefined>;
    /**
     * Key-value tags for the device. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * Map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * Type of device.
     */
    readonly type: pulumi.Output<string | undefined>;
    /**
     * Vendor of the device.
     */
    readonly vendor: pulumi.Output<string | undefined>;
    /**
     * Create a Device resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: DeviceArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Device resources.
 */
export interface DeviceState {
    /**
     * ARN of the device.
     */
    arn?: pulumi.Input<string>;
    /**
     * AWS location of the device. Documented below.
     */
    awsLocation?: pulumi.Input<inputs.networkmanager.DeviceAwsLocation>;
    /**
     * Description of the device.
     */
    description?: pulumi.Input<string>;
    /**
     * ID of the global network.
     *
     * The following arguments are optional:
     */
    globalNetworkId?: pulumi.Input<string>;
    /**
     * Location of the device. Documented below.
     */
    location?: pulumi.Input<inputs.networkmanager.DeviceLocation>;
    /**
     * Model of device.
     */
    model?: pulumi.Input<string>;
    /**
     * Serial number of the device.
     */
    serialNumber?: pulumi.Input<string>;
    /**
     * ID of the site.
     */
    siteId?: pulumi.Input<string>;
    /**
     * Key-value tags for the device. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Type of device.
     */
    type?: pulumi.Input<string>;
    /**
     * Vendor of the device.
     */
    vendor?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a Device resource.
 */
export interface DeviceArgs {
    /**
     * AWS location of the device. Documented below.
     */
    awsLocation?: pulumi.Input<inputs.networkmanager.DeviceAwsLocation>;
    /**
     * Description of the device.
     */
    description?: pulumi.Input<string>;
    /**
     * ID of the global network.
     *
     * The following arguments are optional:
     */
    globalNetworkId: pulumi.Input<string>;
    /**
     * Location of the device. Documented below.
     */
    location?: pulumi.Input<inputs.networkmanager.DeviceLocation>;
    /**
     * Model of device.
     */
    model?: pulumi.Input<string>;
    /**
     * Serial number of the device.
     */
    serialNumber?: pulumi.Input<string>;
    /**
     * ID of the site.
     */
    siteId?: pulumi.Input<string>;
    /**
     * Key-value tags for the device. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Type of device.
     */
    type?: pulumi.Input<string>;
    /**
     * Vendor of the device.
     */
    vendor?: pulumi.Input<string>;
}
