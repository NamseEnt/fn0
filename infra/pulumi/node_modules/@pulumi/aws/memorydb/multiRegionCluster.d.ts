import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a MemoryDB Multi Region Cluster.
 *
 * More information about MemoryDB can be found in the [Developer Guide](https://docs.aws.amazon.com/memorydb/latest/devguide/what-is-memorydb-for-redis.html).
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.memorydb.MultiRegionCluster("example", {
 *     multiRegionClusterNameSuffix: "example",
 *     nodeType: "db.r7g.xlarge",
 * });
 * const exampleCluster = new aws.memorydb.Cluster("example", {
 *     aclName: exampleAwsMemorydbAcl.id,
 *     autoMinorVersionUpgrade: false,
 *     name: "example",
 *     nodeType: "db.t4g.small",
 *     numShards: 2,
 *     securityGroupIds: [exampleAwsSecurityGroup.id],
 *     snapshotRetentionLimit: 7,
 *     subnetGroupName: exampleAwsMemorydbSubnetGroup.id,
 *     multiRegionClusterName: example.multiRegionClusterName,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import a cluster using the `multi_region_cluster_name`. For example:
 *
 * ```sh
 * $ pulumi import aws:memorydb/multiRegionCluster:MultiRegionCluster example virxk-example
 * ```
 */
export declare class MultiRegionCluster extends pulumi.CustomResource {
    /**
     * Get an existing MultiRegionCluster resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: MultiRegionClusterState, opts?: pulumi.CustomResourceOptions): MultiRegionCluster;
    /**
     * Returns true if the given object is an instance of MultiRegionCluster.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is MultiRegionCluster;
    /**
     * The ARN of the multi-region cluster.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * description for the multi-region cluster.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * The name of the engine to be used for the multi-region cluster. Valid values are `redis` and `valkey`.
     */
    readonly engine: pulumi.Output<string>;
    /**
     * The version of the engine to be used for the multi-region cluster. Downgrades are not supported.
     */
    readonly engineVersion: pulumi.Output<string>;
    /**
     * The name of the multi-region cluster.
     */
    readonly multiRegionClusterName: pulumi.Output<string>;
    /**
     * A suffix to be added to the multi-region cluster name. An AWS generated prefix is automatically applied to the multi-region cluster name when it is created.
     */
    readonly multiRegionClusterNameSuffix: pulumi.Output<string>;
    /**
     * The name of the multi-region parameter group to be associated with the cluster.
     */
    readonly multiRegionParameterGroupName: pulumi.Output<string>;
    /**
     * The node type to be used for the multi-region cluster.
     *
     * The following arguments are optional:
     */
    readonly nodeType: pulumi.Output<string>;
    /**
     * The number of shards for the multi-region cluster.
     */
    readonly numShards: pulumi.Output<number>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    readonly status: pulumi.Output<string>;
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
    readonly timeouts: pulumi.Output<outputs.memorydb.MultiRegionClusterTimeouts | undefined>;
    /**
     * A flag to enable in-transit encryption on the cluster.
     */
    readonly tlsEnabled: pulumi.Output<boolean>;
    readonly updateStrategy: pulumi.Output<string | undefined>;
    /**
     * Create a MultiRegionCluster resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: MultiRegionClusterArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering MultiRegionCluster resources.
 */
export interface MultiRegionClusterState {
    /**
     * The ARN of the multi-region cluster.
     */
    arn?: pulumi.Input<string>;
    /**
     * description for the multi-region cluster.
     */
    description?: pulumi.Input<string>;
    /**
     * The name of the engine to be used for the multi-region cluster. Valid values are `redis` and `valkey`.
     */
    engine?: pulumi.Input<string>;
    /**
     * The version of the engine to be used for the multi-region cluster. Downgrades are not supported.
     */
    engineVersion?: pulumi.Input<string>;
    /**
     * The name of the multi-region cluster.
     */
    multiRegionClusterName?: pulumi.Input<string>;
    /**
     * A suffix to be added to the multi-region cluster name. An AWS generated prefix is automatically applied to the multi-region cluster name when it is created.
     */
    multiRegionClusterNameSuffix?: pulumi.Input<string>;
    /**
     * The name of the multi-region parameter group to be associated with the cluster.
     */
    multiRegionParameterGroupName?: pulumi.Input<string>;
    /**
     * The node type to be used for the multi-region cluster.
     *
     * The following arguments are optional:
     */
    nodeType?: pulumi.Input<string>;
    /**
     * The number of shards for the multi-region cluster.
     */
    numShards?: pulumi.Input<number>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    status?: pulumi.Input<string>;
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
    timeouts?: pulumi.Input<inputs.memorydb.MultiRegionClusterTimeouts>;
    /**
     * A flag to enable in-transit encryption on the cluster.
     */
    tlsEnabled?: pulumi.Input<boolean>;
    updateStrategy?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a MultiRegionCluster resource.
 */
export interface MultiRegionClusterArgs {
    /**
     * description for the multi-region cluster.
     */
    description?: pulumi.Input<string>;
    /**
     * The name of the engine to be used for the multi-region cluster. Valid values are `redis` and `valkey`.
     */
    engine?: pulumi.Input<string>;
    /**
     * The version of the engine to be used for the multi-region cluster. Downgrades are not supported.
     */
    engineVersion?: pulumi.Input<string>;
    /**
     * A suffix to be added to the multi-region cluster name. An AWS generated prefix is automatically applied to the multi-region cluster name when it is created.
     */
    multiRegionClusterNameSuffix: pulumi.Input<string>;
    /**
     * The name of the multi-region parameter group to be associated with the cluster.
     */
    multiRegionParameterGroupName?: pulumi.Input<string>;
    /**
     * The node type to be used for the multi-region cluster.
     *
     * The following arguments are optional:
     */
    nodeType: pulumi.Input<string>;
    /**
     * The number of shards for the multi-region cluster.
     */
    numShards?: pulumi.Input<number>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    timeouts?: pulumi.Input<inputs.memorydb.MultiRegionClusterTimeouts>;
    /**
     * A flag to enable in-transit encryption on the cluster.
     */
    tlsEnabled?: pulumi.Input<boolean>;
    updateStrategy?: pulumi.Input<string>;
}
