import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a CodeBuild Fleet Resource.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = new aws.codebuild.Fleet("test", {
 *     baseCapacity: 2,
 *     computeType: "BUILD_GENERAL1_SMALL",
 *     environmentType: "LINUX_CONTAINER",
 *     name: "full-example-codebuild-fleet",
 *     overflowBehavior: "QUEUE",
 *     scalingConfiguration: {
 *         maxCapacity: 5,
 *         scalingType: "TARGET_TRACKING_SCALING",
 *         targetTrackingScalingConfigs: [{
 *             metricType: "FLEET_UTILIZATION_RATE",
 *             targetValue: 97.5,
 *         }],
 *     },
 * });
 * ```
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.codebuild.Fleet("example", {name: "example-codebuild-fleet"});
 * ```
 *
 * ## Import
 *
 * ### Identity Schema
 *
 * #### Required
 *
 * - `arn` (String) Amazon Resource Name (ARN) of the CodeBuild fleet.
 *
 * Using `pulumi import`, import CodeBuild Fleet using the `name`. For example:
 *
 * console
 *
 * % pulumi import aws_codebuild_fleet.name fleet-name
 */
export declare class Fleet extends pulumi.CustomResource {
    /**
     * Get an existing Fleet resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: FleetState, opts?: pulumi.CustomResourceOptions): Fleet;
    /**
     * Returns true if the given object is an instance of Fleet.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Fleet;
    /**
     * ARN of the Fleet.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Number of machines allocated to the ﬂeet.
     */
    readonly baseCapacity: pulumi.Output<number>;
    /**
     * The compute configuration of the compute fleet. This is only required if `computeType` is set to `ATTRIBUTE_BASED_COMPUTE` or `CUSTOM_INSTANCE_TYPE`. See `computeConfiguration` below.
     */
    readonly computeConfiguration: pulumi.Output<outputs.codebuild.FleetComputeConfiguration | undefined>;
    /**
     * Compute resources the compute fleet uses. See [compute types](https://docs.aws.amazon.com/codebuild/latest/userguide/build-env-ref-compute-types.html#environment.types) for more information and valid values.
     */
    readonly computeType: pulumi.Output<string>;
    /**
     * Creation time of the fleet.
     */
    readonly created: pulumi.Output<string>;
    /**
     * Environment type of the compute fleet. See [environment types](https://docs.aws.amazon.com/codebuild/latest/userguide/build-env-ref-compute-types.html#environment.types) for more information and valid values.
     *
     * The following arguments are optional:
     */
    readonly environmentType: pulumi.Output<string>;
    /**
     * The service role associated with the compute fleet.
     */
    readonly fleetServiceRole: pulumi.Output<string | undefined>;
    /**
     * The Amazon Machine Image (AMI) of the compute fleet.
     */
    readonly imageId: pulumi.Output<string | undefined>;
    /**
     * Last modification time of the fleet.
     */
    readonly lastModified: pulumi.Output<string>;
    /**
     * Fleet name.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Overflow behavior for compute fleet. Valid values: `ON_DEMAND`, `QUEUE`.
     */
    readonly overflowBehavior: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Configuration block. This option is only valid when your overflow behavior is `QUEUE`. See `scalingConfiguration` below.
     */
    readonly scalingConfiguration: pulumi.Output<outputs.codebuild.FleetScalingConfiguration | undefined>;
    /**
     * Nested attribute containing information about the current status of the fleet.
     */
    readonly statuses: pulumi.Output<outputs.codebuild.FleetStatus[]>;
    /**
     * Map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * Configuration block. See `vpcConfig` below.
     */
    readonly vpcConfigs: pulumi.Output<outputs.codebuild.FleetVpcConfig[] | undefined>;
    /**
     * Create a Fleet resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: FleetArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Fleet resources.
 */
export interface FleetState {
    /**
     * ARN of the Fleet.
     */
    arn?: pulumi.Input<string>;
    /**
     * Number of machines allocated to the ﬂeet.
     */
    baseCapacity?: pulumi.Input<number>;
    /**
     * The compute configuration of the compute fleet. This is only required if `computeType` is set to `ATTRIBUTE_BASED_COMPUTE` or `CUSTOM_INSTANCE_TYPE`. See `computeConfiguration` below.
     */
    computeConfiguration?: pulumi.Input<inputs.codebuild.FleetComputeConfiguration>;
    /**
     * Compute resources the compute fleet uses. See [compute types](https://docs.aws.amazon.com/codebuild/latest/userguide/build-env-ref-compute-types.html#environment.types) for more information and valid values.
     */
    computeType?: pulumi.Input<string>;
    /**
     * Creation time of the fleet.
     */
    created?: pulumi.Input<string>;
    /**
     * Environment type of the compute fleet. See [environment types](https://docs.aws.amazon.com/codebuild/latest/userguide/build-env-ref-compute-types.html#environment.types) for more information and valid values.
     *
     * The following arguments are optional:
     */
    environmentType?: pulumi.Input<string>;
    /**
     * The service role associated with the compute fleet.
     */
    fleetServiceRole?: pulumi.Input<string>;
    /**
     * The Amazon Machine Image (AMI) of the compute fleet.
     */
    imageId?: pulumi.Input<string>;
    /**
     * Last modification time of the fleet.
     */
    lastModified?: pulumi.Input<string>;
    /**
     * Fleet name.
     */
    name?: pulumi.Input<string>;
    /**
     * Overflow behavior for compute fleet. Valid values: `ON_DEMAND`, `QUEUE`.
     */
    overflowBehavior?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Configuration block. This option is only valid when your overflow behavior is `QUEUE`. See `scalingConfiguration` below.
     */
    scalingConfiguration?: pulumi.Input<inputs.codebuild.FleetScalingConfiguration>;
    /**
     * Nested attribute containing information about the current status of the fleet.
     */
    statuses?: pulumi.Input<pulumi.Input<inputs.codebuild.FleetStatus>[]>;
    /**
     * Map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Configuration block. See `vpcConfig` below.
     */
    vpcConfigs?: pulumi.Input<pulumi.Input<inputs.codebuild.FleetVpcConfig>[]>;
}
/**
 * The set of arguments for constructing a Fleet resource.
 */
export interface FleetArgs {
    /**
     * Number of machines allocated to the ﬂeet.
     */
    baseCapacity: pulumi.Input<number>;
    /**
     * The compute configuration of the compute fleet. This is only required if `computeType` is set to `ATTRIBUTE_BASED_COMPUTE` or `CUSTOM_INSTANCE_TYPE`. See `computeConfiguration` below.
     */
    computeConfiguration?: pulumi.Input<inputs.codebuild.FleetComputeConfiguration>;
    /**
     * Compute resources the compute fleet uses. See [compute types](https://docs.aws.amazon.com/codebuild/latest/userguide/build-env-ref-compute-types.html#environment.types) for more information and valid values.
     */
    computeType: pulumi.Input<string>;
    /**
     * Environment type of the compute fleet. See [environment types](https://docs.aws.amazon.com/codebuild/latest/userguide/build-env-ref-compute-types.html#environment.types) for more information and valid values.
     *
     * The following arguments are optional:
     */
    environmentType: pulumi.Input<string>;
    /**
     * The service role associated with the compute fleet.
     */
    fleetServiceRole?: pulumi.Input<string>;
    /**
     * The Amazon Machine Image (AMI) of the compute fleet.
     */
    imageId?: pulumi.Input<string>;
    /**
     * Fleet name.
     */
    name?: pulumi.Input<string>;
    /**
     * Overflow behavior for compute fleet. Valid values: `ON_DEMAND`, `QUEUE`.
     */
    overflowBehavior?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Configuration block. This option is only valid when your overflow behavior is `QUEUE`. See `scalingConfiguration` below.
     */
    scalingConfiguration?: pulumi.Input<inputs.codebuild.FleetScalingConfiguration>;
    /**
     * Map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Configuration block. See `vpcConfig` below.
     */
    vpcConfigs?: pulumi.Input<pulumi.Input<inputs.codebuild.FleetVpcConfig>[]>;
}
