import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a SageMaker AI Device resource.
 *
 * ## Example Usage
 *
 * ### Basic usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.sagemaker.Device("example", {
 *     deviceFleetName: exampleAwsSagemakerDeviceFleet.deviceFleetName,
 *     device: {
 *         deviceName: "example",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import SageMaker AI Devices using the `device-fleet-name/device-name`. For example:
 *
 * ```sh
 * $ pulumi import aws:sagemaker/device:Device example my-fleet/my-device
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
    readonly agentVersion: pulumi.Output<string>;
    /**
     * The Amazon Resource Name (ARN) assigned by AWS to this Device.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The device to register with SageMaker AI Edge Manager. See Device details below.
     */
    readonly device: pulumi.Output<outputs.sagemaker.DeviceDevice>;
    /**
     * The name of the Device Fleet.
     */
    readonly deviceFleetName: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
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
    agentVersion?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) assigned by AWS to this Device.
     */
    arn?: pulumi.Input<string>;
    /**
     * The device to register with SageMaker AI Edge Manager. See Device details below.
     */
    device?: pulumi.Input<inputs.sagemaker.DeviceDevice>;
    /**
     * The name of the Device Fleet.
     */
    deviceFleetName?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a Device resource.
 */
export interface DeviceArgs {
    /**
     * The device to register with SageMaker AI Edge Manager. See Device details below.
     */
    device: pulumi.Input<inputs.sagemaker.DeviceDevice>;
    /**
     * The name of the Device Fleet.
     */
    deviceFleetName: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
