import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.redshift.Cluster("example", {
 *     clusterIdentifier: "tf-redshift-cluster",
 *     databaseName: "mydb",
 *     masterUsername: "exampleuser",
 *     masterPassword: "Mustbe8characters",
 *     nodeType: "dc1.large",
 *     clusterType: "single-node",
 * });
 * ```
 *
 * ### With Managed Credentials
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.redshift.Cluster("example", {
 *     clusterIdentifier: "tf-redshift-cluster",
 *     databaseName: "mydb",
 *     masterUsername: "exampleuser",
 *     nodeType: "dc1.large",
 *     clusterType: "single-node",
 *     manageMasterPassword: true,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Redshift Clusters using the `cluster_identifier`. For example:
 *
 * ```sh
 * $ pulumi import aws:redshift/cluster:Cluster myprodcluster tf-redshift-cluster-12345
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
     * If true , major version upgrades can be applied during the maintenance window to the Amazon Redshift engine that is running on the cluster. Default is `true`.
     */
    readonly allowVersionUpgrade: pulumi.Output<boolean | undefined>;
    /**
     * Specifies whether any cluster modifications are applied immediately, or during the next maintenance window. Default is `false`.
     */
    readonly applyImmediately: pulumi.Output<boolean | undefined>;
    /**
     * The value represents how the cluster is configured to use AQUA (Advanced Query Accelerator) after the cluster is restored.
     * No longer supported by the AWS API.
     * Always returns `auto`.
     *
     * @deprecated aqua_configuration_status is deprecated. This parameter is no longer supported by the AWS API. It will be removed in the next major version of the provider.
     */
    readonly aquaConfigurationStatus: pulumi.Output<string>;
    /**
     * Amazon Resource Name (ARN) of cluster
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The number of days that automated snapshots are retained. If the value is 0, automated snapshots are disabled. Even if automated snapshots are disabled, you can still create manual snapshots when you want with create-cluster-snapshot. Default is 1.
     */
    readonly automatedSnapshotRetentionPeriod: pulumi.Output<number | undefined>;
    /**
     * The EC2 Availability Zone (AZ) in which you want Amazon Redshift to provision the cluster. For example, if you have several EC2 instances running in a specific Availability Zone, then you might want the cluster to be provisioned in the same zone in order to decrease network latency. Can only be changed if `availabilityZoneRelocationEnabled` is `true`.
     */
    readonly availabilityZone: pulumi.Output<string>;
    /**
     * If true, the cluster can be relocated to another availabity zone, either automatically by AWS or when requested. Default is `false`. Available for use on clusters from the RA3 instance family.
     */
    readonly availabilityZoneRelocationEnabled: pulumi.Output<boolean | undefined>;
    /**
     * The Cluster Identifier. Must be a lower case string.
     */
    readonly clusterIdentifier: pulumi.Output<string>;
    /**
     * The namespace Amazon Resource Name (ARN) of the cluster
     */
    readonly clusterNamespaceArn: pulumi.Output<string>;
    /**
     * The nodes in the cluster. Cluster node blocks are documented below
     */
    readonly clusterNodes: pulumi.Output<outputs.redshift.ClusterClusterNode[]>;
    /**
     * The name of the parameter group to be associated with this cluster.
     */
    readonly clusterParameterGroupName: pulumi.Output<string>;
    /**
     * The public key for the cluster
     */
    readonly clusterPublicKey: pulumi.Output<string>;
    /**
     * The specific revision number of the database in the cluster
     */
    readonly clusterRevisionNumber: pulumi.Output<string>;
    /**
     * The name of a cluster subnet group to be associated with this cluster. If this parameter is not provided the resulting cluster will be deployed outside virtual private cloud (VPC).
     */
    readonly clusterSubnetGroupName: pulumi.Output<string>;
    /**
     * The cluster type to use. Either `single-node` or `multi-node`.
     */
    readonly clusterType: pulumi.Output<string>;
    /**
     * The version of the Amazon Redshift engine software that you want to deploy on the cluster.
     * The version selected runs on all the nodes in the cluster.
     */
    readonly clusterVersion: pulumi.Output<string | undefined>;
    /**
     * The name of the first database to be created when the cluster is created.
     * If you do not provide a name, Amazon Redshift will create a default database called `dev`.
     */
    readonly databaseName: pulumi.Output<string>;
    /**
     * The Amazon Resource Name (ARN) for the IAM role that was set as default for the cluster when the cluster was created.
     */
    readonly defaultIamRoleArn: pulumi.Output<string>;
    /**
     * The DNS name of the cluster
     */
    readonly dnsName: pulumi.Output<string>;
    /**
     * The Elastic IP (EIP) address for the cluster.
     */
    readonly elasticIp: pulumi.Output<string | undefined>;
    /**
     * If true , the data in the cluster is encrypted at rest.
     * Default is `true`.
     */
    readonly encrypted: pulumi.Output<string | undefined>;
    /**
     * The connection endpoint
     */
    readonly endpoint: pulumi.Output<string>;
    /**
     * If true , enhanced VPC routing is enabled.
     */
    readonly enhancedVpcRouting: pulumi.Output<boolean>;
    /**
     * The identifier of the final snapshot that is to be created immediately before deleting the cluster. If this parameter is provided, `skipFinalSnapshot` must be false.
     */
    readonly finalSnapshotIdentifier: pulumi.Output<string | undefined>;
    /**
     * A list of IAM Role ARNs to associate with the cluster. A Maximum of 10 can be associated to the cluster at any time.
     */
    readonly iamRoles: pulumi.Output<string[]>;
    /**
     * The ARN for the KMS encryption key. When specifying `kmsKeyId`, `encrypted` needs to be set to true.
     */
    readonly kmsKeyId: pulumi.Output<string>;
    /**
     * The name of the maintenance track for the restored cluster. When you take a snapshot, the snapshot inherits the MaintenanceTrack value from the cluster. The snapshot might be on a different track than the cluster that was the source for the snapshot. For example, suppose that you take a snapshot of  a cluster that is on the current track and then change the cluster to be on the trailing track. In this case, the snapshot and the source cluster are on different tracks. Default value is `current`.
     */
    readonly maintenanceTrackName: pulumi.Output<string | undefined>;
    /**
     * Whether to use AWS SecretsManager to manage the cluster admin credentials.
     * Conflicts with `masterPassword` and `masterPasswordWo`.
     * One of `masterPassword` or `manageMasterPassword` is required unless `snapshotIdentifier` is provided.
     */
    readonly manageMasterPassword: pulumi.Output<boolean | undefined>;
    /**
     * The default number of days to retain a manual snapshot. If the value is -1, the snapshot is retained indefinitely. This setting doesn't change the retention period of existing snapshots. Valid values are between `-1` and `3653`. Default value is `-1`.
     */
    readonly manualSnapshotRetentionPeriod: pulumi.Output<number | undefined>;
    /**
     * Password for the master DB user.
     * Conflicts with `manageMasterPassword` and `masterPasswordWo`.
     * One of `masterPassword`, `masterPasswordWo` or `manageMasterPassword` is required unless `snapshotIdentifier` is provided.
     * Note that this may show up in logs, and it will be stored in the state file.
     * Password must contain at least 8 characters and contain at least one uppercase letter, one lowercase letter, and one number.
     */
    readonly masterPassword: pulumi.Output<string | undefined>;
    /**
     * ARN of the cluster admin credentials secret
     */
    readonly masterPasswordSecretArn: pulumi.Output<string>;
    /**
     * ID of the KMS key used to encrypt the cluster admin credentials secret.
     */
    readonly masterPasswordSecretKmsKeyId: pulumi.Output<string>;
    /**
     * **NOTE:** This field is write-only and its value will not be updated in state as part of read operations.
     * Password for the master DB user.
     * Conflicts with `manageMasterPassword` and `masterPassword`.
     * One of `masterPasswordWo`, `masterPassword` or `manageMasterPassword` is required unless `snapshotIdentifier` is provided.
     * Note that this may show up in logs.
     * Password must contain at least 8 characters and contain at least one uppercase letter, one lowercase letter, and one number.
     */
    readonly masterPasswordWo: pulumi.Output<string | undefined>;
    /**
     * Used together with `masterPasswordWo` to trigger an update. Increment this value when an update to the `masterPasswordWo` is required.
     */
    readonly masterPasswordWoVersion: pulumi.Output<number | undefined>;
    /**
     * Username for the master DB user.
     */
    readonly masterUsername: pulumi.Output<string | undefined>;
    /**
     * Specifies if the Redshift cluster is multi-AZ.
     */
    readonly multiAz: pulumi.Output<boolean | undefined>;
    /**
     * The node type to be provisioned for the cluster.
     */
    readonly nodeType: pulumi.Output<string>;
    /**
     * The number of compute nodes in the cluster. This parameter is required when the ClusterType parameter is specified as multi-node. Default is 1.
     */
    readonly numberOfNodes: pulumi.Output<number | undefined>;
    /**
     * The AWS customer account used to create or copy the snapshot. Required if you are restoring a snapshot you do not own, optional if you own the snapshot.
     */
    readonly ownerAccount: pulumi.Output<string | undefined>;
    /**
     * The port number on which the cluster accepts incoming connections. Valid values are between `1115` and `65535`.
     * The cluster is accessible only via the JDBC and ODBC connection strings.
     * Part of the connection string requires the port on which the cluster will listen for incoming connections.
     * Default port is `5439`.
     */
    readonly port: pulumi.Output<number | undefined>;
    /**
     * The weekly time range (in UTC) during which automated cluster maintenance can occur.
     * Format: ddd:hh24:mi-ddd:hh24:mi
     */
    readonly preferredMaintenanceWindow: pulumi.Output<string>;
    /**
     * If true, the cluster can be accessed from a public network. Default is `false`.
     */
    readonly publiclyAccessible: pulumi.Output<boolean | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Determines whether a final snapshot of the cluster is created before Amazon Redshift deletes the cluster. If true , a final cluster snapshot is not created. If false , a final cluster snapshot is created before the cluster is deleted. Default is false.
     */
    readonly skipFinalSnapshot: pulumi.Output<boolean | undefined>;
    /**
     * The ARN of the snapshot from which to create the new cluster. Conflicts with `snapshotIdentifier`.
     */
    readonly snapshotArn: pulumi.Output<string | undefined>;
    /**
     * The name of the cluster the source snapshot was created from.
     */
    readonly snapshotClusterIdentifier: pulumi.Output<string | undefined>;
    /**
     * The name of the snapshot from which to create the new cluster.  Conflicts with `snapshotArn`.
     */
    readonly snapshotIdentifier: pulumi.Output<string | undefined>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     *
     * For more detailed documentation about each argument, refer to
     * the [AWS official documentation](http://docs.aws.amazon.com/cli/latest/reference/redshift/index.html#cli-aws-redshift).
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
     * A list of Virtual Private Cloud (VPC) security groups to be associated with the cluster.
     */
    readonly vpcSecurityGroupIds: pulumi.Output<string[]>;
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
     * If true , major version upgrades can be applied during the maintenance window to the Amazon Redshift engine that is running on the cluster. Default is `true`.
     */
    allowVersionUpgrade?: pulumi.Input<boolean>;
    /**
     * Specifies whether any cluster modifications are applied immediately, or during the next maintenance window. Default is `false`.
     */
    applyImmediately?: pulumi.Input<boolean>;
    /**
     * The value represents how the cluster is configured to use AQUA (Advanced Query Accelerator) after the cluster is restored.
     * No longer supported by the AWS API.
     * Always returns `auto`.
     *
     * @deprecated aqua_configuration_status is deprecated. This parameter is no longer supported by the AWS API. It will be removed in the next major version of the provider.
     */
    aquaConfigurationStatus?: pulumi.Input<string>;
    /**
     * Amazon Resource Name (ARN) of cluster
     */
    arn?: pulumi.Input<string>;
    /**
     * The number of days that automated snapshots are retained. If the value is 0, automated snapshots are disabled. Even if automated snapshots are disabled, you can still create manual snapshots when you want with create-cluster-snapshot. Default is 1.
     */
    automatedSnapshotRetentionPeriod?: pulumi.Input<number>;
    /**
     * The EC2 Availability Zone (AZ) in which you want Amazon Redshift to provision the cluster. For example, if you have several EC2 instances running in a specific Availability Zone, then you might want the cluster to be provisioned in the same zone in order to decrease network latency. Can only be changed if `availabilityZoneRelocationEnabled` is `true`.
     */
    availabilityZone?: pulumi.Input<string>;
    /**
     * If true, the cluster can be relocated to another availabity zone, either automatically by AWS or when requested. Default is `false`. Available for use on clusters from the RA3 instance family.
     */
    availabilityZoneRelocationEnabled?: pulumi.Input<boolean>;
    /**
     * The Cluster Identifier. Must be a lower case string.
     */
    clusterIdentifier?: pulumi.Input<string>;
    /**
     * The namespace Amazon Resource Name (ARN) of the cluster
     */
    clusterNamespaceArn?: pulumi.Input<string>;
    /**
     * The nodes in the cluster. Cluster node blocks are documented below
     */
    clusterNodes?: pulumi.Input<pulumi.Input<inputs.redshift.ClusterClusterNode>[]>;
    /**
     * The name of the parameter group to be associated with this cluster.
     */
    clusterParameterGroupName?: pulumi.Input<string>;
    /**
     * The public key for the cluster
     */
    clusterPublicKey?: pulumi.Input<string>;
    /**
     * The specific revision number of the database in the cluster
     */
    clusterRevisionNumber?: pulumi.Input<string>;
    /**
     * The name of a cluster subnet group to be associated with this cluster. If this parameter is not provided the resulting cluster will be deployed outside virtual private cloud (VPC).
     */
    clusterSubnetGroupName?: pulumi.Input<string>;
    /**
     * The cluster type to use. Either `single-node` or `multi-node`.
     */
    clusterType?: pulumi.Input<string>;
    /**
     * The version of the Amazon Redshift engine software that you want to deploy on the cluster.
     * The version selected runs on all the nodes in the cluster.
     */
    clusterVersion?: pulumi.Input<string>;
    /**
     * The name of the first database to be created when the cluster is created.
     * If you do not provide a name, Amazon Redshift will create a default database called `dev`.
     */
    databaseName?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) for the IAM role that was set as default for the cluster when the cluster was created.
     */
    defaultIamRoleArn?: pulumi.Input<string>;
    /**
     * The DNS name of the cluster
     */
    dnsName?: pulumi.Input<string>;
    /**
     * The Elastic IP (EIP) address for the cluster.
     */
    elasticIp?: pulumi.Input<string>;
    /**
     * If true , the data in the cluster is encrypted at rest.
     * Default is `true`.
     */
    encrypted?: pulumi.Input<string>;
    /**
     * The connection endpoint
     */
    endpoint?: pulumi.Input<string>;
    /**
     * If true , enhanced VPC routing is enabled.
     */
    enhancedVpcRouting?: pulumi.Input<boolean>;
    /**
     * The identifier of the final snapshot that is to be created immediately before deleting the cluster. If this parameter is provided, `skipFinalSnapshot` must be false.
     */
    finalSnapshotIdentifier?: pulumi.Input<string>;
    /**
     * A list of IAM Role ARNs to associate with the cluster. A Maximum of 10 can be associated to the cluster at any time.
     */
    iamRoles?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The ARN for the KMS encryption key. When specifying `kmsKeyId`, `encrypted` needs to be set to true.
     */
    kmsKeyId?: pulumi.Input<string>;
    /**
     * The name of the maintenance track for the restored cluster. When you take a snapshot, the snapshot inherits the MaintenanceTrack value from the cluster. The snapshot might be on a different track than the cluster that was the source for the snapshot. For example, suppose that you take a snapshot of  a cluster that is on the current track and then change the cluster to be on the trailing track. In this case, the snapshot and the source cluster are on different tracks. Default value is `current`.
     */
    maintenanceTrackName?: pulumi.Input<string>;
    /**
     * Whether to use AWS SecretsManager to manage the cluster admin credentials.
     * Conflicts with `masterPassword` and `masterPasswordWo`.
     * One of `masterPassword` or `manageMasterPassword` is required unless `snapshotIdentifier` is provided.
     */
    manageMasterPassword?: pulumi.Input<boolean>;
    /**
     * The default number of days to retain a manual snapshot. If the value is -1, the snapshot is retained indefinitely. This setting doesn't change the retention period of existing snapshots. Valid values are between `-1` and `3653`. Default value is `-1`.
     */
    manualSnapshotRetentionPeriod?: pulumi.Input<number>;
    /**
     * Password for the master DB user.
     * Conflicts with `manageMasterPassword` and `masterPasswordWo`.
     * One of `masterPassword`, `masterPasswordWo` or `manageMasterPassword` is required unless `snapshotIdentifier` is provided.
     * Note that this may show up in logs, and it will be stored in the state file.
     * Password must contain at least 8 characters and contain at least one uppercase letter, one lowercase letter, and one number.
     */
    masterPassword?: pulumi.Input<string>;
    /**
     * ARN of the cluster admin credentials secret
     */
    masterPasswordSecretArn?: pulumi.Input<string>;
    /**
     * ID of the KMS key used to encrypt the cluster admin credentials secret.
     */
    masterPasswordSecretKmsKeyId?: pulumi.Input<string>;
    /**
     * **NOTE:** This field is write-only and its value will not be updated in state as part of read operations.
     * Password for the master DB user.
     * Conflicts with `manageMasterPassword` and `masterPassword`.
     * One of `masterPasswordWo`, `masterPassword` or `manageMasterPassword` is required unless `snapshotIdentifier` is provided.
     * Note that this may show up in logs.
     * Password must contain at least 8 characters and contain at least one uppercase letter, one lowercase letter, and one number.
     */
    masterPasswordWo?: pulumi.Input<string>;
    /**
     * Used together with `masterPasswordWo` to trigger an update. Increment this value when an update to the `masterPasswordWo` is required.
     */
    masterPasswordWoVersion?: pulumi.Input<number>;
    /**
     * Username for the master DB user.
     */
    masterUsername?: pulumi.Input<string>;
    /**
     * Specifies if the Redshift cluster is multi-AZ.
     */
    multiAz?: pulumi.Input<boolean>;
    /**
     * The node type to be provisioned for the cluster.
     */
    nodeType?: pulumi.Input<string>;
    /**
     * The number of compute nodes in the cluster. This parameter is required when the ClusterType parameter is specified as multi-node. Default is 1.
     */
    numberOfNodes?: pulumi.Input<number>;
    /**
     * The AWS customer account used to create or copy the snapshot. Required if you are restoring a snapshot you do not own, optional if you own the snapshot.
     */
    ownerAccount?: pulumi.Input<string>;
    /**
     * The port number on which the cluster accepts incoming connections. Valid values are between `1115` and `65535`.
     * The cluster is accessible only via the JDBC and ODBC connection strings.
     * Part of the connection string requires the port on which the cluster will listen for incoming connections.
     * Default port is `5439`.
     */
    port?: pulumi.Input<number>;
    /**
     * The weekly time range (in UTC) during which automated cluster maintenance can occur.
     * Format: ddd:hh24:mi-ddd:hh24:mi
     */
    preferredMaintenanceWindow?: pulumi.Input<string>;
    /**
     * If true, the cluster can be accessed from a public network. Default is `false`.
     */
    publiclyAccessible?: pulumi.Input<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Determines whether a final snapshot of the cluster is created before Amazon Redshift deletes the cluster. If true , a final cluster snapshot is not created. If false , a final cluster snapshot is created before the cluster is deleted. Default is false.
     */
    skipFinalSnapshot?: pulumi.Input<boolean>;
    /**
     * The ARN of the snapshot from which to create the new cluster. Conflicts with `snapshotIdentifier`.
     */
    snapshotArn?: pulumi.Input<string>;
    /**
     * The name of the cluster the source snapshot was created from.
     */
    snapshotClusterIdentifier?: pulumi.Input<string>;
    /**
     * The name of the snapshot from which to create the new cluster.  Conflicts with `snapshotArn`.
     */
    snapshotIdentifier?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     *
     * For more detailed documentation about each argument, refer to
     * the [AWS official documentation](http://docs.aws.amazon.com/cli/latest/reference/redshift/index.html#cli-aws-redshift).
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
     * A list of Virtual Private Cloud (VPC) security groups to be associated with the cluster.
     */
    vpcSecurityGroupIds?: pulumi.Input<pulumi.Input<string>[]>;
}
/**
 * The set of arguments for constructing a Cluster resource.
 */
export interface ClusterArgs {
    /**
     * If true , major version upgrades can be applied during the maintenance window to the Amazon Redshift engine that is running on the cluster. Default is `true`.
     */
    allowVersionUpgrade?: pulumi.Input<boolean>;
    /**
     * Specifies whether any cluster modifications are applied immediately, or during the next maintenance window. Default is `false`.
     */
    applyImmediately?: pulumi.Input<boolean>;
    /**
     * The value represents how the cluster is configured to use AQUA (Advanced Query Accelerator) after the cluster is restored.
     * No longer supported by the AWS API.
     * Always returns `auto`.
     *
     * @deprecated aqua_configuration_status is deprecated. This parameter is no longer supported by the AWS API. It will be removed in the next major version of the provider.
     */
    aquaConfigurationStatus?: pulumi.Input<string>;
    /**
     * The number of days that automated snapshots are retained. If the value is 0, automated snapshots are disabled. Even if automated snapshots are disabled, you can still create manual snapshots when you want with create-cluster-snapshot. Default is 1.
     */
    automatedSnapshotRetentionPeriod?: pulumi.Input<number>;
    /**
     * The EC2 Availability Zone (AZ) in which you want Amazon Redshift to provision the cluster. For example, if you have several EC2 instances running in a specific Availability Zone, then you might want the cluster to be provisioned in the same zone in order to decrease network latency. Can only be changed if `availabilityZoneRelocationEnabled` is `true`.
     */
    availabilityZone?: pulumi.Input<string>;
    /**
     * If true, the cluster can be relocated to another availabity zone, either automatically by AWS or when requested. Default is `false`. Available for use on clusters from the RA3 instance family.
     */
    availabilityZoneRelocationEnabled?: pulumi.Input<boolean>;
    /**
     * The Cluster Identifier. Must be a lower case string.
     */
    clusterIdentifier: pulumi.Input<string>;
    /**
     * The name of the parameter group to be associated with this cluster.
     */
    clusterParameterGroupName?: pulumi.Input<string>;
    /**
     * The name of a cluster subnet group to be associated with this cluster. If this parameter is not provided the resulting cluster will be deployed outside virtual private cloud (VPC).
     */
    clusterSubnetGroupName?: pulumi.Input<string>;
    /**
     * The cluster type to use. Either `single-node` or `multi-node`.
     */
    clusterType?: pulumi.Input<string>;
    /**
     * The version of the Amazon Redshift engine software that you want to deploy on the cluster.
     * The version selected runs on all the nodes in the cluster.
     */
    clusterVersion?: pulumi.Input<string>;
    /**
     * The name of the first database to be created when the cluster is created.
     * If you do not provide a name, Amazon Redshift will create a default database called `dev`.
     */
    databaseName?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) for the IAM role that was set as default for the cluster when the cluster was created.
     */
    defaultIamRoleArn?: pulumi.Input<string>;
    /**
     * The Elastic IP (EIP) address for the cluster.
     */
    elasticIp?: pulumi.Input<string>;
    /**
     * If true , the data in the cluster is encrypted at rest.
     * Default is `true`.
     */
    encrypted?: pulumi.Input<string>;
    /**
     * If true , enhanced VPC routing is enabled.
     */
    enhancedVpcRouting?: pulumi.Input<boolean>;
    /**
     * The identifier of the final snapshot that is to be created immediately before deleting the cluster. If this parameter is provided, `skipFinalSnapshot` must be false.
     */
    finalSnapshotIdentifier?: pulumi.Input<string>;
    /**
     * A list of IAM Role ARNs to associate with the cluster. A Maximum of 10 can be associated to the cluster at any time.
     */
    iamRoles?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The ARN for the KMS encryption key. When specifying `kmsKeyId`, `encrypted` needs to be set to true.
     */
    kmsKeyId?: pulumi.Input<string>;
    /**
     * The name of the maintenance track for the restored cluster. When you take a snapshot, the snapshot inherits the MaintenanceTrack value from the cluster. The snapshot might be on a different track than the cluster that was the source for the snapshot. For example, suppose that you take a snapshot of  a cluster that is on the current track and then change the cluster to be on the trailing track. In this case, the snapshot and the source cluster are on different tracks. Default value is `current`.
     */
    maintenanceTrackName?: pulumi.Input<string>;
    /**
     * Whether to use AWS SecretsManager to manage the cluster admin credentials.
     * Conflicts with `masterPassword` and `masterPasswordWo`.
     * One of `masterPassword` or `manageMasterPassword` is required unless `snapshotIdentifier` is provided.
     */
    manageMasterPassword?: pulumi.Input<boolean>;
    /**
     * The default number of days to retain a manual snapshot. If the value is -1, the snapshot is retained indefinitely. This setting doesn't change the retention period of existing snapshots. Valid values are between `-1` and `3653`. Default value is `-1`.
     */
    manualSnapshotRetentionPeriod?: pulumi.Input<number>;
    /**
     * Password for the master DB user.
     * Conflicts with `manageMasterPassword` and `masterPasswordWo`.
     * One of `masterPassword`, `masterPasswordWo` or `manageMasterPassword` is required unless `snapshotIdentifier` is provided.
     * Note that this may show up in logs, and it will be stored in the state file.
     * Password must contain at least 8 characters and contain at least one uppercase letter, one lowercase letter, and one number.
     */
    masterPassword?: pulumi.Input<string>;
    /**
     * ID of the KMS key used to encrypt the cluster admin credentials secret.
     */
    masterPasswordSecretKmsKeyId?: pulumi.Input<string>;
    /**
     * **NOTE:** This field is write-only and its value will not be updated in state as part of read operations.
     * Password for the master DB user.
     * Conflicts with `manageMasterPassword` and `masterPassword`.
     * One of `masterPasswordWo`, `masterPassword` or `manageMasterPassword` is required unless `snapshotIdentifier` is provided.
     * Note that this may show up in logs.
     * Password must contain at least 8 characters and contain at least one uppercase letter, one lowercase letter, and one number.
     */
    masterPasswordWo?: pulumi.Input<string>;
    /**
     * Used together with `masterPasswordWo` to trigger an update. Increment this value when an update to the `masterPasswordWo` is required.
     */
    masterPasswordWoVersion?: pulumi.Input<number>;
    /**
     * Username for the master DB user.
     */
    masterUsername?: pulumi.Input<string>;
    /**
     * Specifies if the Redshift cluster is multi-AZ.
     */
    multiAz?: pulumi.Input<boolean>;
    /**
     * The node type to be provisioned for the cluster.
     */
    nodeType: pulumi.Input<string>;
    /**
     * The number of compute nodes in the cluster. This parameter is required when the ClusterType parameter is specified as multi-node. Default is 1.
     */
    numberOfNodes?: pulumi.Input<number>;
    /**
     * The AWS customer account used to create or copy the snapshot. Required if you are restoring a snapshot you do not own, optional if you own the snapshot.
     */
    ownerAccount?: pulumi.Input<string>;
    /**
     * The port number on which the cluster accepts incoming connections. Valid values are between `1115` and `65535`.
     * The cluster is accessible only via the JDBC and ODBC connection strings.
     * Part of the connection string requires the port on which the cluster will listen for incoming connections.
     * Default port is `5439`.
     */
    port?: pulumi.Input<number>;
    /**
     * The weekly time range (in UTC) during which automated cluster maintenance can occur.
     * Format: ddd:hh24:mi-ddd:hh24:mi
     */
    preferredMaintenanceWindow?: pulumi.Input<string>;
    /**
     * If true, the cluster can be accessed from a public network. Default is `false`.
     */
    publiclyAccessible?: pulumi.Input<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Determines whether a final snapshot of the cluster is created before Amazon Redshift deletes the cluster. If true , a final cluster snapshot is not created. If false , a final cluster snapshot is created before the cluster is deleted. Default is false.
     */
    skipFinalSnapshot?: pulumi.Input<boolean>;
    /**
     * The ARN of the snapshot from which to create the new cluster. Conflicts with `snapshotIdentifier`.
     */
    snapshotArn?: pulumi.Input<string>;
    /**
     * The name of the cluster the source snapshot was created from.
     */
    snapshotClusterIdentifier?: pulumi.Input<string>;
    /**
     * The name of the snapshot from which to create the new cluster.  Conflicts with `snapshotArn`.
     */
    snapshotIdentifier?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     *
     * For more detailed documentation about each argument, refer to
     * the [AWS official documentation](http://docs.aws.amazon.com/cli/latest/reference/redshift/index.html#cli-aws-redshift).
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * A list of Virtual Private Cloud (VPC) security groups to be associated with the cluster.
     */
    vpcSecurityGroupIds?: pulumi.Input<pulumi.Input<string>[]>;
}
