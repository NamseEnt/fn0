import * as pulumi from "@pulumi/pulumi";
/**
 * Manages RDS Aurora Cluster Database Activity Streams.
 *
 * Database Activity Streams have some limits and requirements, refer to the [Monitoring Amazon Aurora using Database Activity Streams](https://docs.aws.amazon.com/AmazonRDS/latest/AuroraUserGuide/DBActivityStreams.html) documentation for detailed limitations and requirements.
 *
 * > **Note:** This resource always calls the RDS [`StartActivityStream`][2] API with the `ApplyImmediately` parameter set to `true`. This is because the provider needs the activity stream to be started in order for it to get the associated attributes.
 *
 * > **Note:** This resource depends on having at least one `aws.rds.ClusterInstance` created. To avoid race conditions when all resources are being created together, add an explicit resource reference using the resource `dependsOn` meta-argument.
 *
 * > **Note:** This resource is available in all regions except the following: `cn-north-1`, `cn-northwest-1`, `us-gov-east-1`, `us-gov-west-1`
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
 *     masterPassword: "mustbeeightcharaters",
 *     engine: aws.rds.EngineType.AuroraPostgresql,
 *     engineVersion: "13.4",
 * });
 * const defaultClusterInstance = new aws.rds.ClusterInstance("default", {
 *     identifier: "aurora-instance-demo",
 *     clusterIdentifier: _default.clusterIdentifier,
 *     engine: _default.engine.apply((x) => aws.rds.EngineType[x]),
 *     instanceClass: aws.rds.InstanceType.R6G_Large,
 * });
 * const defaultKey = new aws.kms.Key("default", {description: "AWS KMS Key to encrypt Database Activity Stream"});
 * const defaultClusterActivityStream = new aws.rds.ClusterActivityStream("default", {
 *     resourceArn: _default.arn,
 *     mode: "async",
 *     kmsKeyId: defaultKey.keyId,
 * }, {
 *     dependsOn: [defaultClusterInstance],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import RDS Aurora Cluster Database Activity Streams using the `resource_arn`. For example:
 *
 * ```sh
 * $ pulumi import aws:rds/clusterActivityStream:ClusterActivityStream default arn:aws:rds:us-west-2:123456789012:cluster:aurora-cluster-demo
 * ```
 */
export declare class ClusterActivityStream extends pulumi.CustomResource {
    /**
     * Get an existing ClusterActivityStream resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ClusterActivityStreamState, opts?: pulumi.CustomResourceOptions): ClusterActivityStream;
    /**
     * Returns true if the given object is an instance of ClusterActivityStream.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ClusterActivityStream;
    /**
     * Specifies whether the database activity stream includes engine-native audit fields. This option only applies to an Oracle DB instance. By default, no engine-native audit fields are included. Defaults `false`.
     *
     * For more detailed documentation about each argument, refer to
     * the [AWS official documentation](https://docs.aws.amazon.com/cli/latest/reference/rds/start-activity-stream.html).
     */
    readonly engineNativeAuditFieldsIncluded: pulumi.Output<boolean | undefined>;
    /**
     * The name of the Amazon Kinesis data stream to be used for the database activity stream.
     */
    readonly kinesisStreamName: pulumi.Output<string>;
    /**
     * The AWS KMS key identifier for encrypting messages in the database activity stream. The AWS KMS key identifier is the key ARN, key ID, alias ARN, or alias name for the KMS key.
     */
    readonly kmsKeyId: pulumi.Output<string>;
    /**
     * Specifies the mode of the database activity stream. Database events such as a change or access generate an activity stream event. The database session can handle these events either synchronously or asynchronously. One of: `sync`, `async`.
     */
    readonly mode: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The Amazon Resource Name (ARN) of the DB cluster.
     */
    readonly resourceArn: pulumi.Output<string>;
    /**
     * Create a ClusterActivityStream resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ClusterActivityStreamArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ClusterActivityStream resources.
 */
export interface ClusterActivityStreamState {
    /**
     * Specifies whether the database activity stream includes engine-native audit fields. This option only applies to an Oracle DB instance. By default, no engine-native audit fields are included. Defaults `false`.
     *
     * For more detailed documentation about each argument, refer to
     * the [AWS official documentation](https://docs.aws.amazon.com/cli/latest/reference/rds/start-activity-stream.html).
     */
    engineNativeAuditFieldsIncluded?: pulumi.Input<boolean>;
    /**
     * The name of the Amazon Kinesis data stream to be used for the database activity stream.
     */
    kinesisStreamName?: pulumi.Input<string>;
    /**
     * The AWS KMS key identifier for encrypting messages in the database activity stream. The AWS KMS key identifier is the key ARN, key ID, alias ARN, or alias name for the KMS key.
     */
    kmsKeyId?: pulumi.Input<string>;
    /**
     * Specifies the mode of the database activity stream. Database events such as a change or access generate an activity stream event. The database session can handle these events either synchronously or asynchronously. One of: `sync`, `async`.
     */
    mode?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) of the DB cluster.
     */
    resourceArn?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a ClusterActivityStream resource.
 */
export interface ClusterActivityStreamArgs {
    /**
     * Specifies whether the database activity stream includes engine-native audit fields. This option only applies to an Oracle DB instance. By default, no engine-native audit fields are included. Defaults `false`.
     *
     * For more detailed documentation about each argument, refer to
     * the [AWS official documentation](https://docs.aws.amazon.com/cli/latest/reference/rds/start-activity-stream.html).
     */
    engineNativeAuditFieldsIncluded?: pulumi.Input<boolean>;
    /**
     * The AWS KMS key identifier for encrypting messages in the database activity stream. The AWS KMS key identifier is the key ARN, key ID, alias ARN, or alias name for the KMS key.
     */
    kmsKeyId: pulumi.Input<string>;
    /**
     * Specifies the mode of the database activity stream. Database events such as a change or access generate an activity stream event. The database session can handle these events either synchronously or asynchronously. One of: `sync`, `async`.
     */
    mode: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) of the DB cluster.
     */
    resourceArn: pulumi.Input<string>;
}
