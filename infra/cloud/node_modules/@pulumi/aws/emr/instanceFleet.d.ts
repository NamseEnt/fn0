import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides an Elastic MapReduce Cluster Instance Fleet configuration.
 * See [Amazon Elastic MapReduce Documentation](https://aws.amazon.com/documentation/emr/) for more information.
 *
 * > **NOTE:** At this time, Instance Fleets cannot be destroyed through the API nor
 * web interface. Instance Fleets are destroyed when the EMR Cluster is destroyed.
 * the provider will resize any Instance Fleet to zero when destroying the resource.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const task = new aws.emr.InstanceFleet("task", {
 *     clusterId: cluster.id,
 *     instanceTypeConfigs: [
 *         {
 *             bidPriceAsPercentageOfOnDemandPrice: 100,
 *             ebsConfigs: [{
 *                 size: 100,
 *                 type: "gp2",
 *                 volumesPerInstance: 1,
 *             }],
 *             instanceType: "m4.xlarge",
 *             weightedCapacity: 1,
 *         },
 *         {
 *             bidPriceAsPercentageOfOnDemandPrice: 100,
 *             ebsConfigs: [{
 *                 size: 100,
 *                 type: "gp2",
 *                 volumesPerInstance: 1,
 *             }],
 *             instanceType: "m4.2xlarge",
 *             weightedCapacity: 2,
 *         },
 *     ],
 *     launchSpecifications: {
 *         spotSpecifications: [{
 *             allocationStrategy: "capacity-optimized",
 *             blockDurationMinutes: 0,
 *             timeoutAction: "TERMINATE_CLUSTER",
 *             timeoutDurationMinutes: 10,
 *         }],
 *     },
 *     name: "task fleet",
 *     targetOnDemandCapacity: 1,
 *     targetSpotCapacity: 1,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import EMR Instance Fleet using the EMR Cluster identifier and Instance Fleet identifier separated by a forward slash (`/`). For example:
 *
 * ```sh
 * $ pulumi import aws:emr/instanceFleet:InstanceFleet example j-123456ABCDEF/if-15EK4O09RZLNR
 * ```
 */
export declare class InstanceFleet extends pulumi.CustomResource {
    /**
     * Get an existing InstanceFleet resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: InstanceFleetState, opts?: pulumi.CustomResourceOptions): InstanceFleet;
    /**
     * Returns true if the given object is an instance of InstanceFleet.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is InstanceFleet;
    /**
     * ID of the EMR Cluster to attach to. Changing this forces a new resource to be created.
     */
    readonly clusterId: pulumi.Output<string>;
    /**
     * Configuration block for instance fleet
     */
    readonly instanceTypeConfigs: pulumi.Output<outputs.emr.InstanceFleetInstanceTypeConfig[] | undefined>;
    /**
     * Configuration block for launch specification
     */
    readonly launchSpecifications: pulumi.Output<outputs.emr.InstanceFleetLaunchSpecifications | undefined>;
    /**
     * Friendly name given to the instance fleet.
     */
    readonly name: pulumi.Output<string>;
    /**
     * The number of On-Demand units that have been provisioned for the instance
     * fleet to fulfill TargetOnDemandCapacity. This provisioned capacity might be less than or greater than TargetOnDemandCapacity.
     */
    readonly provisionedOnDemandCapacity: pulumi.Output<number>;
    /**
     * The number of Spot units that have been provisioned for this instance fleet
     * to fulfill TargetSpotCapacity. This provisioned capacity might be less than or greater than TargetSpotCapacity.
     */
    readonly provisionedSpotCapacity: pulumi.Output<number>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The target capacity of On-Demand units for the instance fleet, which determines how many On-Demand instances to provision.
     */
    readonly targetOnDemandCapacity: pulumi.Output<number | undefined>;
    /**
     * The target capacity of Spot units for the instance fleet, which determines how many Spot instances to provision.
     */
    readonly targetSpotCapacity: pulumi.Output<number | undefined>;
    /**
     * Create a InstanceFleet resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: InstanceFleetArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering InstanceFleet resources.
 */
export interface InstanceFleetState {
    /**
     * ID of the EMR Cluster to attach to. Changing this forces a new resource to be created.
     */
    clusterId?: pulumi.Input<string>;
    /**
     * Configuration block for instance fleet
     */
    instanceTypeConfigs?: pulumi.Input<pulumi.Input<inputs.emr.InstanceFleetInstanceTypeConfig>[]>;
    /**
     * Configuration block for launch specification
     */
    launchSpecifications?: pulumi.Input<inputs.emr.InstanceFleetLaunchSpecifications>;
    /**
     * Friendly name given to the instance fleet.
     */
    name?: pulumi.Input<string>;
    /**
     * The number of On-Demand units that have been provisioned for the instance
     * fleet to fulfill TargetOnDemandCapacity. This provisioned capacity might be less than or greater than TargetOnDemandCapacity.
     */
    provisionedOnDemandCapacity?: pulumi.Input<number>;
    /**
     * The number of Spot units that have been provisioned for this instance fleet
     * to fulfill TargetSpotCapacity. This provisioned capacity might be less than or greater than TargetSpotCapacity.
     */
    provisionedSpotCapacity?: pulumi.Input<number>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The target capacity of On-Demand units for the instance fleet, which determines how many On-Demand instances to provision.
     */
    targetOnDemandCapacity?: pulumi.Input<number>;
    /**
     * The target capacity of Spot units for the instance fleet, which determines how many Spot instances to provision.
     */
    targetSpotCapacity?: pulumi.Input<number>;
}
/**
 * The set of arguments for constructing a InstanceFleet resource.
 */
export interface InstanceFleetArgs {
    /**
     * ID of the EMR Cluster to attach to. Changing this forces a new resource to be created.
     */
    clusterId: pulumi.Input<string>;
    /**
     * Configuration block for instance fleet
     */
    instanceTypeConfigs?: pulumi.Input<pulumi.Input<inputs.emr.InstanceFleetInstanceTypeConfig>[]>;
    /**
     * Configuration block for launch specification
     */
    launchSpecifications?: pulumi.Input<inputs.emr.InstanceFleetLaunchSpecifications>;
    /**
     * Friendly name given to the instance fleet.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The target capacity of On-Demand units for the instance fleet, which determines how many On-Demand instances to provision.
     */
    targetOnDemandCapacity?: pulumi.Input<number>;
    /**
     * The target capacity of Spot units for the instance fleet, which determines how many Spot instances to provision.
     */
    targetSpotCapacity?: pulumi.Input<number>;
}
