import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a GameLift Fleet resource.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.gamelift.Fleet("example", {
 *     buildId: exampleAwsGameliftBuild.id,
 *     ec2InstanceType: "t2.micro",
 *     fleetType: "ON_DEMAND",
 *     name: "example-fleet-name",
 *     runtimeConfiguration: {
 *         serverProcesses: [{
 *             concurrentExecutions: 1,
 *             launchPath: "C:\\game\\GomokuServer.exe",
 *         }],
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import GameLift Fleets using the ID. For example:
 *
 * ```sh
 * $ pulumi import aws:gamelift/fleet:Fleet example <fleet-id>
 * ```
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
     * Fleet ARN.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Build ARN.
     */
    readonly buildArn: pulumi.Output<string>;
    /**
     * ID of the GameLift Build to be deployed on the fleet. Conflicts with `scriptId`.
     */
    readonly buildId: pulumi.Output<string | undefined>;
    /**
     * Prompts GameLift to generate a TLS/SSL certificate for the fleet. See certificate_configuration.
     */
    readonly certificateConfiguration: pulumi.Output<outputs.gamelift.FleetCertificateConfiguration>;
    /**
     * Human-readable description of the fleet.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * Range of IP addresses and port settings that permit inbound traffic to access server processes running on the fleet. See below.
     */
    readonly ec2InboundPermissions: pulumi.Output<outputs.gamelift.FleetEc2InboundPermission[]>;
    /**
     * Name of an EC2 instance typeE.g., `t2.micro`
     */
    readonly ec2InstanceType: pulumi.Output<string>;
    /**
     * Type of fleet. This value must be `ON_DEMAND` or `SPOT`. Defaults to `ON_DEMAND`.
     */
    readonly fleetType: pulumi.Output<string | undefined>;
    /**
     * ARN of an IAM role that instances in the fleet can assume.
     */
    readonly instanceRoleArn: pulumi.Output<string | undefined>;
    readonly logPaths: pulumi.Output<string[]>;
    /**
     * List of names of metric groups to add this fleet to. A metric group tracks metrics across all fleets in the group. Defaults to `default`.
     */
    readonly metricGroups: pulumi.Output<string[]>;
    /**
     * The name of the fleet.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Game session protection policy to apply to all instances in this fleetE.g., `FullProtection`. Defaults to `NoProtection`.
     */
    readonly newGameSessionProtectionPolicy: pulumi.Output<string | undefined>;
    /**
     * Operating system of the fleet's computing resources.
     */
    readonly operatingSystem: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Policy that limits the number of game sessions an individual player can create over a span of time for this fleet. See below.
     */
    readonly resourceCreationLimitPolicy: pulumi.Output<outputs.gamelift.FleetResourceCreationLimitPolicy | undefined>;
    /**
     * Instructions for launching server processes on each instance in the fleet. See below.
     */
    readonly runtimeConfiguration: pulumi.Output<outputs.gamelift.FleetRuntimeConfiguration | undefined>;
    /**
     * Script ARN.
     */
    readonly scriptArn: pulumi.Output<string>;
    /**
     * ID of the GameLift Script to be deployed on the fleet. Conflicts with `buildId`.
     */
    readonly scriptId: pulumi.Output<string | undefined>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Fleet ARN.
     */
    arn?: pulumi.Input<string>;
    /**
     * Build ARN.
     */
    buildArn?: pulumi.Input<string>;
    /**
     * ID of the GameLift Build to be deployed on the fleet. Conflicts with `scriptId`.
     */
    buildId?: pulumi.Input<string>;
    /**
     * Prompts GameLift to generate a TLS/SSL certificate for the fleet. See certificate_configuration.
     */
    certificateConfiguration?: pulumi.Input<inputs.gamelift.FleetCertificateConfiguration>;
    /**
     * Human-readable description of the fleet.
     */
    description?: pulumi.Input<string>;
    /**
     * Range of IP addresses and port settings that permit inbound traffic to access server processes running on the fleet. See below.
     */
    ec2InboundPermissions?: pulumi.Input<pulumi.Input<inputs.gamelift.FleetEc2InboundPermission>[]>;
    /**
     * Name of an EC2 instance typeE.g., `t2.micro`
     */
    ec2InstanceType?: pulumi.Input<string>;
    /**
     * Type of fleet. This value must be `ON_DEMAND` or `SPOT`. Defaults to `ON_DEMAND`.
     */
    fleetType?: pulumi.Input<string>;
    /**
     * ARN of an IAM role that instances in the fleet can assume.
     */
    instanceRoleArn?: pulumi.Input<string>;
    logPaths?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * List of names of metric groups to add this fleet to. A metric group tracks metrics across all fleets in the group. Defaults to `default`.
     */
    metricGroups?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The name of the fleet.
     */
    name?: pulumi.Input<string>;
    /**
     * Game session protection policy to apply to all instances in this fleetE.g., `FullProtection`. Defaults to `NoProtection`.
     */
    newGameSessionProtectionPolicy?: pulumi.Input<string>;
    /**
     * Operating system of the fleet's computing resources.
     */
    operatingSystem?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Policy that limits the number of game sessions an individual player can create over a span of time for this fleet. See below.
     */
    resourceCreationLimitPolicy?: pulumi.Input<inputs.gamelift.FleetResourceCreationLimitPolicy>;
    /**
     * Instructions for launching server processes on each instance in the fleet. See below.
     */
    runtimeConfiguration?: pulumi.Input<inputs.gamelift.FleetRuntimeConfiguration>;
    /**
     * Script ARN.
     */
    scriptArn?: pulumi.Input<string>;
    /**
     * ID of the GameLift Script to be deployed on the fleet. Conflicts with `buildId`.
     */
    scriptId?: pulumi.Input<string>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
 * The set of arguments for constructing a Fleet resource.
 */
export interface FleetArgs {
    /**
     * ID of the GameLift Build to be deployed on the fleet. Conflicts with `scriptId`.
     */
    buildId?: pulumi.Input<string>;
    /**
     * Prompts GameLift to generate a TLS/SSL certificate for the fleet. See certificate_configuration.
     */
    certificateConfiguration?: pulumi.Input<inputs.gamelift.FleetCertificateConfiguration>;
    /**
     * Human-readable description of the fleet.
     */
    description?: pulumi.Input<string>;
    /**
     * Range of IP addresses and port settings that permit inbound traffic to access server processes running on the fleet. See below.
     */
    ec2InboundPermissions?: pulumi.Input<pulumi.Input<inputs.gamelift.FleetEc2InboundPermission>[]>;
    /**
     * Name of an EC2 instance typeE.g., `t2.micro`
     */
    ec2InstanceType: pulumi.Input<string>;
    /**
     * Type of fleet. This value must be `ON_DEMAND` or `SPOT`. Defaults to `ON_DEMAND`.
     */
    fleetType?: pulumi.Input<string>;
    /**
     * ARN of an IAM role that instances in the fleet can assume.
     */
    instanceRoleArn?: pulumi.Input<string>;
    /**
     * List of names of metric groups to add this fleet to. A metric group tracks metrics across all fleets in the group. Defaults to `default`.
     */
    metricGroups?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The name of the fleet.
     */
    name?: pulumi.Input<string>;
    /**
     * Game session protection policy to apply to all instances in this fleetE.g., `FullProtection`. Defaults to `NoProtection`.
     */
    newGameSessionProtectionPolicy?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Policy that limits the number of game sessions an individual player can create over a span of time for this fleet. See below.
     */
    resourceCreationLimitPolicy?: pulumi.Input<inputs.gamelift.FleetResourceCreationLimitPolicy>;
    /**
     * Instructions for launching server processes on each instance in the fleet. See below.
     */
    runtimeConfiguration?: pulumi.Input<inputs.gamelift.FleetRuntimeConfiguration>;
    /**
     * ID of the GameLift Script to be deployed on the fleet. Conflicts with `buildId`.
     */
    scriptId?: pulumi.Input<string>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
