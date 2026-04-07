import * as pulumi from "@pulumi/pulumi";
/**
 * Provides an DocumentDB Cluster Resource Instance. A Cluster Instance Resource defines
 * attributes that are specific to a single instance in a [DocumentDB Cluster][1].
 *
 * You do not designate a primary and subsequent replicas. Instead, you simply add DocumentDB
 * Instances and DocumentDB manages the replication. You can use the [count][3]
 * meta-parameter to make multiple instances and join them all to the same DocumentDB
 * Cluster, or you may specify different Cluster Instance resources with various
 * `instanceClass` sizes.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const _default = new aws.docdb.Cluster("default", {
 *     clusterIdentifier: "docdb-cluster-demo",
 *     availabilityZones: [
 *         "us-west-2a",
 *         "us-west-2b",
 *         "us-west-2c",
 *     ],
 *     masterUsername: "foo",
 *     masterPassword: "barbut8chars",
 * });
 * const clusterInstances: aws.docdb.ClusterInstance[] = [];
 * for (const range = {value: 0}; range.value < 2; range.value++) {
 *     clusterInstances.push(new aws.docdb.ClusterInstance(`cluster_instances-${range.value}`, {
 *         identifier: `docdb-cluster-demo-${range.value}`,
 *         clusterIdentifier: _default.id,
 *         instanceClass: "db.r5.large",
 *     }));
 * }
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import DocumentDB Cluster Instances using the `identifier`. For example:
 *
 * ```sh
 * $ pulumi import aws:docdb/clusterInstance:ClusterInstance prod_instance_1 aurora-cluster-instance-1
 * ```
 */
export declare class ClusterInstance extends pulumi.CustomResource {
    /**
     * Get an existing ClusterInstance resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ClusterInstanceState, opts?: pulumi.CustomResourceOptions): ClusterInstance;
    /**
     * Returns true if the given object is an instance of ClusterInstance.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ClusterInstance;
    /**
     * Specifies whether any database modifications
     * are applied immediately, or during the next maintenance window. Default is`false`.
     */
    readonly applyImmediately: pulumi.Output<boolean | undefined>;
    /**
     * Amazon Resource Name (ARN) of cluster instance
     */
    readonly arn: pulumi.Output<string>;
    /**
     * This parameter does not apply to Amazon DocumentDB. Amazon DocumentDB does not perform minor version upgrades regardless of the value set (see [docs](https://docs.aws.amazon.com/documentdb/latest/developerguide/API_DBInstance.html)). Default `true`.
     */
    readonly autoMinorVersionUpgrade: pulumi.Output<boolean | undefined>;
    /**
     * The EC2 Availability Zone that the DB instance is created in. See [docs](https://docs.aws.amazon.com/documentdb/latest/developerguide/API_CreateDBInstance.html) about the details.
     */
    readonly availabilityZone: pulumi.Output<string>;
    /**
     * The identifier of the certificate authority (CA) certificate for the DB instance.
     */
    readonly caCertIdentifier: pulumi.Output<string>;
    /**
     * The identifier of the `aws.docdb.Cluster` in which to launch this instance.
     */
    readonly clusterIdentifier: pulumi.Output<string>;
    /**
     * Copy all DB instance `tags` to snapshots. Default is `false`.
     */
    readonly copyTagsToSnapshot: pulumi.Output<boolean | undefined>;
    /**
     * The DB subnet group to associate with this DB instance.
     */
    readonly dbSubnetGroupName: pulumi.Output<string>;
    /**
     * The region-unique, immutable identifier for the DB instance.
     */
    readonly dbiResourceId: pulumi.Output<string>;
    /**
     * A value that indicates whether to enable Performance Insights for the DB Instance. Default `false`. See [docs] (https://docs.aws.amazon.com/documentdb/latest/developerguide/performance-insights.html) about the details.
     */
    readonly enablePerformanceInsights: pulumi.Output<boolean | undefined>;
    /**
     * The DNS address for this instance. May not be writable
     */
    readonly endpoint: pulumi.Output<string>;
    /**
     * The name of the database engine to be used for the DocumentDB instance. Defaults to `docdb`. Valid Values: `docdb`.
     */
    readonly engine: pulumi.Output<string | undefined>;
    /**
     * The database engine version
     */
    readonly engineVersion: pulumi.Output<string>;
    /**
     * The identifier for the DocumentDB instance, if omitted, the provider will assign a random, unique identifier.
     */
    readonly identifier: pulumi.Output<string>;
    /**
     * Creates a unique identifier beginning with the specified prefix. Conflicts with `identifier`.
     */
    readonly identifierPrefix: pulumi.Output<string>;
    /**
     * The instance class to use. For details on CPU and memory, see [Scaling for DocumentDB Instances](https://docs.aws.amazon.com/documentdb/latest/developerguide/db-cluster-manage-performance.html#db-cluster-manage-scaling-instance).
     * DocumentDB currently supports the below instance classes.
     * Please see [AWS Documentation](https://docs.aws.amazon.com/documentdb/latest/developerguide/db-instance-classes.html#db-instance-class-specs) for complete details.
     * - db.r6g.large
     * - db.r6g.xlarge
     * - db.r6g.2xlarge
     * - db.r6g.4xlarge
     * - db.r6g.8xlarge
     * - db.r6g.12xlarge
     * - db.r6g.16xlarge
     * - db.r5.large
     * - db.r5.xlarge
     * - db.r5.2xlarge
     * - db.r5.4xlarge
     * - db.r5.12xlarge
     * - db.r5.24xlarge
     * - db.r4.large
     * - db.r4.xlarge
     * - db.r4.2xlarge
     * - db.r4.4xlarge
     * - db.r4.8xlarge
     * - db.r4.16xlarge
     * - db.t4g.medium
     * - db.t3.medium
     */
    readonly instanceClass: pulumi.Output<string>;
    /**
     * The ARN for the KMS encryption key if one is set to the cluster.
     */
    readonly kmsKeyId: pulumi.Output<string>;
    /**
     * The KMS key identifier is the key ARN, key ID, alias ARN, or alias name for the KMS key. If you do not specify a value for PerformanceInsightsKMSKeyId, then Amazon DocumentDB uses your default KMS key.
     */
    readonly performanceInsightsKmsKeyId: pulumi.Output<string>;
    /**
     * The database port
     */
    readonly port: pulumi.Output<number>;
    /**
     * The daily time range during which automated backups are created if automated backups are enabled.
     */
    readonly preferredBackupWindow: pulumi.Output<string>;
    /**
     * The window to perform maintenance in.
     * Syntax: "ddd:hh24:mi-ddd:hh24:mi". Eg: "Mon:00:00-Mon:03:00".
     */
    readonly preferredMaintenanceWindow: pulumi.Output<string>;
    /**
     * Default 0. Failover Priority setting on instance level. The reader who has lower tier has higher priority to get promoter to writer.
     */
    readonly promotionTier: pulumi.Output<number | undefined>;
    readonly publiclyAccessible: pulumi.Output<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Specifies whether the DB cluster is encrypted.
     */
    readonly storageEncrypted: pulumi.Output<boolean>;
    /**
     * A map of tags to assign to the instance. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Boolean indicating if this instance is writable. `False` indicates this instance is a read replica.
     */
    readonly writer: pulumi.Output<boolean>;
    /**
     * Create a ClusterInstance resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ClusterInstanceArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ClusterInstance resources.
 */
export interface ClusterInstanceState {
    /**
     * Specifies whether any database modifications
     * are applied immediately, or during the next maintenance window. Default is`false`.
     */
    applyImmediately?: pulumi.Input<boolean>;
    /**
     * Amazon Resource Name (ARN) of cluster instance
     */
    arn?: pulumi.Input<string>;
    /**
     * This parameter does not apply to Amazon DocumentDB. Amazon DocumentDB does not perform minor version upgrades regardless of the value set (see [docs](https://docs.aws.amazon.com/documentdb/latest/developerguide/API_DBInstance.html)). Default `true`.
     */
    autoMinorVersionUpgrade?: pulumi.Input<boolean>;
    /**
     * The EC2 Availability Zone that the DB instance is created in. See [docs](https://docs.aws.amazon.com/documentdb/latest/developerguide/API_CreateDBInstance.html) about the details.
     */
    availabilityZone?: pulumi.Input<string>;
    /**
     * The identifier of the certificate authority (CA) certificate for the DB instance.
     */
    caCertIdentifier?: pulumi.Input<string>;
    /**
     * The identifier of the `aws.docdb.Cluster` in which to launch this instance.
     */
    clusterIdentifier?: pulumi.Input<string>;
    /**
     * Copy all DB instance `tags` to snapshots. Default is `false`.
     */
    copyTagsToSnapshot?: pulumi.Input<boolean>;
    /**
     * The DB subnet group to associate with this DB instance.
     */
    dbSubnetGroupName?: pulumi.Input<string>;
    /**
     * The region-unique, immutable identifier for the DB instance.
     */
    dbiResourceId?: pulumi.Input<string>;
    /**
     * A value that indicates whether to enable Performance Insights for the DB Instance. Default `false`. See [docs] (https://docs.aws.amazon.com/documentdb/latest/developerguide/performance-insights.html) about the details.
     */
    enablePerformanceInsights?: pulumi.Input<boolean>;
    /**
     * The DNS address for this instance. May not be writable
     */
    endpoint?: pulumi.Input<string>;
    /**
     * The name of the database engine to be used for the DocumentDB instance. Defaults to `docdb`. Valid Values: `docdb`.
     */
    engine?: pulumi.Input<string>;
    /**
     * The database engine version
     */
    engineVersion?: pulumi.Input<string>;
    /**
     * The identifier for the DocumentDB instance, if omitted, the provider will assign a random, unique identifier.
     */
    identifier?: pulumi.Input<string>;
    /**
     * Creates a unique identifier beginning with the specified prefix. Conflicts with `identifier`.
     */
    identifierPrefix?: pulumi.Input<string>;
    /**
     * The instance class to use. For details on CPU and memory, see [Scaling for DocumentDB Instances](https://docs.aws.amazon.com/documentdb/latest/developerguide/db-cluster-manage-performance.html#db-cluster-manage-scaling-instance).
     * DocumentDB currently supports the below instance classes.
     * Please see [AWS Documentation](https://docs.aws.amazon.com/documentdb/latest/developerguide/db-instance-classes.html#db-instance-class-specs) for complete details.
     * - db.r6g.large
     * - db.r6g.xlarge
     * - db.r6g.2xlarge
     * - db.r6g.4xlarge
     * - db.r6g.8xlarge
     * - db.r6g.12xlarge
     * - db.r6g.16xlarge
     * - db.r5.large
     * - db.r5.xlarge
     * - db.r5.2xlarge
     * - db.r5.4xlarge
     * - db.r5.12xlarge
     * - db.r5.24xlarge
     * - db.r4.large
     * - db.r4.xlarge
     * - db.r4.2xlarge
     * - db.r4.4xlarge
     * - db.r4.8xlarge
     * - db.r4.16xlarge
     * - db.t4g.medium
     * - db.t3.medium
     */
    instanceClass?: pulumi.Input<string>;
    /**
     * The ARN for the KMS encryption key if one is set to the cluster.
     */
    kmsKeyId?: pulumi.Input<string>;
    /**
     * The KMS key identifier is the key ARN, key ID, alias ARN, or alias name for the KMS key. If you do not specify a value for PerformanceInsightsKMSKeyId, then Amazon DocumentDB uses your default KMS key.
     */
    performanceInsightsKmsKeyId?: pulumi.Input<string>;
    /**
     * The database port
     */
    port?: pulumi.Input<number>;
    /**
     * The daily time range during which automated backups are created if automated backups are enabled.
     */
    preferredBackupWindow?: pulumi.Input<string>;
    /**
     * The window to perform maintenance in.
     * Syntax: "ddd:hh24:mi-ddd:hh24:mi". Eg: "Mon:00:00-Mon:03:00".
     */
    preferredMaintenanceWindow?: pulumi.Input<string>;
    /**
     * Default 0. Failover Priority setting on instance level. The reader who has lower tier has higher priority to get promoter to writer.
     */
    promotionTier?: pulumi.Input<number>;
    publiclyAccessible?: pulumi.Input<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Specifies whether the DB cluster is encrypted.
     */
    storageEncrypted?: pulumi.Input<boolean>;
    /**
     * A map of tags to assign to the instance. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Boolean indicating if this instance is writable. `False` indicates this instance is a read replica.
     */
    writer?: pulumi.Input<boolean>;
}
/**
 * The set of arguments for constructing a ClusterInstance resource.
 */
export interface ClusterInstanceArgs {
    /**
     * Specifies whether any database modifications
     * are applied immediately, or during the next maintenance window. Default is`false`.
     */
    applyImmediately?: pulumi.Input<boolean>;
    /**
     * This parameter does not apply to Amazon DocumentDB. Amazon DocumentDB does not perform minor version upgrades regardless of the value set (see [docs](https://docs.aws.amazon.com/documentdb/latest/developerguide/API_DBInstance.html)). Default `true`.
     */
    autoMinorVersionUpgrade?: pulumi.Input<boolean>;
    /**
     * The EC2 Availability Zone that the DB instance is created in. See [docs](https://docs.aws.amazon.com/documentdb/latest/developerguide/API_CreateDBInstance.html) about the details.
     */
    availabilityZone?: pulumi.Input<string>;
    /**
     * The identifier of the certificate authority (CA) certificate for the DB instance.
     */
    caCertIdentifier?: pulumi.Input<string>;
    /**
     * The identifier of the `aws.docdb.Cluster` in which to launch this instance.
     */
    clusterIdentifier: pulumi.Input<string>;
    /**
     * Copy all DB instance `tags` to snapshots. Default is `false`.
     */
    copyTagsToSnapshot?: pulumi.Input<boolean>;
    /**
     * A value that indicates whether to enable Performance Insights for the DB Instance. Default `false`. See [docs] (https://docs.aws.amazon.com/documentdb/latest/developerguide/performance-insights.html) about the details.
     */
    enablePerformanceInsights?: pulumi.Input<boolean>;
    /**
     * The name of the database engine to be used for the DocumentDB instance. Defaults to `docdb`. Valid Values: `docdb`.
     */
    engine?: pulumi.Input<string>;
    /**
     * The identifier for the DocumentDB instance, if omitted, the provider will assign a random, unique identifier.
     */
    identifier?: pulumi.Input<string>;
    /**
     * Creates a unique identifier beginning with the specified prefix. Conflicts with `identifier`.
     */
    identifierPrefix?: pulumi.Input<string>;
    /**
     * The instance class to use. For details on CPU and memory, see [Scaling for DocumentDB Instances](https://docs.aws.amazon.com/documentdb/latest/developerguide/db-cluster-manage-performance.html#db-cluster-manage-scaling-instance).
     * DocumentDB currently supports the below instance classes.
     * Please see [AWS Documentation](https://docs.aws.amazon.com/documentdb/latest/developerguide/db-instance-classes.html#db-instance-class-specs) for complete details.
     * - db.r6g.large
     * - db.r6g.xlarge
     * - db.r6g.2xlarge
     * - db.r6g.4xlarge
     * - db.r6g.8xlarge
     * - db.r6g.12xlarge
     * - db.r6g.16xlarge
     * - db.r5.large
     * - db.r5.xlarge
     * - db.r5.2xlarge
     * - db.r5.4xlarge
     * - db.r5.12xlarge
     * - db.r5.24xlarge
     * - db.r4.large
     * - db.r4.xlarge
     * - db.r4.2xlarge
     * - db.r4.4xlarge
     * - db.r4.8xlarge
     * - db.r4.16xlarge
     * - db.t4g.medium
     * - db.t3.medium
     */
    instanceClass: pulumi.Input<string>;
    /**
     * The KMS key identifier is the key ARN, key ID, alias ARN, or alias name for the KMS key. If you do not specify a value for PerformanceInsightsKMSKeyId, then Amazon DocumentDB uses your default KMS key.
     */
    performanceInsightsKmsKeyId?: pulumi.Input<string>;
    /**
     * The window to perform maintenance in.
     * Syntax: "ddd:hh24:mi-ddd:hh24:mi". Eg: "Mon:00:00-Mon:03:00".
     */
    preferredMaintenanceWindow?: pulumi.Input<string>;
    /**
     * Default 0. Failover Priority setting on instance level. The reader who has lower tier has higher priority to get promoter to writer.
     */
    promotionTier?: pulumi.Input<number>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the instance. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
