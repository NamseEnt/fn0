import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages an AWS DocDB (DocumentDB) Elastic Cluster.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.docdb.ElasticCluster("example", {
 *     name: "my-docdb-cluster",
 *     adminUserName: "foo",
 *     adminUserPassword: "mustbeeightchars",
 *     authType: "PLAIN_TEXT",
 *     shardCapacity: 2,
 *     shardCount: 1,
 * });
 * ```
 *
 * ## Import
 *
 * ### Identity Schema
 *
 * #### Required
 *
 * - `arn` (String) Amazon Resource Name (ARN) of the DocDB Elastic cluster.
 *
 * Using `pulumi import`, import DocDB (DocumentDB) Elastic Cluster using the `arn` argument. For example,
 *
 * console
 *
 * % pulumi import aws_docdbelastic_cluster.example arn:aws:docdb-elastic:us-east-1:000011112222:cluster/12345678-7abc-def0-1234-56789abcdef
 */
export declare class ElasticCluster extends pulumi.CustomResource {
    /**
     * Get an existing ElasticCluster resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ElasticClusterState, opts?: pulumi.CustomResourceOptions): ElasticCluster;
    /**
     * Returns true if the given object is an instance of ElasticCluster.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ElasticCluster;
    /**
     * Name of the Elastic DocumentDB cluster administrator
     */
    readonly adminUserName: pulumi.Output<string>;
    /**
     * Password for the Elastic DocumentDB cluster administrator. Can contain any printable ASCII characters. Must be at least 8 characters
     */
    readonly adminUserPassword: pulumi.Output<string>;
    /**
     * ARN of the DocumentDB Elastic Cluster
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Authentication type for the Elastic DocumentDB cluster. Valid values are `PLAIN_TEXT` and `SECRET_ARN`
     */
    readonly authType: pulumi.Output<string>;
    /**
     * The number of days for which automatic snapshots are retained. It should be in between 1 and 35. If not specified, the default value of 1 is set.
     */
    readonly backupRetentionPeriod: pulumi.Output<number>;
    /**
     * The DNS address of the DocDB instance
     */
    readonly endpoint: pulumi.Output<string>;
    /**
     * ARN of a KMS key that is used to encrypt the Elastic DocumentDB cluster. If not specified, the default encryption key that KMS creates for your account is used.
     */
    readonly kmsKeyId: pulumi.Output<string>;
    /**
     * Name of the Elastic DocumentDB cluster
     */
    readonly name: pulumi.Output<string>;
    /**
     * The daily time range during which automated backups are created if automated backups are enabled, as determined by the `backupRetentionPeriod`.
     */
    readonly preferredBackupWindow: pulumi.Output<string>;
    /**
     * Weekly time range during which system maintenance can occur in UTC. Format: `ddd:hh24:mi-ddd:hh24:mi`. If not specified, AWS will choose a random 30-minute window on a random day of the week.
     */
    readonly preferredMaintenanceWindow: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Number of vCPUs assigned to each elastic cluster shard. Maximum is 64. Allowed values are 2, 4, 8, 16, 32, 64
     */
    readonly shardCapacity: pulumi.Output<number>;
    /**
     * Number of shards assigned to the elastic cluster. Maximum is 32
     *
     * The following arguments are optional:
     */
    readonly shardCount: pulumi.Output<number>;
    /**
     * IDs of subnets in which the Elastic DocumentDB Cluster operates.
     */
    readonly subnetIds: pulumi.Output<string[]>;
    /**
     * A map of tags to assign to the collection. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    readonly timeouts: pulumi.Output<outputs.docdb.ElasticClusterTimeouts | undefined>;
    /**
     * List of VPC security groups to associate with the Elastic DocumentDB Cluster
     *
     * For more detailed documentation about each argument, refer to
     * the [AWS official documentation](https://docs.aws.amazon.com/cli/latest/reference/docdb-elastic/create-cluster.html).
     */
    readonly vpcSecurityGroupIds: pulumi.Output<string[]>;
    /**
     * Create a ElasticCluster resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ElasticClusterArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ElasticCluster resources.
 */
export interface ElasticClusterState {
    /**
     * Name of the Elastic DocumentDB cluster administrator
     */
    adminUserName?: pulumi.Input<string>;
    /**
     * Password for the Elastic DocumentDB cluster administrator. Can contain any printable ASCII characters. Must be at least 8 characters
     */
    adminUserPassword?: pulumi.Input<string>;
    /**
     * ARN of the DocumentDB Elastic Cluster
     */
    arn?: pulumi.Input<string>;
    /**
     * Authentication type for the Elastic DocumentDB cluster. Valid values are `PLAIN_TEXT` and `SECRET_ARN`
     */
    authType?: pulumi.Input<string>;
    /**
     * The number of days for which automatic snapshots are retained. It should be in between 1 and 35. If not specified, the default value of 1 is set.
     */
    backupRetentionPeriod?: pulumi.Input<number>;
    /**
     * The DNS address of the DocDB instance
     */
    endpoint?: pulumi.Input<string>;
    /**
     * ARN of a KMS key that is used to encrypt the Elastic DocumentDB cluster. If not specified, the default encryption key that KMS creates for your account is used.
     */
    kmsKeyId?: pulumi.Input<string>;
    /**
     * Name of the Elastic DocumentDB cluster
     */
    name?: pulumi.Input<string>;
    /**
     * The daily time range during which automated backups are created if automated backups are enabled, as determined by the `backupRetentionPeriod`.
     */
    preferredBackupWindow?: pulumi.Input<string>;
    /**
     * Weekly time range during which system maintenance can occur in UTC. Format: `ddd:hh24:mi-ddd:hh24:mi`. If not specified, AWS will choose a random 30-minute window on a random day of the week.
     */
    preferredMaintenanceWindow?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Number of vCPUs assigned to each elastic cluster shard. Maximum is 64. Allowed values are 2, 4, 8, 16, 32, 64
     */
    shardCapacity?: pulumi.Input<number>;
    /**
     * Number of shards assigned to the elastic cluster. Maximum is 32
     *
     * The following arguments are optional:
     */
    shardCount?: pulumi.Input<number>;
    /**
     * IDs of subnets in which the Elastic DocumentDB Cluster operates.
     */
    subnetIds?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * A map of tags to assign to the collection. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    timeouts?: pulumi.Input<inputs.docdb.ElasticClusterTimeouts>;
    /**
     * List of VPC security groups to associate with the Elastic DocumentDB Cluster
     *
     * For more detailed documentation about each argument, refer to
     * the [AWS official documentation](https://docs.aws.amazon.com/cli/latest/reference/docdb-elastic/create-cluster.html).
     */
    vpcSecurityGroupIds?: pulumi.Input<pulumi.Input<string>[]>;
}
/**
 * The set of arguments for constructing a ElasticCluster resource.
 */
export interface ElasticClusterArgs {
    /**
     * Name of the Elastic DocumentDB cluster administrator
     */
    adminUserName: pulumi.Input<string>;
    /**
     * Password for the Elastic DocumentDB cluster administrator. Can contain any printable ASCII characters. Must be at least 8 characters
     */
    adminUserPassword: pulumi.Input<string>;
    /**
     * Authentication type for the Elastic DocumentDB cluster. Valid values are `PLAIN_TEXT` and `SECRET_ARN`
     */
    authType: pulumi.Input<string>;
    /**
     * The number of days for which automatic snapshots are retained. It should be in between 1 and 35. If not specified, the default value of 1 is set.
     */
    backupRetentionPeriod?: pulumi.Input<number>;
    /**
     * ARN of a KMS key that is used to encrypt the Elastic DocumentDB cluster. If not specified, the default encryption key that KMS creates for your account is used.
     */
    kmsKeyId?: pulumi.Input<string>;
    /**
     * Name of the Elastic DocumentDB cluster
     */
    name?: pulumi.Input<string>;
    /**
     * The daily time range during which automated backups are created if automated backups are enabled, as determined by the `backupRetentionPeriod`.
     */
    preferredBackupWindow?: pulumi.Input<string>;
    /**
     * Weekly time range during which system maintenance can occur in UTC. Format: `ddd:hh24:mi-ddd:hh24:mi`. If not specified, AWS will choose a random 30-minute window on a random day of the week.
     */
    preferredMaintenanceWindow?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Number of vCPUs assigned to each elastic cluster shard. Maximum is 64. Allowed values are 2, 4, 8, 16, 32, 64
     */
    shardCapacity: pulumi.Input<number>;
    /**
     * Number of shards assigned to the elastic cluster. Maximum is 32
     *
     * The following arguments are optional:
     */
    shardCount: pulumi.Input<number>;
    /**
     * IDs of subnets in which the Elastic DocumentDB Cluster operates.
     */
    subnetIds?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * A map of tags to assign to the collection. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    timeouts?: pulumi.Input<inputs.docdb.ElasticClusterTimeouts>;
    /**
     * List of VPC security groups to associate with the Elastic DocumentDB Cluster
     *
     * For more detailed documentation about each argument, refer to
     * the [AWS official documentation](https://docs.aws.amazon.com/cli/latest/reference/docdb-elastic/create-cluster.html).
     */
    vpcSecurityGroupIds?: pulumi.Input<pulumi.Input<string>[]>;
}
