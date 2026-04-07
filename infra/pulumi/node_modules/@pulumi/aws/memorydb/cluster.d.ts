import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a MemoryDB Cluster.
 *
 * More information about MemoryDB can be found in the [Developer Guide](https://docs.aws.amazon.com/memorydb/latest/devguide/what-is-memorydb-for-redis.html).
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.memorydb.Cluster("example", {
 *     aclName: "open-access",
 *     name: "my-cluster",
 *     nodeType: "db.t4g.small",
 *     engine: "redis",
 *     engineVersion: "7.1",
 *     numShards: 2,
 *     securityGroupIds: [exampleAwsSecurityGroup.id],
 *     snapshotRetentionLimit: 7,
 *     subnetGroupName: exampleAwsMemorydbSubnetGroup.id,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import a cluster using the `name`. For example:
 *
 * ```sh
 * $ pulumi import aws:memorydb/cluster:Cluster example my-cluster
 * ```
 */
export declare class Cluster extends pulumi.CustomResource {
    /**
     * Get an existing Cluster resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ClusterState, opts?: pulumi.CustomResourceOptions): Cluster;
    /**
     * Returns true if the given object is an instance of Cluster.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Cluster;
    /**
     * The name of the Access Control List to associate with the cluster.
     */
    readonly aclName: pulumi.Output<string>;
    /**
     * The ARN of the cluster.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * When set to `true`, the cluster will automatically receive minor engine version upgrades after launch. Defaults to `true`.
     */
    readonly autoMinorVersionUpgrade: pulumi.Output<boolean | undefined>;
    readonly clusterEndpoints: pulumi.Output<outputs.memorydb.ClusterClusterEndpoint[]>;
    /**
     * Enables data tiering. This option is not supported by all instance types. For more information, see [Data tiering](https://docs.aws.amazon.com/memorydb/latest/devguide/data-tiering.html).
     */
    readonly dataTiering: pulumi.Output<boolean | undefined>;
    /**
     * Description for the cluster. Defaults to `"Managed by Pulumi"`.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * The engine that will run on your nodes. Supported values are `redis` and `valkey`.
     */
    readonly engine: pulumi.Output<string>;
    /**
     * Patch version number of the engine used by the cluster.
     */
    readonly enginePatchVersion: pulumi.Output<string>;
    /**
     * Version number of the engine to be used for the cluster. Downgrades are not supported.
     */
    readonly engineVersion: pulumi.Output<string>;
    /**
     * Name of the final cluster snapshot to be created when this resource is deleted. If omitted, no final snapshot will be made.
     */
    readonly finalSnapshotName: pulumi.Output<string | undefined>;
    /**
     * ARN of the KMS key used to encrypt the cluster at rest.
     */
    readonly kmsKeyArn: pulumi.Output<string | undefined>;
    /**
     * Specifies the weekly time range during which maintenance on the cluster is performed. Specify as a range in the format `ddd:hh24:mi-ddd:hh24:mi` (24H Clock UTC). The minimum maintenance window is a 60 minute period. Example: `sun:23:00-mon:01:30`.
     */
    readonly maintenanceWindow: pulumi.Output<string>;
    /**
     * The multi region cluster identifier specified on `aws.memorydb.MultiRegionCluster`.
     */
    readonly multiRegionClusterName: pulumi.Output<string | undefined>;
    /**
     * Name of the cluster. If omitted, the provider will assign a random, unique name. Conflicts with `namePrefix`.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Creates a unique name beginning with the specified prefix. Conflicts with `name`.
     */
    readonly namePrefix: pulumi.Output<string>;
    /**
     * The compute and memory capacity of the nodes in the cluster. See AWS documentation on [supported node types](https://docs.aws.amazon.com/memorydb/latest/devguide/nodes.supportedtypes.html) as well as [vertical scaling](https://docs.aws.amazon.com/memorydb/latest/devguide/cluster-vertical-scaling.html).
     *
     * The following arguments are optional:
     */
    readonly nodeType: pulumi.Output<string>;
    /**
     * The number of replicas to apply to each shard, up to a maximum of 5. Defaults to `1` (i.e. 2 nodes per shard).
     */
    readonly numReplicasPerShard: pulumi.Output<number | undefined>;
    /**
     * The number of shards in the cluster. Defaults to `1`.
     */
    readonly numShards: pulumi.Output<number | undefined>;
    /**
     * The name of the parameter group associated with the cluster.
     */
    readonly parameterGroupName: pulumi.Output<string>;
    /**
     * The port number on which each of the nodes accepts connections. Defaults to `6379`.
     */
    readonly port: pulumi.Output<number>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Set of VPC Security Group ID-s to associate with this cluster.
     */
    readonly securityGroupIds: pulumi.Output<string[] | undefined>;
    /**
     * Set of shards in this cluster.
     */
    readonly shards: pulumi.Output<outputs.memorydb.ClusterShard[]>;
    /**
     * List of ARN-s that uniquely identify RDB snapshot files stored in S3. The snapshot files will be used to populate the new cluster. Object names in the ARN-s cannot contain any commas.
     */
    readonly snapshotArns: pulumi.Output<string[] | undefined>;
    /**
     * The name of a snapshot from which to restore data into the new cluster.
     */
    readonly snapshotName: pulumi.Output<string | undefined>;
    /**
     * The number of days for which MemoryDB retains automatic snapshots before deleting them. When set to `0`, automatic backups are disabled. Defaults to `0`.
     */
    readonly snapshotRetentionLimit: pulumi.Output<number>;
    /**
     * The daily time range (in UTC) during which MemoryDB begins taking a daily snapshot of your shard. Example: `05:00-09:00`.
     */
    readonly snapshotWindow: pulumi.Output<string>;
    /**
     * ARN of the SNS topic to which cluster notifications are sent.
     */
    readonly snsTopicArn: pulumi.Output<string | undefined>;
    /**
     * The name of the subnet group to be used for the cluster. Defaults to a subnet group consisting of default VPC subnets.
     */
    readonly subnetGroupName: pulumi.Output<string>;
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
     * A flag to enable in-transit encryption on the cluster. When set to `false`, the `aclName` must be `open-access`. Defaults to `true`.
     */
    readonly tlsEnabled: pulumi.Output<boolean | undefined>;
    /**
     * Create a Cluster resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ClusterArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Cluster resources.
 */
export interface ClusterState {
    /**
     * The name of the Access Control List to associate with the cluster.
     */
    aclName?: pulumi.Input<string>;
    /**
     * The ARN of the cluster.
     */
    arn?: pulumi.Input<string>;
    /**
     * When set to `true`, the cluster will automatically receive minor engine version upgrades after launch. Defaults to `true`.
     */
    autoMinorVersionUpgrade?: pulumi.Input<boolean>;
    clusterEndpoints?: pulumi.Input<pulumi.Input<inputs.memorydb.ClusterClusterEndpoint>[]>;
    /**
     * Enables data tiering. This option is not supported by all instance types. For more information, see [Data tiering](https://docs.aws.amazon.com/memorydb/latest/devguide/data-tiering.html).
     */
    dataTiering?: pulumi.Input<boolean>;
    /**
     * Description for the cluster. Defaults to `"Managed by Pulumi"`.
     */
    description?: pulumi.Input<string>;
    /**
     * The engine that will run on your nodes. Supported values are `redis` and `valkey`.
     */
    engine?: pulumi.Input<string>;
    /**
     * Patch version number of the engine used by the cluster.
     */
    enginePatchVersion?: pulumi.Input<string>;
    /**
     * Version number of the engine to be used for the cluster. Downgrades are not supported.
     */
    engineVersion?: pulumi.Input<string>;
    /**
     * Name of the final cluster snapshot to be created when this resource is deleted. If omitted, no final snapshot will be made.
     */
    finalSnapshotName?: pulumi.Input<string>;
    /**
     * ARN of the KMS key used to encrypt the cluster at rest.
     */
    kmsKeyArn?: pulumi.Input<string>;
    /**
     * Specifies the weekly time range during which maintenance on the cluster is performed. Specify as a range in the format `ddd:hh24:mi-ddd:hh24:mi` (24H Clock UTC). The minimum maintenance window is a 60 minute period. Example: `sun:23:00-mon:01:30`.
     */
    maintenanceWindow?: pulumi.Input<string>;
    /**
     * The multi region cluster identifier specified on `aws.memorydb.MultiRegionCluster`.
     */
    multiRegionClusterName?: pulumi.Input<string>;
    /**
     * Name of the cluster. If omitted, the provider will assign a random, unique name. Conflicts with `namePrefix`.
     */
    name?: pulumi.Input<string>;
    /**
     * Creates a unique name beginning with the specified prefix. Conflicts with `name`.
     */
    namePrefix?: pulumi.Input<string>;
    /**
     * The compute and memory capacity of the nodes in the cluster. See AWS documentation on [supported node types](https://docs.aws.amazon.com/memorydb/latest/devguide/nodes.supportedtypes.html) as well as [vertical scaling](https://docs.aws.amazon.com/memorydb/latest/devguide/cluster-vertical-scaling.html).
     *
     * The following arguments are optional:
     */
    nodeType?: pulumi.Input<string>;
    /**
     * The number of replicas to apply to each shard, up to a maximum of 5. Defaults to `1` (i.e. 2 nodes per shard).
     */
    numReplicasPerShard?: pulumi.Input<number>;
    /**
     * The number of shards in the cluster. Defaults to `1`.
     */
    numShards?: pulumi.Input<number>;
    /**
     * The name of the parameter group associated with the cluster.
     */
    parameterGroupName?: pulumi.Input<string>;
    /**
     * The port number on which each of the nodes accepts connections. Defaults to `6379`.
     */
    port?: pulumi.Input<number>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Set of VPC Security Group ID-s to associate with this cluster.
     */
    securityGroupIds?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Set of shards in this cluster.
     */
    shards?: pulumi.Input<pulumi.Input<inputs.memorydb.ClusterShard>[]>;
    /**
     * List of ARN-s that uniquely identify RDB snapshot files stored in S3. The snapshot files will be used to populate the new cluster. Object names in the ARN-s cannot contain any commas.
     */
    snapshotArns?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The name of a snapshot from which to restore data into the new cluster.
     */
    snapshotName?: pulumi.Input<string>;
    /**
     * The number of days for which MemoryDB retains automatic snapshots before deleting them. When set to `0`, automatic backups are disabled. Defaults to `0`.
     */
    snapshotRetentionLimit?: pulumi.Input<number>;
    /**
     * The daily time range (in UTC) during which MemoryDB begins taking a daily snapshot of your shard. Example: `05:00-09:00`.
     */
    snapshotWindow?: pulumi.Input<string>;
    /**
     * ARN of the SNS topic to which cluster notifications are sent.
     */
    snsTopicArn?: pulumi.Input<string>;
    /**
     * The name of the subnet group to be used for the cluster. Defaults to a subnet group consisting of default VPC subnets.
     */
    subnetGroupName?: pulumi.Input<string>;
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
    /**
     * A flag to enable in-transit encryption on the cluster. When set to `false`, the `aclName` must be `open-access`. Defaults to `true`.
     */
    tlsEnabled?: pulumi.Input<boolean>;
}
/**
 * The set of arguments for constructing a Cluster resource.
 */
export interface ClusterArgs {
    /**
     * The name of the Access Control List to associate with the cluster.
     */
    aclName: pulumi.Input<string>;
    /**
     * When set to `true`, the cluster will automatically receive minor engine version upgrades after launch. Defaults to `true`.
     */
    autoMinorVersionUpgrade?: pulumi.Input<boolean>;
    /**
     * Enables data tiering. This option is not supported by all instance types. For more information, see [Data tiering](https://docs.aws.amazon.com/memorydb/latest/devguide/data-tiering.html).
     */
    dataTiering?: pulumi.Input<boolean>;
    /**
     * Description for the cluster. Defaults to `"Managed by Pulumi"`.
     */
    description?: pulumi.Input<string>;
    /**
     * The engine that will run on your nodes. Supported values are `redis` and `valkey`.
     */
    engine?: pulumi.Input<string>;
    /**
     * Version number of the engine to be used for the cluster. Downgrades are not supported.
     */
    engineVersion?: pulumi.Input<string>;
    /**
     * Name of the final cluster snapshot to be created when this resource is deleted. If omitted, no final snapshot will be made.
     */
    finalSnapshotName?: pulumi.Input<string>;
    /**
     * ARN of the KMS key used to encrypt the cluster at rest.
     */
    kmsKeyArn?: pulumi.Input<string>;
    /**
     * Specifies the weekly time range during which maintenance on the cluster is performed. Specify as a range in the format `ddd:hh24:mi-ddd:hh24:mi` (24H Clock UTC). The minimum maintenance window is a 60 minute period. Example: `sun:23:00-mon:01:30`.
     */
    maintenanceWindow?: pulumi.Input<string>;
    /**
     * The multi region cluster identifier specified on `aws.memorydb.MultiRegionCluster`.
     */
    multiRegionClusterName?: pulumi.Input<string>;
    /**
     * Name of the cluster. If omitted, the provider will assign a random, unique name. Conflicts with `namePrefix`.
     */
    name?: pulumi.Input<string>;
    /**
     * Creates a unique name beginning with the specified prefix. Conflicts with `name`.
     */
    namePrefix?: pulumi.Input<string>;
    /**
     * The compute and memory capacity of the nodes in the cluster. See AWS documentation on [supported node types](https://docs.aws.amazon.com/memorydb/latest/devguide/nodes.supportedtypes.html) as well as [vertical scaling](https://docs.aws.amazon.com/memorydb/latest/devguide/cluster-vertical-scaling.html).
     *
     * The following arguments are optional:
     */
    nodeType: pulumi.Input<string>;
    /**
     * The number of replicas to apply to each shard, up to a maximum of 5. Defaults to `1` (i.e. 2 nodes per shard).
     */
    numReplicasPerShard?: pulumi.Input<number>;
    /**
     * The number of shards in the cluster. Defaults to `1`.
     */
    numShards?: pulumi.Input<number>;
    /**
     * The name of the parameter group associated with the cluster.
     */
    parameterGroupName?: pulumi.Input<string>;
    /**
     * The port number on which each of the nodes accepts connections. Defaults to `6379`.
     */
    port?: pulumi.Input<number>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Set of VPC Security Group ID-s to associate with this cluster.
     */
    securityGroupIds?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * List of ARN-s that uniquely identify RDB snapshot files stored in S3. The snapshot files will be used to populate the new cluster. Object names in the ARN-s cannot contain any commas.
     */
    snapshotArns?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The name of a snapshot from which to restore data into the new cluster.
     */
    snapshotName?: pulumi.Input<string>;
    /**
     * The number of days for which MemoryDB retains automatic snapshots before deleting them. When set to `0`, automatic backups are disabled. Defaults to `0`.
     */
    snapshotRetentionLimit?: pulumi.Input<number>;
    /**
     * The daily time range (in UTC) during which MemoryDB begins taking a daily snapshot of your shard. Example: `05:00-09:00`.
     */
    snapshotWindow?: pulumi.Input<string>;
    /**
     * ARN of the SNS topic to which cluster notifications are sent.
     */
    snsTopicArn?: pulumi.Input<string>;
    /**
     * The name of the subnet group to be used for the cluster. Defaults to a subnet group consisting of default VPC subnets.
     */
    subnetGroupName?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * A flag to enable in-transit encryption on the cluster. When set to `false`, the `aclName` must be `open-access`. Defaults to `true`.
     */
    tlsEnabled?: pulumi.Input<boolean>;
}
