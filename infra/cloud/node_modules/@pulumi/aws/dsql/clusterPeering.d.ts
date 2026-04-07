import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an Amazon Aurora DSQL Cluster Peering.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example1 = new aws.dsql.Cluster("example_1", {multiRegionProperties: {
 *     witnessRegion: "us-west-2",
 * }});
 * const example2 = new aws.dsql.Cluster("example_2", {multiRegionProperties: {
 *     witnessRegion: "us-west-2",
 * }});
 * const example1ClusterPeering = new aws.dsql.ClusterPeering("example_1", {
 *     identifier: example1.identifier,
 *     clusters: [example2.arn],
 *     witnessRegion: example1.multiRegionProperties.apply(multiRegionProperties => multiRegionProperties?.witnessRegion),
 * });
 * const example2ClusterPeering = new aws.dsql.ClusterPeering("example_2", {
 *     identifier: example2.identifier,
 *     clusters: [example1.arn],
 *     witnessRegion: example2.multiRegionProperties.apply(multiRegionProperties => multiRegionProperties?.witnessRegion),
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import DSQL Cluster Peering using the `identifier`. For example:
 *
 * ```sh
 * $ pulumi import aws:dsql/clusterPeering:ClusterPeering example cluster-id-12345678
 * ```
 */
export declare class ClusterPeering extends pulumi.CustomResource {
    /**
     * Get an existing ClusterPeering resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ClusterPeeringState, opts?: pulumi.CustomResourceOptions): ClusterPeering;
    /**
     * Returns true if the given object is an instance of ClusterPeering.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ClusterPeering;
    /**
     * List of DSQL Cluster ARNs to be peered to this cluster.
     */
    readonly clusters: pulumi.Output<string[]>;
    /**
     * DSQL Cluster Identifier.
     */
    readonly identifier: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    readonly timeouts: pulumi.Output<outputs.dsql.ClusterPeeringTimeouts | undefined>;
    /**
     * Witness region for a multi-region cluster.
     */
    readonly witnessRegion: pulumi.Output<string>;
    /**
     * Create a ClusterPeering resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ClusterPeeringArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ClusterPeering resources.
 */
export interface ClusterPeeringState {
    /**
     * List of DSQL Cluster ARNs to be peered to this cluster.
     */
    clusters?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * DSQL Cluster Identifier.
     */
    identifier?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    timeouts?: pulumi.Input<inputs.dsql.ClusterPeeringTimeouts>;
    /**
     * Witness region for a multi-region cluster.
     */
    witnessRegion?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a ClusterPeering resource.
 */
export interface ClusterPeeringArgs {
    /**
     * List of DSQL Cluster ARNs to be peered to this cluster.
     */
    clusters: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * DSQL Cluster Identifier.
     */
    identifier: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    timeouts?: pulumi.Input<inputs.dsql.ClusterPeeringTimeouts>;
    /**
     * Witness region for a multi-region cluster.
     */
    witnessRegion: pulumi.Input<string>;
}
