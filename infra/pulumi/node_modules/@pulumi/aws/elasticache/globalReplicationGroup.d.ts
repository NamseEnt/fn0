import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides an ElastiCache Global Replication Group resource, which manages replication between two or more Replication Groups in different regions. For more information, see the [ElastiCache User Guide](https://docs.aws.amazon.com/AmazonElastiCache/latest/red-ug/Redis-Global-Datastore.html).
 *
 * ## Example Usage
 *
 * ### Global replication group with one secondary replication group
 *
 * The global replication group depends on the primary group existing. Secondary replication groups depend on the global replication group. the provider dependency management will handle this transparently using resource value references.
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const primary = new aws.elasticache.ReplicationGroup("primary", {
 *     replicationGroupId: "example-primary",
 *     description: "primary replication group",
 *     engine: "redis",
 *     engineVersion: "5.0.6",
 *     nodeType: "cache.m5.large",
 *     numCacheClusters: 1,
 * });
 * const example = new aws.elasticache.GlobalReplicationGroup("example", {
 *     globalReplicationGroupIdSuffix: "example",
 *     primaryReplicationGroupId: primary.id,
 * });
 * const secondary = new aws.elasticache.ReplicationGroup("secondary", {
 *     replicationGroupId: "example-secondary",
 *     description: "secondary replication group",
 *     globalReplicationGroupId: example.globalReplicationGroupId,
 *     numCacheClusters: 1,
 * });
 * ```
 *
 * ### Managing Redis OOS/Valkey Engine Versions
 *
 * The initial Redis version is determined by the version set on the primary replication group.
 * However, once it is part of a Global Replication Group,
 * the Global Replication Group manages the version of all member replication groups.
 *
 * The provider is configured to ignore changes to `engine`, `engineVersion` and `parameterGroupName` inside `aws.elasticache.ReplicationGroup` resources if they belong to a global replication group.
 *
 * In this example,
 * the primary replication group will be created with Redis 6.0,
 * and then upgraded to Redis 6.2 once added to the Global Replication Group.
 * The secondary replication group will be created with Redis 6.2.
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const primary = new aws.elasticache.ReplicationGroup("primary", {
 *     replicationGroupId: "example-primary",
 *     description: "primary replication group",
 *     engine: "redis",
 *     engineVersion: "6.0",
 *     nodeType: "cache.m5.large",
 *     numCacheClusters: 1,
 * });
 * const example = new aws.elasticache.GlobalReplicationGroup("example", {
 *     globalReplicationGroupIdSuffix: "example",
 *     primaryReplicationGroupId: primary.id,
 *     engineVersion: "6.2",
 * });
 * const secondary = new aws.elasticache.ReplicationGroup("secondary", {
 *     replicationGroupId: "example-secondary",
 *     description: "secondary replication group",
 *     globalReplicationGroupId: example.globalReplicationGroupId,
 *     numCacheClusters: 1,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import ElastiCache Global Replication Groups using the `global_replication_group_id`. For example:
 *
 * ```sh
 * $ pulumi import aws:elasticache/globalReplicationGroup:GlobalReplicationGroup my_global_replication_group okuqm-global-replication-group-1
 * ```
 */
export declare class GlobalReplicationGroup extends pulumi.CustomResource {
    /**
     * Get an existing GlobalReplicationGroup resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: GlobalReplicationGroupState, opts?: pulumi.CustomResourceOptions): GlobalReplicationGroup;
    /**
     * Returns true if the given object is an instance of GlobalReplicationGroup.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is GlobalReplicationGroup;
    /**
     * The ARN of the ElastiCache Global Replication Group.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * A flag that indicate whether the encryption at rest is enabled.
     */
    readonly atRestEncryptionEnabled: pulumi.Output<boolean>;
    /**
     * A flag that indicate whether AuthToken (password) is enabled.
     */
    readonly authTokenEnabled: pulumi.Output<boolean>;
    /**
     * Specifies whether read-only replicas will be automatically promoted to read/write primary if the existing primary fails.
     * When creating, by default the Global Replication Group inherits the automatic failover setting of the primary replication group.
     */
    readonly automaticFailoverEnabled: pulumi.Output<boolean>;
    /**
     * The instance class used.
     * See AWS documentation for information on [supported node types](https://docs.aws.amazon.com/AmazonElastiCache/latest/red-ug/CacheNodes.SupportedTypes.html)
     * and [guidance on selecting node types](https://docs.aws.amazon.com/AmazonElastiCache/latest/red-ug/nodes-select-size.html).
     * When creating, by default the Global Replication Group inherits the node type of the primary replication group.
     */
    readonly cacheNodeType: pulumi.Output<string>;
    /**
     * Indicates whether the Global Datastore is cluster enabled.
     */
    readonly clusterEnabled: pulumi.Output<boolean>;
    /**
     * The name of the cache engine to be used for the clusters in this global replication group.
     * When creating, by default the Global Replication Group inherits the engine of the primary replication group.
     * If an engine is specified, the Global Replication Group and all member replication groups will be upgraded to this engine.
     * Valid values are `redis` or `valkey`.
     * Default is `redis` if `engineVersion` is specified.
     */
    readonly engine: pulumi.Output<string>;
    /**
     * Engine version to use for the Global Replication Group.
     * When creating, by default the Global Replication Group inherits the version of the primary replication group.
     * If a version is specified, the Global Replication Group and all member replication groups will be upgraded to this version.
     * Cannot be downgraded without replacing the Global Replication Group and all member replication groups.
     * When the version is 7 or higher, the major and minor version should be set, e.g., `7.2`.
     * When the version is 6, the major and minor version can be set, e.g., `6.2`,
     * or the minor version can be unspecified which will use the latest version at creation time, e.g., `6.x`.
     * The actual engine version used is returned in the attribute `engineVersionActual`, see Attribute Reference below.
     */
    readonly engineVersion: pulumi.Output<string>;
    /**
     * The full version number of the cache engine running on the members of this global replication group.
     */
    readonly engineVersionActual: pulumi.Output<string>;
    /**
     * Set of node groups (shards) on the global replication group.
     * Has the values:
     */
    readonly globalNodeGroups: pulumi.Output<outputs.elasticache.GlobalReplicationGroupGlobalNodeGroup[]>;
    /**
     * A user-created description for the global replication group.
     */
    readonly globalReplicationGroupDescription: pulumi.Output<string | undefined>;
    /**
     * The full ID of the global replication group.
     */
    readonly globalReplicationGroupId: pulumi.Output<string>;
    /**
     * The suffix name of a Global Datastore. If `globalReplicationGroupIdSuffix` is changed, creates a new resource.
     */
    readonly globalReplicationGroupIdSuffix: pulumi.Output<string>;
    /**
     * The number of node groups (shards) on the global replication group.
     */
    readonly numNodeGroups: pulumi.Output<number>;
    /**
     * An ElastiCache Parameter Group to use for the Global Replication Group.
     * Required when upgrading an engine or major engine version, but will be ignored if left configured after the upgrade is complete.
     * Specifying without a major version upgrade will fail.
     * Note that ElastiCache creates a copy of this parameter group for each member replication group.
     */
    readonly parameterGroupName: pulumi.Output<string | undefined>;
    /**
     * The ID of the primary cluster that accepts writes and will replicate updates to the secondary cluster. If `primaryReplicationGroupId` is changed, creates a new resource.
     */
    readonly primaryReplicationGroupId: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * A flag that indicates whether the encryption in transit is enabled.
     */
    readonly transitEncryptionEnabled: pulumi.Output<boolean>;
    /**
     * Create a GlobalReplicationGroup resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: GlobalReplicationGroupArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering GlobalReplicationGroup resources.
 */
export interface GlobalReplicationGroupState {
    /**
     * The ARN of the ElastiCache Global Replication Group.
     */
    arn?: pulumi.Input<string>;
    /**
     * A flag that indicate whether the encryption at rest is enabled.
     */
    atRestEncryptionEnabled?: pulumi.Input<boolean>;
    /**
     * A flag that indicate whether AuthToken (password) is enabled.
     */
    authTokenEnabled?: pulumi.Input<boolean>;
    /**
     * Specifies whether read-only replicas will be automatically promoted to read/write primary if the existing primary fails.
     * When creating, by default the Global Replication Group inherits the automatic failover setting of the primary replication group.
     */
    automaticFailoverEnabled?: pulumi.Input<boolean>;
    /**
     * The instance class used.
     * See AWS documentation for information on [supported node types](https://docs.aws.amazon.com/AmazonElastiCache/latest/red-ug/CacheNodes.SupportedTypes.html)
     * and [guidance on selecting node types](https://docs.aws.amazon.com/AmazonElastiCache/latest/red-ug/nodes-select-size.html).
     * When creating, by default the Global Replication Group inherits the node type of the primary replication group.
     */
    cacheNodeType?: pulumi.Input<string>;
    /**
     * Indicates whether the Global Datastore is cluster enabled.
     */
    clusterEnabled?: pulumi.Input<boolean>;
    /**
     * The name of the cache engine to be used for the clusters in this global replication group.
     * When creating, by default the Global Replication Group inherits the engine of the primary replication group.
     * If an engine is specified, the Global Replication Group and all member replication groups will be upgraded to this engine.
     * Valid values are `redis` or `valkey`.
     * Default is `redis` if `engineVersion` is specified.
     */
    engine?: pulumi.Input<string>;
    /**
     * Engine version to use for the Global Replication Group.
     * When creating, by default the Global Replication Group inherits the version of the primary replication group.
     * If a version is specified, the Global Replication Group and all member replication groups will be upgraded to this version.
     * Cannot be downgraded without replacing the Global Replication Group and all member replication groups.
     * When the version is 7 or higher, the major and minor version should be set, e.g., `7.2`.
     * When the version is 6, the major and minor version can be set, e.g., `6.2`,
     * or the minor version can be unspecified which will use the latest version at creation time, e.g., `6.x`.
     * The actual engine version used is returned in the attribute `engineVersionActual`, see Attribute Reference below.
     */
    engineVersion?: pulumi.Input<string>;
    /**
     * The full version number of the cache engine running on the members of this global replication group.
     */
    engineVersionActual?: pulumi.Input<string>;
    /**
     * Set of node groups (shards) on the global replication group.
     * Has the values:
     */
    globalNodeGroups?: pulumi.Input<pulumi.Input<inputs.elasticache.GlobalReplicationGroupGlobalNodeGroup>[]>;
    /**
     * A user-created description for the global replication group.
     */
    globalReplicationGroupDescription?: pulumi.Input<string>;
    /**
     * The full ID of the global replication group.
     */
    globalReplicationGroupId?: pulumi.Input<string>;
    /**
     * The suffix name of a Global Datastore. If `globalReplicationGroupIdSuffix` is changed, creates a new resource.
     */
    globalReplicationGroupIdSuffix?: pulumi.Input<string>;
    /**
     * The number of node groups (shards) on the global replication group.
     */
    numNodeGroups?: pulumi.Input<number>;
    /**
     * An ElastiCache Parameter Group to use for the Global Replication Group.
     * Required when upgrading an engine or major engine version, but will be ignored if left configured after the upgrade is complete.
     * Specifying without a major version upgrade will fail.
     * Note that ElastiCache creates a copy of this parameter group for each member replication group.
     */
    parameterGroupName?: pulumi.Input<string>;
    /**
     * The ID of the primary cluster that accepts writes and will replicate updates to the secondary cluster. If `primaryReplicationGroupId` is changed, creates a new resource.
     */
    primaryReplicationGroupId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A flag that indicates whether the encryption in transit is enabled.
     */
    transitEncryptionEnabled?: pulumi.Input<boolean>;
}
/**
 * The set of arguments for constructing a GlobalReplicationGroup resource.
 */
export interface GlobalReplicationGroupArgs {
    /**
     * Specifies whether read-only replicas will be automatically promoted to read/write primary if the existing primary fails.
     * When creating, by default the Global Replication Group inherits the automatic failover setting of the primary replication group.
     */
    automaticFailoverEnabled?: pulumi.Input<boolean>;
    /**
     * The instance class used.
     * See AWS documentation for information on [supported node types](https://docs.aws.amazon.com/AmazonElastiCache/latest/red-ug/CacheNodes.SupportedTypes.html)
     * and [guidance on selecting node types](https://docs.aws.amazon.com/AmazonElastiCache/latest/red-ug/nodes-select-size.html).
     * When creating, by default the Global Replication Group inherits the node type of the primary replication group.
     */
    cacheNodeType?: pulumi.Input<string>;
    /**
     * The name of the cache engine to be used for the clusters in this global replication group.
     * When creating, by default the Global Replication Group inherits the engine of the primary replication group.
     * If an engine is specified, the Global Replication Group and all member replication groups will be upgraded to this engine.
     * Valid values are `redis` or `valkey`.
     * Default is `redis` if `engineVersion` is specified.
     */
    engine?: pulumi.Input<string>;
    /**
     * Engine version to use for the Global Replication Group.
     * When creating, by default the Global Replication Group inherits the version of the primary replication group.
     * If a version is specified, the Global Replication Group and all member replication groups will be upgraded to this version.
     * Cannot be downgraded without replacing the Global Replication Group and all member replication groups.
     * When the version is 7 or higher, the major and minor version should be set, e.g., `7.2`.
     * When the version is 6, the major and minor version can be set, e.g., `6.2`,
     * or the minor version can be unspecified which will use the latest version at creation time, e.g., `6.x`.
     * The actual engine version used is returned in the attribute `engineVersionActual`, see Attribute Reference below.
     */
    engineVersion?: pulumi.Input<string>;
    /**
     * A user-created description for the global replication group.
     */
    globalReplicationGroupDescription?: pulumi.Input<string>;
    /**
     * The suffix name of a Global Datastore. If `globalReplicationGroupIdSuffix` is changed, creates a new resource.
     */
    globalReplicationGroupIdSuffix: pulumi.Input<string>;
    /**
     * The number of node groups (shards) on the global replication group.
     */
    numNodeGroups?: pulumi.Input<number>;
    /**
     * An ElastiCache Parameter Group to use for the Global Replication Group.
     * Required when upgrading an engine or major engine version, but will be ignored if left configured after the upgrade is complete.
     * Specifying without a major version upgrade will fail.
     * Note that ElastiCache creates a copy of this parameter group for each member replication group.
     */
    parameterGroupName?: pulumi.Input<string>;
    /**
     * The ID of the primary cluster that accepts writes and will replicate updates to the secondary cluster. If `primaryReplicationGroupId` is changed, creates a new resource.
     */
    primaryReplicationGroupId: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
