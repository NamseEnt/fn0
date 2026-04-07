import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides an Neptune Cluster Resource. A Cluster Resource defines attributes that are
 * applied to the entire cluster of Neptune Cluster Instances.
 *
 * Changes to a Neptune Cluster can occur when you manually change a
 * parameter, such as `backupRetentionPeriod`, and are reflected in the next maintenance
 * window. Because of this, this provider may report a difference in its planning
 * phase because a modification has not yet taken place. You can use the
 * `applyImmediately` flag to instruct the service to apply the change immediately
 * (see documentation below).
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const _default = new aws.neptune.Cluster("default", {
 *     clusterIdentifier: "neptune-cluster-demo",
 *     engine: "neptune",
 *     backupRetentionPeriod: 5,
 *     preferredBackupWindow: "07:00-09:00",
 *     skipFinalSnapshot: true,
 *     iamDatabaseAuthenticationEnabled: true,
 *     applyImmediately: true,
 * });
 * ```
 *
 * > **Note:** AWS Neptune does not support user name/password–based access control.
 * See the AWS [Docs](https://docs.aws.amazon.com/neptune/latest/userguide/limits.html) for more information.
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_neptune_cluster` using the cluster identifier. For example:
 *
 * ```sh
 * $ pulumi import aws:neptune/cluster:Cluster example my-cluster
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
     * Whether upgrades between different major versions are allowed. You must set it to `true` when providing an `engineVersion` parameter that uses a different major version than the DB cluster's current version. Default is `false`.
     */
    readonly allowMajorVersionUpgrade: pulumi.Output<boolean>;
    /**
     * Whether any cluster modifications are applied immediately, or during the next maintenance window. Default is `false`.
     */
    readonly applyImmediately: pulumi.Output<boolean>;
    /**
     * Neptune Cluster ARN
     */
    readonly arn: pulumi.Output<string>;
    /**
     * List of EC2 Availability Zones that instances in the Neptune cluster can be created in.
     */
    readonly availabilityZones: pulumi.Output<string[]>;
    /**
     * Days to retain backups for. Default `1`
     */
    readonly backupRetentionPeriod: pulumi.Output<number | undefined>;
    readonly clusterIdentifier: pulumi.Output<string>;
    /**
     * Creates a unique cluster identifier beginning with the specified prefix. Conflicts with `clusterIdentifier`.
     */
    readonly clusterIdentifierPrefix: pulumi.Output<string>;
    /**
     * List of Neptune Instances that are a part of this cluster
     */
    readonly clusterMembers: pulumi.Output<string[]>;
    /**
     * Neptune Cluster Resource ID
     */
    readonly clusterResourceId: pulumi.Output<string>;
    /**
     * If set to true, tags are copied to any snapshot of the DB cluster that is created.
     */
    readonly copyTagsToSnapshot: pulumi.Output<boolean | undefined>;
    /**
     * Value that indicates whether the DB cluster has deletion protection enabled.The database can't be deleted when deletion protection is enabled. By default, deletion protection is disabled.
     */
    readonly deletionProtection: pulumi.Output<boolean | undefined>;
    /**
     * List of the log types this DB cluster is configured to export to Cloudwatch Logs. Currently only supports `audit` and `slowquery`.
     */
    readonly enableCloudwatchLogsExports: pulumi.Output<string[] | undefined>;
    /**
     * DNS address of the Neptune instance
     */
    readonly endpoint: pulumi.Output<string>;
    /**
     * Name of the database engine to be used for this Neptune cluster. Defaults to `neptune`.
     */
    readonly engine: pulumi.Output<string | undefined>;
    /**
     * Database engine version.
     */
    readonly engineVersion: pulumi.Output<string>;
    /**
     * Name of your final Neptune snapshot when this Neptune cluster is deleted. If omitted, no final snapshot will be made.
     */
    readonly finalSnapshotIdentifier: pulumi.Output<string | undefined>;
    /**
     * Global cluster identifier specified on `aws.neptune.GlobalCluster`.
     */
    readonly globalClusterIdentifier: pulumi.Output<string | undefined>;
    /**
     * Route53 Hosted Zone ID of the endpoint
     */
    readonly hostedZoneId: pulumi.Output<string>;
    /**
     * Whether or not mappings of AWS Identity and Access Management (IAM) accounts to database accounts is enabled.
     */
    readonly iamDatabaseAuthenticationEnabled: pulumi.Output<boolean | undefined>;
    /**
     * List of ARNs for the IAM roles to associate to the Neptune Cluster.
     */
    readonly iamRoles: pulumi.Output<string[] | undefined>;
    /**
     * ARN for the KMS encryption key. When specifying `kmsKeyArn`, `storageEncrypted` needs to be set to true.
     */
    readonly kmsKeyArn: pulumi.Output<string>;
    /**
     * Cluster parameter group to associate with the cluster.
     */
    readonly neptuneClusterParameterGroupName: pulumi.Output<string>;
    /**
     * Name of DB parameter group to apply to all instances in the cluster. When upgrading, AWS does not return this value, so do not reference it in other arguments—either leave it unset, configure each instance directly, or ensure it matches the `engineVersion`.
     */
    readonly neptuneInstanceParameterGroupName: pulumi.Output<string | undefined>;
    /**
     * Neptune subnet group to associate with this Neptune instance.
     */
    readonly neptuneSubnetGroupName: pulumi.Output<string>;
    /**
     * Port on which the Neptune accepts connections. Default is `8182`.
     */
    readonly port: pulumi.Output<number | undefined>;
    /**
     * Daily time range during which automated backups are created if automated backups are enabled using the BackupRetentionPeriod parameter. Time in UTC. Default: A 30-minute window selected at random from an 8-hour block of time per regionE.g., 04:00-09:00
     */
    readonly preferredBackupWindow: pulumi.Output<string>;
    /**
     * Weekly time range during which system maintenance can occur, in (UTC) e.g., wed:04:00-wed:04:30
     */
    readonly preferredMaintenanceWindow: pulumi.Output<string>;
    /**
     * Read-only endpoint for the Neptune cluster, automatically load-balanced across replicas
     */
    readonly readerEndpoint: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * ARN of a source Neptune cluster or Neptune instance if this Neptune cluster is to be created as a Read Replica.
     */
    readonly replicationSourceIdentifier: pulumi.Output<string | undefined>;
    /**
     * If set, create the Neptune cluster as a serverless one. See Serverless for example block attributes.
     */
    readonly serverlessV2ScalingConfiguration: pulumi.Output<outputs.neptune.ClusterServerlessV2ScalingConfiguration | undefined>;
    /**
     * Whether a final Neptune snapshot is created before the Neptune cluster is deleted. If true is specified, no Neptune snapshot is created. If false is specified, a Neptune snapshot is created before the Neptune cluster is deleted, using the value from `finalSnapshotIdentifier`. Default is `false`.
     */
    readonly skipFinalSnapshot: pulumi.Output<boolean | undefined>;
    /**
     * Whether or not to create this cluster from a snapshot. You can use either the name or ARN when specifying a Neptune cluster snapshot, or the ARN when specifying a Neptune snapshot. Automated snapshots **should not** be used for this attribute, unless from a different cluster. Automated snapshots are deleted as part of cluster destruction when the resource is replaced.
     */
    readonly snapshotIdentifier: pulumi.Output<string | undefined>;
    /**
     * Whether the Neptune cluster is encrypted. The default is `false` if not specified.
     */
    readonly storageEncrypted: pulumi.Output<boolean | undefined>;
    /**
     * Storage type associated with the cluster `standard/iopt1`. Default: `standard`.
     */
    readonly storageType: pulumi.Output<string>;
    /**
     * Map of tags to assign to the Neptune cluster. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * List of VPC security groups to associate with the Cluster
     */
    readonly vpcSecurityGroupIds: pulumi.Output<string[]>;
    /**
     * Create a Cluster resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: ClusterArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Cluster resources.
 */
export interface ClusterState {
    /**
     * Whether upgrades between different major versions are allowed. You must set it to `true` when providing an `engineVersion` parameter that uses a different major version than the DB cluster's current version. Default is `false`.
     */
    allowMajorVersionUpgrade?: pulumi.Input<boolean>;
    /**
     * Whether any cluster modifications are applied immediately, or during the next maintenance window. Default is `false`.
     */
    applyImmediately?: pulumi.Input<boolean>;
    /**
     * Neptune Cluster ARN
     */
    arn?: pulumi.Input<string>;
    /**
     * List of EC2 Availability Zones that instances in the Neptune cluster can be created in.
     */
    availabilityZones?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Days to retain backups for. Default `1`
     */
    backupRetentionPeriod?: pulumi.Input<number>;
    clusterIdentifier?: pulumi.Input<string>;
    /**
     * Creates a unique cluster identifier beginning with the specified prefix. Conflicts with `clusterIdentifier`.
     */
    clusterIdentifierPrefix?: pulumi.Input<string>;
    /**
     * List of Neptune Instances that are a part of this cluster
     */
    clusterMembers?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Neptune Cluster Resource ID
     */
    clusterResourceId?: pulumi.Input<string>;
    /**
     * If set to true, tags are copied to any snapshot of the DB cluster that is created.
     */
    copyTagsToSnapshot?: pulumi.Input<boolean>;
    /**
     * Value that indicates whether the DB cluster has deletion protection enabled.The database can't be deleted when deletion protection is enabled. By default, deletion protection is disabled.
     */
    deletionProtection?: pulumi.Input<boolean>;
    /**
     * List of the log types this DB cluster is configured to export to Cloudwatch Logs. Currently only supports `audit` and `slowquery`.
     */
    enableCloudwatchLogsExports?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * DNS address of the Neptune instance
     */
    endpoint?: pulumi.Input<string>;
    /**
     * Name of the database engine to be used for this Neptune cluster. Defaults to `neptune`.
     */
    engine?: pulumi.Input<string>;
    /**
     * Database engine version.
     */
    engineVersion?: pulumi.Input<string>;
    /**
     * Name of your final Neptune snapshot when this Neptune cluster is deleted. If omitted, no final snapshot will be made.
     */
    finalSnapshotIdentifier?: pulumi.Input<string>;
    /**
     * Global cluster identifier specified on `aws.neptune.GlobalCluster`.
     */
    globalClusterIdentifier?: pulumi.Input<string>;
    /**
     * Route53 Hosted Zone ID of the endpoint
     */
    hostedZoneId?: pulumi.Input<string>;
    /**
     * Whether or not mappings of AWS Identity and Access Management (IAM) accounts to database accounts is enabled.
     */
    iamDatabaseAuthenticationEnabled?: pulumi.Input<boolean>;
    /**
     * List of ARNs for the IAM roles to associate to the Neptune Cluster.
     */
    iamRoles?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * ARN for the KMS encryption key. When specifying `kmsKeyArn`, `storageEncrypted` needs to be set to true.
     */
    kmsKeyArn?: pulumi.Input<string>;
    /**
     * Cluster parameter group to associate with the cluster.
     */
    neptuneClusterParameterGroupName?: pulumi.Input<string>;
    /**
     * Name of DB parameter group to apply to all instances in the cluster. When upgrading, AWS does not return this value, so do not reference it in other arguments—either leave it unset, configure each instance directly, or ensure it matches the `engineVersion`.
     */
    neptuneInstanceParameterGroupName?: pulumi.Input<string>;
    /**
     * Neptune subnet group to associate with this Neptune instance.
     */
    neptuneSubnetGroupName?: pulumi.Input<string>;
    /**
     * Port on which the Neptune accepts connections. Default is `8182`.
     */
    port?: pulumi.Input<number>;
    /**
     * Daily time range during which automated backups are created if automated backups are enabled using the BackupRetentionPeriod parameter. Time in UTC. Default: A 30-minute window selected at random from an 8-hour block of time per regionE.g., 04:00-09:00
     */
    preferredBackupWindow?: pulumi.Input<string>;
    /**
     * Weekly time range during which system maintenance can occur, in (UTC) e.g., wed:04:00-wed:04:30
     */
    preferredMaintenanceWindow?: pulumi.Input<string>;
    /**
     * Read-only endpoint for the Neptune cluster, automatically load-balanced across replicas
     */
    readerEndpoint?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ARN of a source Neptune cluster or Neptune instance if this Neptune cluster is to be created as a Read Replica.
     */
    replicationSourceIdentifier?: pulumi.Input<string>;
    /**
     * If set, create the Neptune cluster as a serverless one. See Serverless for example block attributes.
     */
    serverlessV2ScalingConfiguration?: pulumi.Input<inputs.neptune.ClusterServerlessV2ScalingConfiguration>;
    /**
     * Whether a final Neptune snapshot is created before the Neptune cluster is deleted. If true is specified, no Neptune snapshot is created. If false is specified, a Neptune snapshot is created before the Neptune cluster is deleted, using the value from `finalSnapshotIdentifier`. Default is `false`.
     */
    skipFinalSnapshot?: pulumi.Input<boolean>;
    /**
     * Whether or not to create this cluster from a snapshot. You can use either the name or ARN when specifying a Neptune cluster snapshot, or the ARN when specifying a Neptune snapshot. Automated snapshots **should not** be used for this attribute, unless from a different cluster. Automated snapshots are deleted as part of cluster destruction when the resource is replaced.
     */
    snapshotIdentifier?: pulumi.Input<string>;
    /**
     * Whether the Neptune cluster is encrypted. The default is `false` if not specified.
     */
    storageEncrypted?: pulumi.Input<boolean>;
    /**
     * Storage type associated with the cluster `standard/iopt1`. Default: `standard`.
     */
    storageType?: pulumi.Input<string>;
    /**
     * Map of tags to assign to the Neptune cluster. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * List of VPC security groups to associate with the Cluster
     */
    vpcSecurityGroupIds?: pulumi.Input<pulumi.Input<string>[]>;
}
/**
 * The set of arguments for constructing a Cluster resource.
 */
export interface ClusterArgs {
    /**
     * Whether upgrades between different major versions are allowed. You must set it to `true` when providing an `engineVersion` parameter that uses a different major version than the DB cluster's current version. Default is `false`.
     */
    allowMajorVersionUpgrade?: pulumi.Input<boolean>;
    /**
     * Whether any cluster modifications are applied immediately, or during the next maintenance window. Default is `false`.
     */
    applyImmediately?: pulumi.Input<boolean>;
    /**
     * List of EC2 Availability Zones that instances in the Neptune cluster can be created in.
     */
    availabilityZones?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Days to retain backups for. Default `1`
     */
    backupRetentionPeriod?: pulumi.Input<number>;
    clusterIdentifier?: pulumi.Input<string>;
    /**
     * Creates a unique cluster identifier beginning with the specified prefix. Conflicts with `clusterIdentifier`.
     */
    clusterIdentifierPrefix?: pulumi.Input<string>;
    /**
     * If set to true, tags are copied to any snapshot of the DB cluster that is created.
     */
    copyTagsToSnapshot?: pulumi.Input<boolean>;
    /**
     * Value that indicates whether the DB cluster has deletion protection enabled.The database can't be deleted when deletion protection is enabled. By default, deletion protection is disabled.
     */
    deletionProtection?: pulumi.Input<boolean>;
    /**
     * List of the log types this DB cluster is configured to export to Cloudwatch Logs. Currently only supports `audit` and `slowquery`.
     */
    enableCloudwatchLogsExports?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Name of the database engine to be used for this Neptune cluster. Defaults to `neptune`.
     */
    engine?: pulumi.Input<string>;
    /**
     * Database engine version.
     */
    engineVersion?: pulumi.Input<string>;
    /**
     * Name of your final Neptune snapshot when this Neptune cluster is deleted. If omitted, no final snapshot will be made.
     */
    finalSnapshotIdentifier?: pulumi.Input<string>;
    /**
     * Global cluster identifier specified on `aws.neptune.GlobalCluster`.
     */
    globalClusterIdentifier?: pulumi.Input<string>;
    /**
     * Whether or not mappings of AWS Identity and Access Management (IAM) accounts to database accounts is enabled.
     */
    iamDatabaseAuthenticationEnabled?: pulumi.Input<boolean>;
    /**
     * List of ARNs for the IAM roles to associate to the Neptune Cluster.
     */
    iamRoles?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * ARN for the KMS encryption key. When specifying `kmsKeyArn`, `storageEncrypted` needs to be set to true.
     */
    kmsKeyArn?: pulumi.Input<string>;
    /**
     * Cluster parameter group to associate with the cluster.
     */
    neptuneClusterParameterGroupName?: pulumi.Input<string>;
    /**
     * Name of DB parameter group to apply to all instances in the cluster. When upgrading, AWS does not return this value, so do not reference it in other arguments—either leave it unset, configure each instance directly, or ensure it matches the `engineVersion`.
     */
    neptuneInstanceParameterGroupName?: pulumi.Input<string>;
    /**
     * Neptune subnet group to associate with this Neptune instance.
     */
    neptuneSubnetGroupName?: pulumi.Input<string>;
    /**
     * Port on which the Neptune accepts connections. Default is `8182`.
     */
    port?: pulumi.Input<number>;
    /**
     * Daily time range during which automated backups are created if automated backups are enabled using the BackupRetentionPeriod parameter. Time in UTC. Default: A 30-minute window selected at random from an 8-hour block of time per regionE.g., 04:00-09:00
     */
    preferredBackupWindow?: pulumi.Input<string>;
    /**
     * Weekly time range during which system maintenance can occur, in (UTC) e.g., wed:04:00-wed:04:30
     */
    preferredMaintenanceWindow?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ARN of a source Neptune cluster or Neptune instance if this Neptune cluster is to be created as a Read Replica.
     */
    replicationSourceIdentifier?: pulumi.Input<string>;
    /**
     * If set, create the Neptune cluster as a serverless one. See Serverless for example block attributes.
     */
    serverlessV2ScalingConfiguration?: pulumi.Input<inputs.neptune.ClusterServerlessV2ScalingConfiguration>;
    /**
     * Whether a final Neptune snapshot is created before the Neptune cluster is deleted. If true is specified, no Neptune snapshot is created. If false is specified, a Neptune snapshot is created before the Neptune cluster is deleted, using the value from `finalSnapshotIdentifier`. Default is `false`.
     */
    skipFinalSnapshot?: pulumi.Input<boolean>;
    /**
     * Whether or not to create this cluster from a snapshot. You can use either the name or ARN when specifying a Neptune cluster snapshot, or the ARN when specifying a Neptune snapshot. Automated snapshots **should not** be used for this attribute, unless from a different cluster. Automated snapshots are deleted as part of cluster destruction when the resource is replaced.
     */
    snapshotIdentifier?: pulumi.Input<string>;
    /**
     * Whether the Neptune cluster is encrypted. The default is `false` if not specified.
     */
    storageEncrypted?: pulumi.Input<boolean>;
    /**
     * Storage type associated with the cluster `standard/iopt1`. Default: `standard`.
     */
    storageType?: pulumi.Input<string>;
    /**
     * Map of tags to assign to the Neptune cluster. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * List of VPC security groups to associate with the Cluster
     */
    vpcSecurityGroupIds?: pulumi.Input<pulumi.Input<string>[]>;
}
