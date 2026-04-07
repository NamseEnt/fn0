import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a Kinesis Stream resource. Amazon Kinesis is a managed service that
 * scales elastically for real-time processing of streaming big data.
 *
 * For more details, see the [Amazon Kinesis Documentation](https://aws.amazon.com/documentation/kinesis/).
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const testStream = new aws.kinesis.Stream("test_stream", {
 *     name: "kinesis-test",
 *     shardCount: 1,
 *     retentionPeriod: 48,
 *     shardLevelMetrics: [
 *         "IncomingBytes",
 *         "OutgoingBytes",
 *     ],
 *     streamModeDetails: {
 *         streamMode: "PROVISIONED",
 *     },
 *     tags: {
 *         Environment: "test",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Kinesis Streams using the `name`. For example:
 *
 * ```sh
 * $ pulumi import aws:kinesis/stream:Stream test_stream pulumi-kinesis-test
 * ```
 */
export declare class Stream extends pulumi.CustomResource {
    /**
     * Get an existing Stream resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: StreamState, opts?: pulumi.CustomResourceOptions): Stream;
    /**
     * Returns true if the given object is an instance of Stream.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Stream;
    /**
     * The Amazon Resource Name (ARN) specifying the Stream (same as `id`)
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The encryption type to use. The only acceptable values are `NONE` or `KMS`. The default value is `NONE`.
     */
    readonly encryptionType: pulumi.Output<string | undefined>;
    /**
     * A boolean that indicates all registered consumers should be deregistered from the stream so that the stream can be destroyed without error. The default value is `false`.
     */
    readonly enforceConsumerDeletion: pulumi.Output<boolean | undefined>;
    /**
     * The GUID for the customer-managed KMS key to use for encryption. You can also use a Kinesis-owned master key by specifying the alias `alias/aws/kinesis`.
     */
    readonly kmsKeyId: pulumi.Output<string | undefined>;
    /**
     * The maximum size for a single data record in KiB. The minimum value is 1024. The maximum value is 10240.
     */
    readonly maxRecordSizeInKib: pulumi.Output<number>;
    /**
     * A name to identify the stream. This is unique to the AWS account and region the Stream is created in.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Length of time data records are accessible after they are added to the stream. The maximum value of a stream's retention period is 8760 hours. Minimum value is 24. Default is 24.
     */
    readonly retentionPeriod: pulumi.Output<number | undefined>;
    /**
     * The number of shards that the stream will use. If the `streamMode` is `PROVISIONED`, this field is required.
     * Amazon has guidelines for specifying the Stream size that should be referenced when creating a Kinesis stream. See [Amazon Kinesis Streams](https://docs.aws.amazon.com/kinesis/latest/dev/amazon-kinesis-streams.html) for more.
     */
    readonly shardCount: pulumi.Output<number | undefined>;
    /**
     * A list of shard-level CloudWatch metrics which can be enabled for the stream. See [Monitoring with CloudWatch](https://docs.aws.amazon.com/streams/latest/dev/monitoring-with-cloudwatch.html) for more. Note that the value ALL should not be used; instead you should provide an explicit list of metrics you wish to enable.
     */
    readonly shardLevelMetrics: pulumi.Output<string[] | undefined>;
    /**
     * Indicates the [capacity mode](https://docs.aws.amazon.com/streams/latest/dev/how-do-i-size-a-stream.html) of the data stream. Detailed below.
     */
    readonly streamModeDetails: pulumi.Output<outputs.kinesis.StreamStreamModeDetails>;
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
     * Create a Stream resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: StreamArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Stream resources.
 */
export interface StreamState {
    /**
     * The Amazon Resource Name (ARN) specifying the Stream (same as `id`)
     */
    arn?: pulumi.Input<string>;
    /**
     * The encryption type to use. The only acceptable values are `NONE` or `KMS`. The default value is `NONE`.
     */
    encryptionType?: pulumi.Input<string>;
    /**
     * A boolean that indicates all registered consumers should be deregistered from the stream so that the stream can be destroyed without error. The default value is `false`.
     */
    enforceConsumerDeletion?: pulumi.Input<boolean>;
    /**
     * The GUID for the customer-managed KMS key to use for encryption. You can also use a Kinesis-owned master key by specifying the alias `alias/aws/kinesis`.
     */
    kmsKeyId?: pulumi.Input<string>;
    /**
     * The maximum size for a single data record in KiB. The minimum value is 1024. The maximum value is 10240.
     */
    maxRecordSizeInKib?: pulumi.Input<number>;
    /**
     * A name to identify the stream. This is unique to the AWS account and region the Stream is created in.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Length of time data records are accessible after they are added to the stream. The maximum value of a stream's retention period is 8760 hours. Minimum value is 24. Default is 24.
     */
    retentionPeriod?: pulumi.Input<number>;
    /**
     * The number of shards that the stream will use. If the `streamMode` is `PROVISIONED`, this field is required.
     * Amazon has guidelines for specifying the Stream size that should be referenced when creating a Kinesis stream. See [Amazon Kinesis Streams](https://docs.aws.amazon.com/kinesis/latest/dev/amazon-kinesis-streams.html) for more.
     */
    shardCount?: pulumi.Input<number>;
    /**
     * A list of shard-level CloudWatch metrics which can be enabled for the stream. See [Monitoring with CloudWatch](https://docs.aws.amazon.com/streams/latest/dev/monitoring-with-cloudwatch.html) for more. Note that the value ALL should not be used; instead you should provide an explicit list of metrics you wish to enable.
     */
    shardLevelMetrics?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Indicates the [capacity mode](https://docs.aws.amazon.com/streams/latest/dev/how-do-i-size-a-stream.html) of the data stream. Detailed below.
     */
    streamModeDetails?: pulumi.Input<inputs.kinesis.StreamStreamModeDetails>;
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
}
/**
 * The set of arguments for constructing a Stream resource.
 */
export interface StreamArgs {
    /**
     * The Amazon Resource Name (ARN) specifying the Stream (same as `id`)
     */
    arn?: pulumi.Input<string>;
    /**
     * The encryption type to use. The only acceptable values are `NONE` or `KMS`. The default value is `NONE`.
     */
    encryptionType?: pulumi.Input<string>;
    /**
     * A boolean that indicates all registered consumers should be deregistered from the stream so that the stream can be destroyed without error. The default value is `false`.
     */
    enforceConsumerDeletion?: pulumi.Input<boolean>;
    /**
     * The GUID for the customer-managed KMS key to use for encryption. You can also use a Kinesis-owned master key by specifying the alias `alias/aws/kinesis`.
     */
    kmsKeyId?: pulumi.Input<string>;
    /**
     * The maximum size for a single data record in KiB. The minimum value is 1024. The maximum value is 10240.
     */
    maxRecordSizeInKib?: pulumi.Input<number>;
    /**
     * A name to identify the stream. This is unique to the AWS account and region the Stream is created in.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Length of time data records are accessible after they are added to the stream. The maximum value of a stream's retention period is 8760 hours. Minimum value is 24. Default is 24.
     */
    retentionPeriod?: pulumi.Input<number>;
    /**
     * The number of shards that the stream will use. If the `streamMode` is `PROVISIONED`, this field is required.
     * Amazon has guidelines for specifying the Stream size that should be referenced when creating a Kinesis stream. See [Amazon Kinesis Streams](https://docs.aws.amazon.com/kinesis/latest/dev/amazon-kinesis-streams.html) for more.
     */
    shardCount?: pulumi.Input<number>;
    /**
     * A list of shard-level CloudWatch metrics which can be enabled for the stream. See [Monitoring with CloudWatch](https://docs.aws.amazon.com/streams/latest/dev/monitoring-with-cloudwatch.html) for more. Note that the value ALL should not be used; instead you should provide an explicit list of metrics you wish to enable.
     */
    shardLevelMetrics?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Indicates the [capacity mode](https://docs.aws.amazon.com/streams/latest/dev/how-do-i-size-a-stream.html) of the data stream. Detailed below.
     */
    streamModeDetails?: pulumi.Input<inputs.kinesis.StreamStreamModeDetails>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
