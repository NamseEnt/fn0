import * as pulumi from "@pulumi/pulumi";
import * as enums from "../types/enums";
/**
 * Provides an RDS Cluster Instance Resource. A Cluster Instance Resource defines
 * attributes that are specific to a single instance in a RDS Cluster,
 * specifically running Amazon Aurora.
 *
 * Unlike other RDS resources that support replication, with Amazon Aurora you do
 * not designate a primary and subsequent replicas. Instead, you simply add RDS
 * Instances and Aurora manages the replication. You can use the [count][5]
 * meta-parameter to make multiple instances and join them all to the same RDS
 * Cluster, or you may specify different Cluster Instance resources with various
 * `instanceClass` sizes.
 *
 * For more information on Amazon Aurora, see [Aurora on Amazon RDS](https://docs.aws.amazon.com/AmazonRDS/latest/UserGuide/CHAP_Aurora.html) in the Amazon RDS User Guide.
 *
 * > **NOTE:** Deletion Protection from the RDS service can only be enabled at the cluster level, not for individual cluster instances. You can still add the [`protect` CustomResourceOption](https://www.pulumi.com/docs/intro/concepts/programming-model/#protect) to this resource configuration if you desire protection from accidental deletion.
 *
 * > **NOTE:** `aurora` is no longer a valid `engine` because of [Amazon Aurora's MySQL-Compatible Edition version 1 end of life](https://docs.aws.amazon.com/AmazonRDS/latest/AuroraUserGuide/Aurora.MySQL56.EOL.html).
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const _default = new aws.rds.Cluster("default", {
 *     clusterIdentifier: "aurora-cluster-demo",
 *     availabilityZones: [
 *         "us-west-2a",
 *         "us-west-2b",
 *         "us-west-2c",
 *     ],
 *     databaseName: "mydb",
 *     masterUsername: "foo",
 *     masterPassword: "barbut8chars",
 * });
 * const clusterInstances: aws.rds.ClusterInstance[] = [];
 * for (const range = {value: 0}; range.value < 2; range.value++) {
 *     clusterInstances.push(new aws.rds.ClusterInstance(`cluster_instances-${range.value}`, {
 *         identifier: `aurora-cluster-demo-${range.value}`,
 *         clusterIdentifier: _default.id,
 *         instanceClass: aws.rds.InstanceType.R4_Large,
 *         engine: _default.engine.apply((x) => aws.rds.EngineType[x]),
 *         engineVersion: _default.engineVersion,
 *     }));
 * }
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import RDS Cluster Instances using the `identifier`. For example:
 *
 * ```sh
 * $ pulumi import aws:rds/clusterInstance:ClusterInstance prod_instance_1 aurora-cluster-instance-1
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
     * Specifies whether any database modifications are applied immediately, or during the next maintenance window. Default is`false`.
     */
    readonly applyImmediately: pulumi.Output<boolean>;
    /**
     * Amazon Resource Name (ARN) of cluster instance
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Indicates that minor engine upgrades will be applied automatically to the DB instance during the maintenance window. Default `true`.
     */
    readonly autoMinorVersionUpgrade: pulumi.Output<boolean | undefined>;
    /**
     * EC2 Availability Zone that the DB instance is created in. See [docs](https://docs.aws.amazon.com/AmazonRDS/latest/APIReference/API_CreateDBInstance.html) about the details.
     */
    readonly availabilityZone: pulumi.Output<string>;
    /**
     * Identifier of the CA certificate for the DB instance.
     */
    readonly caCertIdentifier: pulumi.Output<string>;
    /**
     * Identifier of the `aws.rds.Cluster` in which to launch this instance.
     */
    readonly clusterIdentifier: pulumi.Output<string>;
    /**
     * Indicates whether to copy all of the user-defined tags from the DB instance to snapshots of the DB instance. Default `false`.
     */
    readonly copyTagsToSnapshot: pulumi.Output<boolean | undefined>;
    /**
     * Instance profile associated with the underlying Amazon EC2 instance of an RDS Custom DB instance.
     */
    readonly customIamInstanceProfile: pulumi.Output<string | undefined>;
    /**
     * Name of the DB parameter group to associate with this instance.
     */
    readonly dbParameterGroupName: pulumi.Output<string>;
    /**
     * Specifies the DB subnet group to associate with this DB instance. The default behavior varies depending on whether `dbSubnetGroupName` is specified. Please refer to official [AWS documentation](https://docs.aws.amazon.com/cli/latest/reference/rds/create-db-instance.html) to understand how `dbSubnetGroupName` and `publiclyAccessible` parameters affect DB instance behaviour. **NOTE:** This must match the `dbSubnetGroupName` of the attached `aws.rds.Cluster`.
     */
    readonly dbSubnetGroupName: pulumi.Output<string>;
    /**
     * Region-unique, immutable identifier for the DB instance.
     */
    readonly dbiResourceId: pulumi.Output<string>;
    /**
     * DNS address for this instance. May not be writable
     */
    readonly endpoint: pulumi.Output<string>;
    /**
     * Name of the database engine to be used for the RDS cluster instance.
     * Valid Values: `aurora-mysql`, `aurora-postgresql`, `mysql`, `postgres`.(Note that `mysql` and `postgres` are Multi-AZ RDS clusters).
     */
    readonly engine: pulumi.Output<enums.rds.EngineType>;
    /**
     * Database engine version. Please note that to upgrade the `engineVersion` of the instance, it must be done on the `aws.rds.Cluster` `engineVersion`. Trying to upgrade in `awsClusterInstance` will not update the `engineVersion`.
     */
    readonly engineVersion: pulumi.Output<string>;
    /**
     * Database engine version
     */
    readonly engineVersionActual: pulumi.Output<string>;
    /**
     * Forces an instance to be destroyed when a part of a read replica cluster. **Note:** will promote the read replica to a standalone cluster before instance deletion.
     */
    readonly forceDestroy: pulumi.Output<boolean | undefined>;
    /**
     * Identifier for the RDS instance, if omitted, Pulumi will assign a random, unique identifier.
     */
    readonly identifier: pulumi.Output<string>;
    /**
     * Creates a unique identifier beginning with the specified prefix. Conflicts with `identifier`.
     */
    readonly identifierPrefix: pulumi.Output<string>;
    /**
     * Instance class to use. For details on CPU and memory, see [Scaling Aurora DB Instances](https://docs.aws.amazon.com/AmazonRDS/latest/UserGuide/Aurora.Managing.html). Aurora uses `db.*` instance classes/types. Please see [AWS Documentation](https://docs.aws.amazon.com/AmazonRDS/latest/UserGuide/Concepts.DBInstanceClass.html) for currently available instance classes and complete details. For Aurora Serverless v2 use `db.serverless`.
     */
    readonly instanceClass: pulumi.Output<string>;
    /**
     * ARN for the KMS encryption key if one is set to the cluster.
     */
    readonly kmsKeyId: pulumi.Output<string>;
    /**
     * Interval, in seconds, between points when Enhanced Monitoring metrics are collected for the DB instance. To disable collecting Enhanced Monitoring metrics, specify 0. The default is 0. Valid Values: 0, 1, 5, 10, 15, 30, 60.
     */
    readonly monitoringInterval: pulumi.Output<number | undefined>;
    /**
     * ARN for the IAM role that permits RDS to send enhanced monitoring metrics to CloudWatch Logs. You can find more information on the [AWS Documentation](http://docs.aws.amazon.com/AmazonRDS/latest/UserGuide/USER_Monitoring.html) what IAM permissions are needed to allow Enhanced Monitoring for RDS Instances.
     */
    readonly monitoringRoleArn: pulumi.Output<string>;
    /**
     * Network type of the DB instance.
     */
    readonly networkType: pulumi.Output<string>;
    /**
     * Specifies whether Performance Insights is enabled or not. **NOTE:** When Performance Insights is configured at the cluster level through `aws.rds.Cluster`, this argument cannot be set to a value that conflicts with the cluster's configuration.
     */
    readonly performanceInsightsEnabled: pulumi.Output<boolean>;
    /**
     * ARN for the KMS key to encrypt Performance Insights data. When specifying `performanceInsightsKmsKeyId`, `performanceInsightsEnabled` needs to be set to true.
     */
    readonly performanceInsightsKmsKeyId: pulumi.Output<string>;
    /**
     * Amount of time in days to retain Performance Insights data. Valid values are `7`, `731` (2 years) or a multiple of `31`. When specifying `performanceInsightsRetentionPeriod`, `performanceInsightsEnabled` needs to be set to true. Defaults to '7'.
     */
    readonly performanceInsightsRetentionPeriod: pulumi.Output<number>;
    /**
     * Database port
     */
    readonly port: pulumi.Output<number>;
    /**
     * Daily time range during which automated backups are created if automated backups are enabled. Eg: "04:00-09:00". **NOTE:** If `preferredBackupWindow` is set at the cluster level, this argument **must** be omitted.
     */
    readonly preferredBackupWindow: pulumi.Output<string>;
    /**
     * Window to perform maintenance in. Syntax: "ddd:hh24:mi-ddd:hh24:mi". Eg: "Mon:00:00-Mon:03:00".
     */
    readonly preferredMaintenanceWindow: pulumi.Output<string>;
    /**
     * Default 0. Failover Priority setting on instance level. The reader who has lower tier has higher priority to get promoted to writer.
     */
    readonly promotionTier: pulumi.Output<number | undefined>;
    /**
     * Bool to control if instance is publicly accessible. Default `false`. See the documentation on [Creating DB Instances](https://docs.aws.amazon.com/AmazonRDS/latest/APIReference/API_CreateDBInstance.html) for more details on controlling this property.
     */
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
     * Map of tags to assign to the instance. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     *
     * For more detailed documentation about each argument, refer to
     * the [AWS official documentation](https://docs.aws.amazon.com/cli/latest/reference/rds/create-db-instance.html).
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * Map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
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
     * Specifies whether any database modifications are applied immediately, or during the next maintenance window. Default is`false`.
     */
    applyImmediately?: pulumi.Input<boolean>;
    /**
     * Amazon Resource Name (ARN) of cluster instance
     */
    arn?: pulumi.Input<string>;
    /**
     * Indicates that minor engine upgrades will be applied automatically to the DB instance during the maintenance window. Default `true`.
     */
    autoMinorVersionUpgrade?: pulumi.Input<boolean>;
    /**
     * EC2 Availability Zone that the DB instance is created in. See [docs](https://docs.aws.amazon.com/AmazonRDS/latest/APIReference/API_CreateDBInstance.html) about the details.
     */
    availabilityZone?: pulumi.Input<string>;
    /**
     * Identifier of the CA certificate for the DB instance.
     */
    caCertIdentifier?: pulumi.Input<string>;
    /**
     * Identifier of the `aws.rds.Cluster` in which to launch this instance.
     */
    clusterIdentifier?: pulumi.Input<string>;
    /**
     * Indicates whether to copy all of the user-defined tags from the DB instance to snapshots of the DB instance. Default `false`.
     */
    copyTagsToSnapshot?: pulumi.Input<boolean>;
    /**
     * Instance profile associated with the underlying Amazon EC2 instance of an RDS Custom DB instance.
     */
    customIamInstanceProfile?: pulumi.Input<string>;
    /**
     * Name of the DB parameter group to associate with this instance.
     */
    dbParameterGroupName?: pulumi.Input<string>;
    /**
     * Specifies the DB subnet group to associate with this DB instance. The default behavior varies depending on whether `dbSubnetGroupName` is specified. Please refer to official [AWS documentation](https://docs.aws.amazon.com/cli/latest/reference/rds/create-db-instance.html) to understand how `dbSubnetGroupName` and `publiclyAccessible` parameters affect DB instance behaviour. **NOTE:** This must match the `dbSubnetGroupName` of the attached `aws.rds.Cluster`.
     */
    dbSubnetGroupName?: pulumi.Input<string>;
    /**
     * Region-unique, immutable identifier for the DB instance.
     */
    dbiResourceId?: pulumi.Input<string>;
    /**
     * DNS address for this instance. May not be writable
     */
    endpoint?: pulumi.Input<string>;
    /**
     * Name of the database engine to be used for the RDS cluster instance.
     * Valid Values: `aurora-mysql`, `aurora-postgresql`, `mysql`, `postgres`.(Note that `mysql` and `postgres` are Multi-AZ RDS clusters).
     */
    engine?: pulumi.Input<enums.rds.EngineType>;
    /**
     * Database engine version. Please note that to upgrade the `engineVersion` of the instance, it must be done on the `aws.rds.Cluster` `engineVersion`. Trying to upgrade in `awsClusterInstance` will not update the `engineVersion`.
     */
    engineVersion?: pulumi.Input<string>;
    /**
     * Database engine version
     */
    engineVersionActual?: pulumi.Input<string>;
    /**
     * Forces an instance to be destroyed when a part of a read replica cluster. **Note:** will promote the read replica to a standalone cluster before instance deletion.
     */
    forceDestroy?: pulumi.Input<boolean>;
    /**
     * Identifier for the RDS instance, if omitted, Pulumi will assign a random, unique identifier.
     */
    identifier?: pulumi.Input<string>;
    /**
     * Creates a unique identifier beginning with the specified prefix. Conflicts with `identifier`.
     */
    identifierPrefix?: pulumi.Input<string>;
    /**
     * Instance class to use. For details on CPU and memory, see [Scaling Aurora DB Instances](https://docs.aws.amazon.com/AmazonRDS/latest/UserGuide/Aurora.Managing.html). Aurora uses `db.*` instance classes/types. Please see [AWS Documentation](https://docs.aws.amazon.com/AmazonRDS/latest/UserGuide/Concepts.DBInstanceClass.html) for currently available instance classes and complete details. For Aurora Serverless v2 use `db.serverless`.
     */
    instanceClass?: pulumi.Input<string | enums.rds.InstanceType>;
    /**
     * ARN for the KMS encryption key if one is set to the cluster.
     */
    kmsKeyId?: pulumi.Input<string>;
    /**
     * Interval, in seconds, between points when Enhanced Monitoring metrics are collected for the DB instance. To disable collecting Enhanced Monitoring metrics, specify 0. The default is 0. Valid Values: 0, 1, 5, 10, 15, 30, 60.
     */
    monitoringInterval?: pulumi.Input<number>;
    /**
     * ARN for the IAM role that permits RDS to send enhanced monitoring metrics to CloudWatch Logs. You can find more information on the [AWS Documentation](http://docs.aws.amazon.com/AmazonRDS/latest/UserGuide/USER_Monitoring.html) what IAM permissions are needed to allow Enhanced Monitoring for RDS Instances.
     */
    monitoringRoleArn?: pulumi.Input<string>;
    /**
     * Network type of the DB instance.
     */
    networkType?: pulumi.Input<string>;
    /**
     * Specifies whether Performance Insights is enabled or not. **NOTE:** When Performance Insights is configured at the cluster level through `aws.rds.Cluster`, this argument cannot be set to a value that conflicts with the cluster's configuration.
     */
    performanceInsightsEnabled?: pulumi.Input<boolean>;
    /**
     * ARN for the KMS key to encrypt Performance Insights data. When specifying `performanceInsightsKmsKeyId`, `performanceInsightsEnabled` needs to be set to true.
     */
    performanceInsightsKmsKeyId?: pulumi.Input<string>;
    /**
     * Amount of time in days to retain Performance Insights data. Valid values are `7`, `731` (2 years) or a multiple of `31`. When specifying `performanceInsightsRetentionPeriod`, `performanceInsightsEnabled` needs to be set to true. Defaults to '7'.
     */
    performanceInsightsRetentionPeriod?: pulumi.Input<number>;
    /**
     * Database port
     */
    port?: pulumi.Input<number>;
    /**
     * Daily time range during which automated backups are created if automated backups are enabled. Eg: "04:00-09:00". **NOTE:** If `preferredBackupWindow` is set at the cluster level, this argument **must** be omitted.
     */
    preferredBackupWindow?: pulumi.Input<string>;
    /**
     * Window to perform maintenance in. Syntax: "ddd:hh24:mi-ddd:hh24:mi". Eg: "Mon:00:00-Mon:03:00".
     */
    preferredMaintenanceWindow?: pulumi.Input<string>;
    /**
     * Default 0. Failover Priority setting on instance level. The reader who has lower tier has higher priority to get promoted to writer.
     */
    promotionTier?: pulumi.Input<number>;
    /**
     * Bool to control if instance is publicly accessible. Default `false`. See the documentation on [Creating DB Instances](https://docs.aws.amazon.com/AmazonRDS/latest/APIReference/API_CreateDBInstance.html) for more details on controlling this property.
     */
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
     * Map of tags to assign to the instance. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     *
     * For more detailed documentation about each argument, refer to
     * the [AWS official documentation](https://docs.aws.amazon.com/cli/latest/reference/rds/create-db-instance.html).
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
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
     * Specifies whether any database modifications are applied immediately, or during the next maintenance window. Default is`false`.
     */
    applyImmediately?: pulumi.Input<boolean>;
    /**
     * Indicates that minor engine upgrades will be applied automatically to the DB instance during the maintenance window. Default `true`.
     */
    autoMinorVersionUpgrade?: pulumi.Input<boolean>;
    /**
     * EC2 Availability Zone that the DB instance is created in. See [docs](https://docs.aws.amazon.com/AmazonRDS/latest/APIReference/API_CreateDBInstance.html) about the details.
     */
    availabilityZone?: pulumi.Input<string>;
    /**
     * Identifier of the CA certificate for the DB instance.
     */
    caCertIdentifier?: pulumi.Input<string>;
    /**
     * Identifier of the `aws.rds.Cluster` in which to launch this instance.
     */
    clusterIdentifier: pulumi.Input<string>;
    /**
     * Indicates whether to copy all of the user-defined tags from the DB instance to snapshots of the DB instance. Default `false`.
     */
    copyTagsToSnapshot?: pulumi.Input<boolean>;
    /**
     * Instance profile associated with the underlying Amazon EC2 instance of an RDS Custom DB instance.
     */
    customIamInstanceProfile?: pulumi.Input<string>;
    /**
     * Name of the DB parameter group to associate with this instance.
     */
    dbParameterGroupName?: pulumi.Input<string>;
    /**
     * Specifies the DB subnet group to associate with this DB instance. The default behavior varies depending on whether `dbSubnetGroupName` is specified. Please refer to official [AWS documentation](https://docs.aws.amazon.com/cli/latest/reference/rds/create-db-instance.html) to understand how `dbSubnetGroupName` and `publiclyAccessible` parameters affect DB instance behaviour. **NOTE:** This must match the `dbSubnetGroupName` of the attached `aws.rds.Cluster`.
     */
    dbSubnetGroupName?: pulumi.Input<string>;
    /**
     * Name of the database engine to be used for the RDS cluster instance.
     * Valid Values: `aurora-mysql`, `aurora-postgresql`, `mysql`, `postgres`.(Note that `mysql` and `postgres` are Multi-AZ RDS clusters).
     */
    engine: pulumi.Input<enums.rds.EngineType>;
    /**
     * Database engine version. Please note that to upgrade the `engineVersion` of the instance, it must be done on the `aws.rds.Cluster` `engineVersion`. Trying to upgrade in `awsClusterInstance` will not update the `engineVersion`.
     */
    engineVersion?: pulumi.Input<string>;
    /**
     * Forces an instance to be destroyed when a part of a read replica cluster. **Note:** will promote the read replica to a standalone cluster before instance deletion.
     */
    forceDestroy?: pulumi.Input<boolean>;
    /**
     * Identifier for the RDS instance, if omitted, Pulumi will assign a random, unique identifier.
     */
    identifier?: pulumi.Input<string>;
    /**
     * Creates a unique identifier beginning with the specified prefix. Conflicts with `identifier`.
     */
    identifierPrefix?: pulumi.Input<string>;
    /**
     * Instance class to use. For details on CPU and memory, see [Scaling Aurora DB Instances](https://docs.aws.amazon.com/AmazonRDS/latest/UserGuide/Aurora.Managing.html). Aurora uses `db.*` instance classes/types. Please see [AWS Documentation](https://docs.aws.amazon.com/AmazonRDS/latest/UserGuide/Concepts.DBInstanceClass.html) for currently available instance classes and complete details. For Aurora Serverless v2 use `db.serverless`.
     */
    instanceClass: pulumi.Input<string | enums.rds.InstanceType>;
    /**
     * Interval, in seconds, between points when Enhanced Monitoring metrics are collected for the DB instance. To disable collecting Enhanced Monitoring metrics, specify 0. The default is 0. Valid Values: 0, 1, 5, 10, 15, 30, 60.
     */
    monitoringInterval?: pulumi.Input<number>;
    /**
     * ARN for the IAM role that permits RDS to send enhanced monitoring metrics to CloudWatch Logs. You can find more information on the [AWS Documentation](http://docs.aws.amazon.com/AmazonRDS/latest/UserGuide/USER_Monitoring.html) what IAM permissions are needed to allow Enhanced Monitoring for RDS Instances.
     */
    monitoringRoleArn?: pulumi.Input<string>;
    /**
     * Specifies whether Performance Insights is enabled or not. **NOTE:** When Performance Insights is configured at the cluster level through `aws.rds.Cluster`, this argument cannot be set to a value that conflicts with the cluster's configuration.
     */
    performanceInsightsEnabled?: pulumi.Input<boolean>;
    /**
     * ARN for the KMS key to encrypt Performance Insights data. When specifying `performanceInsightsKmsKeyId`, `performanceInsightsEnabled` needs to be set to true.
     */
    performanceInsightsKmsKeyId?: pulumi.Input<string>;
    /**
     * Amount of time in days to retain Performance Insights data. Valid values are `7`, `731` (2 years) or a multiple of `31`. When specifying `performanceInsightsRetentionPeriod`, `performanceInsightsEnabled` needs to be set to true. Defaults to '7'.
     */
    performanceInsightsRetentionPeriod?: pulumi.Input<number>;
    /**
     * Daily time range during which automated backups are created if automated backups are enabled. Eg: "04:00-09:00". **NOTE:** If `preferredBackupWindow` is set at the cluster level, this argument **must** be omitted.
     */
    preferredBackupWindow?: pulumi.Input<string>;
    /**
     * Window to perform maintenance in. Syntax: "ddd:hh24:mi-ddd:hh24:mi". Eg: "Mon:00:00-Mon:03:00".
     */
    preferredMaintenanceWindow?: pulumi.Input<string>;
    /**
     * Default 0. Failover Priority setting on instance level. The reader who has lower tier has higher priority to get promoted to writer.
     */
    promotionTier?: pulumi.Input<number>;
    /**
     * Bool to control if instance is publicly accessible. Default `false`. See the documentation on [Creating DB Instances](https://docs.aws.amazon.com/AmazonRDS/latest/APIReference/API_CreateDBInstance.html) for more details on controlling this property.
     */
    publiclyAccessible?: pulumi.Input<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Map of tags to assign to the instance. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     *
     * For more detailed documentation about each argument, refer to
     * the [AWS official documentation](https://docs.aws.amazon.com/cli/latest/reference/rds/create-db-instance.html).
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
