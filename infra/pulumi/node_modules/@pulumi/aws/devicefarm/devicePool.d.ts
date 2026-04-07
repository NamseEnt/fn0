import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a resource to manage AWS Device Farm Device Pools.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.devicefarm.DevicePool("example", {
 *     name: "example",
 *     projectArn: exampleAwsDevicefarmProject.arn,
 *     rules: [{
 *         attribute: "OS_VERSION",
 *         operator: "EQUALS",
 *         value: "\"AVAILABLE\"",
 *     }],
 * });
 * ```
 *
 * ## Import
 *
 * ### Identity Schema
 *
 * #### Required
 *
 * - `arn` (String) Amazon Resource Name (ARN) of the Device Farm device pool.
 *
 * Using `pulumi import`, import DeviceFarm Device Pools using their ARN. For example:
 *
 * console
 *
 * % pulumi import aws_devicefarm_device_pool.example arn:aws:devicefarm:us-west-2:123456789012:devicepool:4fa784c7-ccb4-4dbf-ba4f-02198320daa1/4fa784c7-ccb4-4dbf-ba4f-02198320daa1
 */
export declare class DevicePool extends pulumi.CustomResource {
    /**
     * Get an existing DevicePool resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: DevicePoolState, opts?: pulumi.CustomResourceOptions): DevicePool;
    /**
     * Returns true if the given object is an instance of DevicePool.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is DevicePool;
    /**
     * The Amazon Resource Name of this Device Pool
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The device pool's description.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * The number of devices that Device Farm can add to your device pool.
     */
    readonly maxDevices: pulumi.Output<number | undefined>;
    /**
     * The name of the Device Pool
     */
    readonly name: pulumi.Output<string>;
    /**
     * The ARN of the project for the device pool.
     */
    readonly projectArn: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The device pool's rules. See Rule.
     */
    readonly rules: pulumi.Output<outputs.devicefarm.DevicePoolRule[]>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * A map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    readonly type: pulumi.Output<string>;
    /**
     * Create a DevicePool resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: DevicePoolArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering DevicePool resources.
 */
export interface DevicePoolState {
    /**
     * The Amazon Resource Name of this Device Pool
     */
    arn?: pulumi.Input<string>;
    /**
     * The device pool's description.
     */
    description?: pulumi.Input<string>;
    /**
     * The number of devices that Device Farm can add to your device pool.
     */
    maxDevices?: pulumi.Input<number>;
    /**
     * The name of the Device Pool
     */
    name?: pulumi.Input<string>;
    /**
     * The ARN of the project for the device pool.
     */
    projectArn?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The device pool's rules. See Rule.
     */
    rules?: pulumi.Input<pulumi.Input<inputs.devicefarm.DevicePoolRule>[]>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * A map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    type?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a DevicePool resource.
 */
export interface DevicePoolArgs {
    /**
     * The device pool's description.
     */
    description?: pulumi.Input<string>;
    /**
     * The number of devices that Device Farm can add to your device pool.
     */
    maxDevices?: pulumi.Input<number>;
    /**
     * The name of the Device Pool
     */
    name?: pulumi.Input<string>;
    /**
     * The ARN of the project for the device pool.
     */
    projectArn: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The device pool's rules. See Rule.
     */
    rules: pulumi.Input<pulumi.Input<inputs.devicefarm.DevicePoolRule>[]>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
