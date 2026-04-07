import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a SageMaker AI Device Fleet resource.
 *
 * ## Example Usage
 *
 * ### Basic usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.sagemaker.DeviceFleet("example", {
 *     deviceFleetName: "example",
 *     roleArn: test.arn,
 *     outputConfig: {
 *         s3OutputLocation: `s3://${exampleAwsS3Bucket.bucket}/prefix/`,
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import SageMaker AI Device Fleets using the `name`. For example:
 *
 * ```sh
 * $ pulumi import aws:sagemaker/deviceFleet:DeviceFleet example my-fleet
 * ```
 */
export declare class DeviceFleet extends pulumi.CustomResource {
    /**
     * Get an existing DeviceFleet resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: DeviceFleetState, opts?: pulumi.CustomResourceOptions): DeviceFleet;
    /**
     * Returns true if the given object is an instance of DeviceFleet.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is DeviceFleet;
    /**
     * The Amazon Resource Name (ARN) assigned by AWS to this Device Fleet.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * A description of the fleet.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * The name of the Device Fleet (must be unique).
     */
    readonly deviceFleetName: pulumi.Output<string>;
    /**
     * Whether to create an AWS IoT Role Alias during device fleet creation. The name of the role alias generated will match this pattern: "SageMakerEdge-{DeviceFleetName}".
     */
    readonly enableIotRoleAlias: pulumi.Output<boolean | undefined>;
    readonly iotRoleAlias: pulumi.Output<string>;
    /**
     * Specifies details about the repository. see Output Config details below.
     */
    readonly outputConfig: pulumi.Output<outputs.sagemaker.DeviceFleetOutputConfig>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The Amazon Resource Name (ARN) that has access to AWS Internet of Things (IoT).
     */
    readonly roleArn: pulumi.Output<string>;
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
    /**
     * Create a DeviceFleet resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: DeviceFleetArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering DeviceFleet resources.
 */
export interface DeviceFleetState {
    /**
     * The Amazon Resource Name (ARN) assigned by AWS to this Device Fleet.
     */
    arn?: pulumi.Input<string>;
    /**
     * A description of the fleet.
     */
    description?: pulumi.Input<string>;
    /**
     * The name of the Device Fleet (must be unique).
     */
    deviceFleetName?: pulumi.Input<string>;
    /**
     * Whether to create an AWS IoT Role Alias during device fleet creation. The name of the role alias generated will match this pattern: "SageMakerEdge-{DeviceFleetName}".
     */
    enableIotRoleAlias?: pulumi.Input<boolean>;
    iotRoleAlias?: pulumi.Input<string>;
    /**
     * Specifies details about the repository. see Output Config details below.
     */
    outputConfig?: pulumi.Input<inputs.sagemaker.DeviceFleetOutputConfig>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) that has access to AWS Internet of Things (IoT).
     */
    roleArn?: pulumi.Input<string>;
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
}
/**
 * The set of arguments for constructing a DeviceFleet resource.
 */
export interface DeviceFleetArgs {
    /**
     * A description of the fleet.
     */
    description?: pulumi.Input<string>;
    /**
     * The name of the Device Fleet (must be unique).
     */
    deviceFleetName: pulumi.Input<string>;
    /**
     * Whether to create an AWS IoT Role Alias during device fleet creation. The name of the role alias generated will match this pattern: "SageMakerEdge-{DeviceFleetName}".
     */
    enableIotRoleAlias?: pulumi.Input<boolean>;
    /**
     * Specifies details about the repository. see Output Config details below.
     */
    outputConfig: pulumi.Input<inputs.sagemaker.DeviceFleetOutputConfig>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) that has access to AWS Internet of Things (IoT).
     */
    roleArn: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
