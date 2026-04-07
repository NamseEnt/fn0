import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a resource to manage EC2 Fleets.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.ec2.Fleet("example", {
 *     launchTemplateConfigs: [{
 *         launchTemplateSpecification: {
 *             launchTemplateId: exampleAwsLaunchTemplate.id,
 *             version: exampleAwsLaunchTemplate.latestVersion,
 *         },
 *     }],
 *     targetCapacitySpecification: {
 *         defaultTargetCapacityType: "spot",
 *         totalTargetCapacity: 5,
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_ec2_fleet` using the Fleet identifier. For example:
 *
 * ```sh
 * $ pulumi import aws:ec2/fleet:Fleet example fleet-b9b55d27-c5fc-41ac-a6f3-48fcc91f080c
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
     * The ARN of the fleet
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Reserved.
     */
    readonly context: pulumi.Output<string | undefined>;
    /**
     * Whether running instances should be terminated if the total target capacity of the EC2 Fleet is decreased below the current size of the EC2. Valid values: `no-termination`, `termination`. Defaults to `termination`. Supported only for fleets of type `maintain`.
     */
    readonly excessCapacityTerminationPolicy: pulumi.Output<string | undefined>;
    /**
     * Information about the instances that were launched by the fleet. Available only when `type` is set to `instant`.
     */
    readonly fleetInstanceSets: pulumi.Output<outputs.ec2.FleetFleetInstanceSet[]>;
    /**
     * The state of the EC2 Fleet.
     */
    readonly fleetState: pulumi.Output<string>;
    /**
     * The number of units fulfilled by this request compared to the set target capacity.
     */
    readonly fulfilledCapacity: pulumi.Output<number>;
    /**
     * The number of units fulfilled by this request compared to the set target On-Demand capacity.
     */
    readonly fulfilledOnDemandCapacity: pulumi.Output<number>;
    /**
     * Nested argument containing EC2 Launch Template configurations. Defined below.
     */
    readonly launchTemplateConfigs: pulumi.Output<outputs.ec2.FleetLaunchTemplateConfig[]>;
    /**
     * Nested argument containing On-Demand configurations. Defined below.
     */
    readonly onDemandOptions: pulumi.Output<outputs.ec2.FleetOnDemandOptions | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Whether EC2 Fleet should replace unhealthy instances. Defaults to `false`. Supported only for fleets of type `maintain`.
     */
    readonly replaceUnhealthyInstances: pulumi.Output<boolean | undefined>;
    /**
     * Nested argument containing Spot configurations. Defined below.
     */
    readonly spotOptions: pulumi.Output<outputs.ec2.FleetSpotOptions | undefined>;
    /**
     * Map of Fleet tags. To tag instances at launch, specify the tags in the Launch Template. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Nested argument containing target capacity configurations. Defined below.
     */
    readonly targetCapacitySpecification: pulumi.Output<outputs.ec2.FleetTargetCapacitySpecification>;
    /**
     * Whether to terminate instances for an EC2 Fleet if it is deleted successfully. Defaults to `false`.
     */
    readonly terminateInstances: pulumi.Output<boolean | undefined>;
    /**
     * Whether running instances should be terminated when the EC2 Fleet expires. Defaults to `false`.
     */
    readonly terminateInstancesWithExpiration: pulumi.Output<boolean | undefined>;
    /**
     * The type of request. Indicates whether the EC2 Fleet only requests the target capacity, or also attempts to maintain it. Valid values: `maintain`, `request`, `instant`. Defaults to `maintain`.
     */
    readonly type: pulumi.Output<string | undefined>;
    /**
     * The start date and time of the request, in UTC format (for example, YYYY-MM-DDTHH:MM:SSZ). The default is to start fulfilling the request immediately.
     */
    readonly validFrom: pulumi.Output<string | undefined>;
    /**
     * The end date and time of the request, in UTC format (for example, YYYY-MM-DDTHH:MM:SSZ). At this point, no new EC2 Fleet requests are placed or able to fulfill the request. If no value is specified, the request remains until you cancel it.
     */
    readonly validUntil: pulumi.Output<string | undefined>;
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
     * The ARN of the fleet
     */
    arn?: pulumi.Input<string>;
    /**
     * Reserved.
     */
    context?: pulumi.Input<string>;
    /**
     * Whether running instances should be terminated if the total target capacity of the EC2 Fleet is decreased below the current size of the EC2. Valid values: `no-termination`, `termination`. Defaults to `termination`. Supported only for fleets of type `maintain`.
     */
    excessCapacityTerminationPolicy?: pulumi.Input<string>;
    /**
     * Information about the instances that were launched by the fleet. Available only when `type` is set to `instant`.
     */
    fleetInstanceSets?: pulumi.Input<pulumi.Input<inputs.ec2.FleetFleetInstanceSet>[]>;
    /**
     * The state of the EC2 Fleet.
     */
    fleetState?: pulumi.Input<string>;
    /**
     * The number of units fulfilled by this request compared to the set target capacity.
     */
    fulfilledCapacity?: pulumi.Input<number>;
    /**
     * The number of units fulfilled by this request compared to the set target On-Demand capacity.
     */
    fulfilledOnDemandCapacity?: pulumi.Input<number>;
    /**
     * Nested argument containing EC2 Launch Template configurations. Defined below.
     */
    launchTemplateConfigs?: pulumi.Input<pulumi.Input<inputs.ec2.FleetLaunchTemplateConfig>[]>;
    /**
     * Nested argument containing On-Demand configurations. Defined below.
     */
    onDemandOptions?: pulumi.Input<inputs.ec2.FleetOnDemandOptions>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Whether EC2 Fleet should replace unhealthy instances. Defaults to `false`. Supported only for fleets of type `maintain`.
     */
    replaceUnhealthyInstances?: pulumi.Input<boolean>;
    /**
     * Nested argument containing Spot configurations. Defined below.
     */
    spotOptions?: pulumi.Input<inputs.ec2.FleetSpotOptions>;
    /**
     * Map of Fleet tags. To tag instances at launch, specify the tags in the Launch Template. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
    /**
     * Nested argument containing target capacity configurations. Defined below.
     */
    targetCapacitySpecification?: pulumi.Input<inputs.ec2.FleetTargetCapacitySpecification>;
    /**
     * Whether to terminate instances for an EC2 Fleet if it is deleted successfully. Defaults to `false`.
     */
    terminateInstances?: pulumi.Input<boolean>;
    /**
     * Whether running instances should be terminated when the EC2 Fleet expires. Defaults to `false`.
     */
    terminateInstancesWithExpiration?: pulumi.Input<boolean>;
    /**
     * The type of request. Indicates whether the EC2 Fleet only requests the target capacity, or also attempts to maintain it. Valid values: `maintain`, `request`, `instant`. Defaults to `maintain`.
     */
    type?: pulumi.Input<string>;
    /**
     * The start date and time of the request, in UTC format (for example, YYYY-MM-DDTHH:MM:SSZ). The default is to start fulfilling the request immediately.
     */
    validFrom?: pulumi.Input<string>;
    /**
     * The end date and time of the request, in UTC format (for example, YYYY-MM-DDTHH:MM:SSZ). At this point, no new EC2 Fleet requests are placed or able to fulfill the request. If no value is specified, the request remains until you cancel it.
     */
    validUntil?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a Fleet resource.
 */
export interface FleetArgs {
    /**
     * Reserved.
     */
    context?: pulumi.Input<string>;
    /**
     * Whether running instances should be terminated if the total target capacity of the EC2 Fleet is decreased below the current size of the EC2. Valid values: `no-termination`, `termination`. Defaults to `termination`. Supported only for fleets of type `maintain`.
     */
    excessCapacityTerminationPolicy?: pulumi.Input<string>;
    /**
     * Information about the instances that were launched by the fleet. Available only when `type` is set to `instant`.
     */
    fleetInstanceSets?: pulumi.Input<pulumi.Input<inputs.ec2.FleetFleetInstanceSet>[]>;
    /**
     * The state of the EC2 Fleet.
     */
    fleetState?: pulumi.Input<string>;
    /**
     * The number of units fulfilled by this request compared to the set target capacity.
     */
    fulfilledCapacity?: pulumi.Input<number>;
    /**
     * The number of units fulfilled by this request compared to the set target On-Demand capacity.
     */
    fulfilledOnDemandCapacity?: pulumi.Input<number>;
    /**
     * Nested argument containing EC2 Launch Template configurations. Defined below.
     */
    launchTemplateConfigs: pulumi.Input<pulumi.Input<inputs.ec2.FleetLaunchTemplateConfig>[]>;
    /**
     * Nested argument containing On-Demand configurations. Defined below.
     */
    onDemandOptions?: pulumi.Input<inputs.ec2.FleetOnDemandOptions>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Whether EC2 Fleet should replace unhealthy instances. Defaults to `false`. Supported only for fleets of type `maintain`.
     */
    replaceUnhealthyInstances?: pulumi.Input<boolean>;
    /**
     * Nested argument containing Spot configurations. Defined below.
     */
    spotOptions?: pulumi.Input<inputs.ec2.FleetSpotOptions>;
    /**
     * Map of Fleet tags. To tag instances at launch, specify the tags in the Launch Template. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Nested argument containing target capacity configurations. Defined below.
     */
    targetCapacitySpecification: pulumi.Input<inputs.ec2.FleetTargetCapacitySpecification>;
    /**
     * Whether to terminate instances for an EC2 Fleet if it is deleted successfully. Defaults to `false`.
     */
    terminateInstances?: pulumi.Input<boolean>;
    /**
     * Whether running instances should be terminated when the EC2 Fleet expires. Defaults to `false`.
     */
    terminateInstancesWithExpiration?: pulumi.Input<boolean>;
    /**
     * The type of request. Indicates whether the EC2 Fleet only requests the target capacity, or also attempts to maintain it. Valid values: `maintain`, `request`, `instant`. Defaults to `maintain`.
     */
    type?: pulumi.Input<string>;
    /**
     * The start date and time of the request, in UTC format (for example, YYYY-MM-DDTHH:MM:SSZ). The default is to start fulfilling the request immediately.
     */
    validFrom?: pulumi.Input<string>;
    /**
     * The end date and time of the request, in UTC format (for example, YYYY-MM-DDTHH:MM:SSZ). At this point, no new EC2 Fleet requests are placed or able to fulfill the request. If no value is specified, the request remains until you cancel it.
     */
    validUntil?: pulumi.Input<string>;
}
