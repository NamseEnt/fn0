import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages an AWS Lambda Event Source Mapping. Use this resource to connect Lambda functions to event sources like Kinesis, DynamoDB, SQS, Amazon MQ, and Managed Streaming for Apache Kafka (MSK).
 *
 * For information about Lambda and how to use it, see [What is AWS Lambda?](http://docs.aws.amazon.com/lambda/latest/dg/welcome.html). For information about event source mappings, see [CreateEventSourceMapping](http://docs.aws.amazon.com/lambda/latest/dg/API_CreateEventSourceMapping.html) in the API docs.
 *
 * ## Example Usage
 *
 * ### DynamoDB Stream
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.lambda.EventSourceMapping("example", {
 *     eventSourceArn: exampleAwsDynamodbTable.streamArn,
 *     functionName: exampleAwsLambdaFunction.arn,
 *     startingPosition: "LATEST",
 *     tags: {
 *         Name: "dynamodb-stream-mapping",
 *     },
 * });
 * ```
 *
 * ### Kinesis Stream
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.lambda.EventSourceMapping("example", {
 *     eventSourceArn: exampleAwsKinesisStream.arn,
 *     functionName: exampleAwsLambdaFunction.arn,
 *     startingPosition: "LATEST",
 *     batchSize: 100,
 *     maximumBatchingWindowInSeconds: 5,
 *     parallelizationFactor: 2,
 *     destinationConfig: {
 *         onFailure: {
 *             destinationArn: dlq.arn,
 *         },
 *     },
 * });
 * ```
 *
 * ### SQS Queue
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.lambda.EventSourceMapping("example", {
 *     eventSourceArn: exampleAwsSqsQueue.arn,
 *     functionName: exampleAwsLambdaFunction.arn,
 *     batchSize: 10,
 *     scalingConfig: {
 *         maximumConcurrency: 100,
 *     },
 * });
 * ```
 *
 * ### SQS with Event Filtering
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.lambda.EventSourceMapping("example", {
 *     eventSourceArn: exampleAwsSqsQueue.arn,
 *     functionName: exampleAwsLambdaFunction.arn,
 *     filterCriteria: {
 *         filters: [{
 *             pattern: JSON.stringify({
 *                 body: {
 *                     Temperature: [{
 *                         numeric: [
 *                             ">",
 *                             0,
 *                             "<=",
 *                             100,
 *                         ],
 *                     }],
 *                     Location: ["New York"],
 *                 },
 *             }),
 *         }],
 *     },
 * });
 * ```
 *
 * ### Amazon MSK
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.lambda.EventSourceMapping("example", {
 *     eventSourceArn: exampleAwsMskCluster.arn,
 *     functionName: exampleAwsLambdaFunction.arn,
 *     topics: [
 *         "orders",
 *         "inventory",
 *     ],
 *     startingPosition: "TRIM_HORIZON",
 *     batchSize: 100,
 *     amazonManagedKafkaEventSourceConfig: {
 *         consumerGroupId: "lambda-consumer-group",
 *     },
 * });
 * ```
 *
 * ### Self-Managed Apache Kafka
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.lambda.EventSourceMapping("example", {
 *     functionName: exampleAwsLambdaFunction.arn,
 *     topics: ["orders"],
 *     startingPosition: "TRIM_HORIZON",
 *     selfManagedEventSource: {
 *         endpoints: {
 *             KAFKA_BOOTSTRAP_SERVERS: "kafka1.example.com:9092,kafka2.example.com:9092",
 *         },
 *     },
 *     selfManagedKafkaEventSourceConfig: {
 *         consumerGroupId: "lambda-consumer-group",
 *     },
 *     sourceAccessConfigurations: [
 *         {
 *             type: "VPC_SUBNET",
 *             uri: `subnet:${example1.id}`,
 *         },
 *         {
 *             type: "VPC_SUBNET",
 *             uri: `subnet:${example2.id}`,
 *         },
 *         {
 *             type: "VPC_SECURITY_GROUP",
 *             uri: `security_group:${exampleAwsSecurityGroup.id}`,
 *         },
 *     ],
 *     provisionedPollerConfig: {
 *         maximumPollers: 100,
 *         minimumPollers: 10,
 *     },
 * });
 * ```
 *
 * ### Amazon MQ (ActiveMQ)
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.lambda.EventSourceMapping("example", {
 *     eventSourceArn: exampleAwsMqBroker.arn,
 *     functionName: exampleAwsLambdaFunction.arn,
 *     queues: "orders",
 *     batchSize: 10,
 *     sourceAccessConfigurations: [{
 *         type: "BASIC_AUTH",
 *         uri: exampleAwsSecretsmanagerSecretVersion.arn,
 *     }],
 * });
 * ```
 *
 * ### Amazon MQ (RabbitMQ)
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.lambda.EventSourceMapping("example", {
 *     eventSourceArn: exampleAwsMqBroker.arn,
 *     functionName: exampleAwsLambdaFunction.arn,
 *     queues: "orders",
 *     batchSize: 1,
 *     sourceAccessConfigurations: [
 *         {
 *             type: "VIRTUAL_HOST",
 *             uri: "/production",
 *         },
 *         {
 *             type: "BASIC_AUTH",
 *             uri: exampleAwsSecretsmanagerSecretVersion.arn,
 *         },
 *     ],
 * });
 * ```
 *
 * ### DocumentDB Change Stream
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.lambda.EventSourceMapping("example", {
 *     eventSourceArn: exampleAwsDocdbCluster.arn,
 *     functionName: exampleAwsLambdaFunction.arn,
 *     startingPosition: "LATEST",
 *     documentDbEventSourceConfig: {
 *         databaseName: "orders",
 *         collectionName: "transactions",
 *         fullDocument: "UpdateLookup",
 *     },
 *     sourceAccessConfigurations: [{
 *         type: "BASIC_AUTH",
 *         uri: exampleAwsSecretsmanagerSecretVersion.arn,
 *     }],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Lambda event source mappings using the `UUID` (event source mapping identifier). For example:
 *
 * ```sh
 * $ pulumi import aws:lambda/eventSourceMapping:EventSourceMapping example 12345kxodurf3443
 * ```
 */
export declare class EventSourceMapping extends pulumi.CustomResource {
    /**
     * Get an existing EventSourceMapping resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: EventSourceMappingState, opts?: pulumi.CustomResourceOptions): EventSourceMapping;
    /**
     * Returns true if the given object is an instance of EventSourceMapping.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is EventSourceMapping;
    /**
     * Additional configuration block for Amazon Managed Kafka sources. Incompatible with `selfManagedEventSource` and `selfManagedKafkaEventSourceConfig`. See below.
     */
    readonly amazonManagedKafkaEventSourceConfig: pulumi.Output<outputs.lambda.EventSourceMappingAmazonManagedKafkaEventSourceConfig>;
    /**
     * Event source mapping ARN.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Largest number of records that Lambda will retrieve from your event source at the time of invocation. Defaults to `100` for DynamoDB, Kinesis, MQ and MSK, `10` for SQS.
     */
    readonly batchSize: pulumi.Output<number | undefined>;
    /**
     * Whether to split the batch in two and retry if the function returns an error. Only available for stream sources (DynamoDB and Kinesis). Defaults to `false`.
     */
    readonly bisectBatchOnFunctionError: pulumi.Output<boolean | undefined>;
    /**
     * Amazon SQS queue, Amazon SNS topic or Amazon S3 bucket (only available for Kafka sources) destination for failed records. Only available for stream sources (DynamoDB and Kinesis) and Kafka sources (Amazon MSK and Self-managed Apache Kafka). See below.
     */
    readonly destinationConfig: pulumi.Output<outputs.lambda.EventSourceMappingDestinationConfig | undefined>;
    /**
     * Configuration settings for a DocumentDB event source. See below.
     */
    readonly documentDbEventSourceConfig: pulumi.Output<outputs.lambda.EventSourceMappingDocumentDbEventSourceConfig | undefined>;
    /**
     * Whether the mapping is enabled. Defaults to `true`.
     */
    readonly enabled: pulumi.Output<boolean | undefined>;
    /**
     * Event source ARN - required for Kinesis stream, DynamoDB stream, SQS queue, MQ broker, MSK cluster or DocumentDB change stream. Incompatible with Self Managed Kafka source.
     */
    readonly eventSourceArn: pulumi.Output<string | undefined>;
    /**
     * Criteria to use for [event filtering](https://docs.aws.amazon.com/lambda/latest/dg/invocation-eventfiltering.html) Kinesis stream, DynamoDB stream, SQS queue event sources. See below.
     */
    readonly filterCriteria: pulumi.Output<outputs.lambda.EventSourceMappingFilterCriteria | undefined>;
    /**
     * ARN of the Lambda function the event source mapping is sending events to. (Note: this is a computed value that differs from `functionName` above.)
     */
    readonly functionArn: pulumi.Output<string>;
    /**
     * Name or ARN of the Lambda function that will be subscribing to events.
     *
     * The following arguments are optional:
     */
    readonly functionName: pulumi.Output<string>;
    /**
     * List of current response type enums applied to the event source mapping for [AWS Lambda checkpointing](https://docs.aws.amazon.com/lambda/latest/dg/with-ddb.html#services-ddb-batchfailurereporting). Only available for SQS and stream sources (DynamoDB and Kinesis). Valid values: `ReportBatchItemFailures`.
     */
    readonly functionResponseTypes: pulumi.Output<string[] | undefined>;
    /**
     * ARN of the Key Management Service (KMS) customer managed key that Lambda uses to encrypt your function's filter criteria.
     */
    readonly kmsKeyArn: pulumi.Output<string | undefined>;
    /**
     * Date this resource was last modified.
     */
    readonly lastModified: pulumi.Output<string>;
    /**
     * Result of the last AWS Lambda invocation of your Lambda function.
     */
    readonly lastProcessingResult: pulumi.Output<string>;
    /**
     * Maximum amount of time to gather records before invoking the function, in seconds (between 0 and 300). Records will continue to buffer until either `maximumBatchingWindowInSeconds` expires or `batchSize` has been met. For streaming event sources, defaults to as soon as records are available in the stream. Only available for stream sources (DynamoDB and Kinesis) and SQS standard queues.
     */
    readonly maximumBatchingWindowInSeconds: pulumi.Output<number | undefined>;
    /**
     * Maximum age of a record that Lambda sends to a function for processing. Only available for stream sources (DynamoDB and Kinesis). Must be either -1 (forever, and the default value) or between 60 and 604800 (inclusive).
     */
    readonly maximumRecordAgeInSeconds: pulumi.Output<number>;
    /**
     * Maximum number of times to retry when the function returns an error. Only available for stream sources (DynamoDB and Kinesis). Minimum and default of -1 (forever), maximum of 10000.
     */
    readonly maximumRetryAttempts: pulumi.Output<number>;
    /**
     * CloudWatch metrics configuration of the event source. Only available for stream sources (DynamoDB and Kinesis) and SQS queues. See below.
     */
    readonly metricsConfig: pulumi.Output<outputs.lambda.EventSourceMappingMetricsConfig | undefined>;
    /**
     * Number of batches to process from each shard concurrently. Only available for stream sources (DynamoDB and Kinesis). Minimum and default of 1, maximum of 10.
     */
    readonly parallelizationFactor: pulumi.Output<number>;
    /**
     * Event poller configuration for the event source. Only valid for Amazon MSK or self-managed Apache Kafka sources. See below.
     */
    readonly provisionedPollerConfig: pulumi.Output<outputs.lambda.EventSourceMappingProvisionedPollerConfig | undefined>;
    /**
     * Name of the Amazon MQ broker destination queue to consume. Only available for MQ sources. The list must contain exactly one queue name.
     */
    readonly queues: pulumi.Output<string | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Scaling configuration of the event source. Only available for SQS queues. See below.
     */
    readonly scalingConfig: pulumi.Output<outputs.lambda.EventSourceMappingScalingConfig | undefined>;
    /**
     * For Self Managed Kafka sources, the location of the self managed cluster. If set, configuration must also include `sourceAccessConfiguration`. See below.
     */
    readonly selfManagedEventSource: pulumi.Output<outputs.lambda.EventSourceMappingSelfManagedEventSource | undefined>;
    /**
     * Additional configuration block for Self Managed Kafka sources. Incompatible with `eventSourceArn` and `amazonManagedKafkaEventSourceConfig`. See below.
     */
    readonly selfManagedKafkaEventSourceConfig: pulumi.Output<outputs.lambda.EventSourceMappingSelfManagedKafkaEventSourceConfig>;
    /**
     * For Self Managed Kafka sources, the access configuration for the source. If set, configuration must also include `selfManagedEventSource`. See below.
     */
    readonly sourceAccessConfigurations: pulumi.Output<outputs.lambda.EventSourceMappingSourceAccessConfiguration[] | undefined>;
    /**
     * Position in the stream where AWS Lambda should start reading. Must be one of `AT_TIMESTAMP` (Kinesis only), `LATEST` or `TRIM_HORIZON` if getting events from Kinesis, DynamoDB, MSK or Self Managed Apache Kafka. Must not be provided if getting events from SQS. More information about these positions can be found in the [AWS DynamoDB Streams API Reference](https://docs.aws.amazon.com/amazondynamodb/latest/APIReference/API_streams_GetShardIterator.html) and [AWS Kinesis API Reference](https://docs.aws.amazon.com/kinesis/latest/APIReference/API_GetShardIterator.html#Kinesis-GetShardIterator-request-ShardIteratorType).
     */
    readonly startingPosition: pulumi.Output<string | undefined>;
    /**
     * Timestamp in [RFC3339 format](https://tools.ietf.org/html/rfc3339#section-5.8) of the data record which to start reading when using `startingPosition` set to `AT_TIMESTAMP`. If a record with this exact timestamp does not exist, the next later record is chosen. If the timestamp is older than the current trim horizon, the oldest available record is chosen.
     */
    readonly startingPositionTimestamp: pulumi.Output<string | undefined>;
    /**
     * State of the event source mapping.
     */
    readonly state: pulumi.Output<string>;
    /**
     * Reason the event source mapping is in its current state.
     */
    readonly stateTransitionReason: pulumi.Output<string>;
    /**
     * Map of tags to assign to the object. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Name of the Kafka topics. Only available for MSK sources. A single topic name must be specified.
     */
    readonly topics: pulumi.Output<string[] | undefined>;
    /**
     * Duration in seconds of a processing window for [AWS Lambda streaming analytics](https://docs.aws.amazon.com/lambda/latest/dg/with-kinesis.html#services-kinesis-windows). The range is between 1 second up to 900 seconds. Only available for stream sources (DynamoDB and Kinesis).
     */
    readonly tumblingWindowInSeconds: pulumi.Output<number | undefined>;
    /**
     * UUID of the created event source mapping.
     */
    readonly uuid: pulumi.Output<string>;
    /**
     * Create a EventSourceMapping resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: EventSourceMappingArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering EventSourceMapping resources.
 */
export interface EventSourceMappingState {
    /**
     * Additional configuration block for Amazon Managed Kafka sources. Incompatible with `selfManagedEventSource` and `selfManagedKafkaEventSourceConfig`. See below.
     */
    amazonManagedKafkaEventSourceConfig?: pulumi.Input<inputs.lambda.EventSourceMappingAmazonManagedKafkaEventSourceConfig>;
    /**
     * Event source mapping ARN.
     */
    arn?: pulumi.Input<string>;
    /**
     * Largest number of records that Lambda will retrieve from your event source at the time of invocation. Defaults to `100` for DynamoDB, Kinesis, MQ and MSK, `10` for SQS.
     */
    batchSize?: pulumi.Input<number>;
    /**
     * Whether to split the batch in two and retry if the function returns an error. Only available for stream sources (DynamoDB and Kinesis). Defaults to `false`.
     */
    bisectBatchOnFunctionError?: pulumi.Input<boolean>;
    /**
     * Amazon SQS queue, Amazon SNS topic or Amazon S3 bucket (only available for Kafka sources) destination for failed records. Only available for stream sources (DynamoDB and Kinesis) and Kafka sources (Amazon MSK and Self-managed Apache Kafka). See below.
     */
    destinationConfig?: pulumi.Input<inputs.lambda.EventSourceMappingDestinationConfig>;
    /**
     * Configuration settings for a DocumentDB event source. See below.
     */
    documentDbEventSourceConfig?: pulumi.Input<inputs.lambda.EventSourceMappingDocumentDbEventSourceConfig>;
    /**
     * Whether the mapping is enabled. Defaults to `true`.
     */
    enabled?: pulumi.Input<boolean>;
    /**
     * Event source ARN - required for Kinesis stream, DynamoDB stream, SQS queue, MQ broker, MSK cluster or DocumentDB change stream. Incompatible with Self Managed Kafka source.
     */
    eventSourceArn?: pulumi.Input<string>;
    /**
     * Criteria to use for [event filtering](https://docs.aws.amazon.com/lambda/latest/dg/invocation-eventfiltering.html) Kinesis stream, DynamoDB stream, SQS queue event sources. See below.
     */
    filterCriteria?: pulumi.Input<inputs.lambda.EventSourceMappingFilterCriteria>;
    /**
     * ARN of the Lambda function the event source mapping is sending events to. (Note: this is a computed value that differs from `functionName` above.)
     */
    functionArn?: pulumi.Input<string>;
    /**
     * Name or ARN of the Lambda function that will be subscribing to events.
     *
     * The following arguments are optional:
     */
    functionName?: pulumi.Input<string>;
    /**
     * List of current response type enums applied to the event source mapping for [AWS Lambda checkpointing](https://docs.aws.amazon.com/lambda/latest/dg/with-ddb.html#services-ddb-batchfailurereporting). Only available for SQS and stream sources (DynamoDB and Kinesis). Valid values: `ReportBatchItemFailures`.
     */
    functionResponseTypes?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * ARN of the Key Management Service (KMS) customer managed key that Lambda uses to encrypt your function's filter criteria.
     */
    kmsKeyArn?: pulumi.Input<string>;
    /**
     * Date this resource was last modified.
     */
    lastModified?: pulumi.Input<string>;
    /**
     * Result of the last AWS Lambda invocation of your Lambda function.
     */
    lastProcessingResult?: pulumi.Input<string>;
    /**
     * Maximum amount of time to gather records before invoking the function, in seconds (between 0 and 300). Records will continue to buffer until either `maximumBatchingWindowInSeconds` expires or `batchSize` has been met. For streaming event sources, defaults to as soon as records are available in the stream. Only available for stream sources (DynamoDB and Kinesis) and SQS standard queues.
     */
    maximumBatchingWindowInSeconds?: pulumi.Input<number>;
    /**
     * Maximum age of a record that Lambda sends to a function for processing. Only available for stream sources (DynamoDB and Kinesis). Must be either -1 (forever, and the default value) or between 60 and 604800 (inclusive).
     */
    maximumRecordAgeInSeconds?: pulumi.Input<number>;
    /**
     * Maximum number of times to retry when the function returns an error. Only available for stream sources (DynamoDB and Kinesis). Minimum and default of -1 (forever), maximum of 10000.
     */
    maximumRetryAttempts?: pulumi.Input<number>;
    /**
     * CloudWatch metrics configuration of the event source. Only available for stream sources (DynamoDB and Kinesis) and SQS queues. See below.
     */
    metricsConfig?: pulumi.Input<inputs.lambda.EventSourceMappingMetricsConfig>;
    /**
     * Number of batches to process from each shard concurrently. Only available for stream sources (DynamoDB and Kinesis). Minimum and default of 1, maximum of 10.
     */
    parallelizationFactor?: pulumi.Input<number>;
    /**
     * Event poller configuration for the event source. Only valid for Amazon MSK or self-managed Apache Kafka sources. See below.
     */
    provisionedPollerConfig?: pulumi.Input<inputs.lambda.EventSourceMappingProvisionedPollerConfig>;
    /**
     * Name of the Amazon MQ broker destination queue to consume. Only available for MQ sources. The list must contain exactly one queue name.
     */
    queues?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Scaling configuration of the event source. Only available for SQS queues. See below.
     */
    scalingConfig?: pulumi.Input<inputs.lambda.EventSourceMappingScalingConfig>;
    /**
     * For Self Managed Kafka sources, the location of the self managed cluster. If set, configuration must also include `sourceAccessConfiguration`. See below.
     */
    selfManagedEventSource?: pulumi.Input<inputs.lambda.EventSourceMappingSelfManagedEventSource>;
    /**
     * Additional configuration block for Self Managed Kafka sources. Incompatible with `eventSourceArn` and `amazonManagedKafkaEventSourceConfig`. See below.
     */
    selfManagedKafkaEventSourceConfig?: pulumi.Input<inputs.lambda.EventSourceMappingSelfManagedKafkaEventSourceConfig>;
    /**
     * For Self Managed Kafka sources, the access configuration for the source. If set, configuration must also include `selfManagedEventSource`. See below.
     */
    sourceAccessConfigurations?: pulumi.Input<pulumi.Input<inputs.lambda.EventSourceMappingSourceAccessConfiguration>[]>;
    /**
     * Position in the stream where AWS Lambda should start reading. Must be one of `AT_TIMESTAMP` (Kinesis only), `LATEST` or `TRIM_HORIZON` if getting events from Kinesis, DynamoDB, MSK or Self Managed Apache Kafka. Must not be provided if getting events from SQS. More information about these positions can be found in the [AWS DynamoDB Streams API Reference](https://docs.aws.amazon.com/amazondynamodb/latest/APIReference/API_streams_GetShardIterator.html) and [AWS Kinesis API Reference](https://docs.aws.amazon.com/kinesis/latest/APIReference/API_GetShardIterator.html#Kinesis-GetShardIterator-request-ShardIteratorType).
     */
    startingPosition?: pulumi.Input<string>;
    /**
     * Timestamp in [RFC3339 format](https://tools.ietf.org/html/rfc3339#section-5.8) of the data record which to start reading when using `startingPosition` set to `AT_TIMESTAMP`. If a record with this exact timestamp does not exist, the next later record is chosen. If the timestamp is older than the current trim horizon, the oldest available record is chosen.
     */
    startingPositionTimestamp?: pulumi.Input<string>;
    /**
     * State of the event source mapping.
     */
    state?: pulumi.Input<string>;
    /**
     * Reason the event source mapping is in its current state.
     */
    stateTransitionReason?: pulumi.Input<string>;
    /**
     * Map of tags to assign to the object. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Name of the Kafka topics. Only available for MSK sources. A single topic name must be specified.
     */
    topics?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Duration in seconds of a processing window for [AWS Lambda streaming analytics](https://docs.aws.amazon.com/lambda/latest/dg/with-kinesis.html#services-kinesis-windows). The range is between 1 second up to 900 seconds. Only available for stream sources (DynamoDB and Kinesis).
     */
    tumblingWindowInSeconds?: pulumi.Input<number>;
    /**
     * UUID of the created event source mapping.
     */
    uuid?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a EventSourceMapping resource.
 */
export interface EventSourceMappingArgs {
    /**
     * Additional configuration block for Amazon Managed Kafka sources. Incompatible with `selfManagedEventSource` and `selfManagedKafkaEventSourceConfig`. See below.
     */
    amazonManagedKafkaEventSourceConfig?: pulumi.Input<inputs.lambda.EventSourceMappingAmazonManagedKafkaEventSourceConfig>;
    /**
     * Largest number of records that Lambda will retrieve from your event source at the time of invocation. Defaults to `100` for DynamoDB, Kinesis, MQ and MSK, `10` for SQS.
     */
    batchSize?: pulumi.Input<number>;
    /**
     * Whether to split the batch in two and retry if the function returns an error. Only available for stream sources (DynamoDB and Kinesis). Defaults to `false`.
     */
    bisectBatchOnFunctionError?: pulumi.Input<boolean>;
    /**
     * Amazon SQS queue, Amazon SNS topic or Amazon S3 bucket (only available for Kafka sources) destination for failed records. Only available for stream sources (DynamoDB and Kinesis) and Kafka sources (Amazon MSK and Self-managed Apache Kafka). See below.
     */
    destinationConfig?: pulumi.Input<inputs.lambda.EventSourceMappingDestinationConfig>;
    /**
     * Configuration settings for a DocumentDB event source. See below.
     */
    documentDbEventSourceConfig?: pulumi.Input<inputs.lambda.EventSourceMappingDocumentDbEventSourceConfig>;
    /**
     * Whether the mapping is enabled. Defaults to `true`.
     */
    enabled?: pulumi.Input<boolean>;
    /**
     * Event source ARN - required for Kinesis stream, DynamoDB stream, SQS queue, MQ broker, MSK cluster or DocumentDB change stream. Incompatible with Self Managed Kafka source.
     */
    eventSourceArn?: pulumi.Input<string>;
    /**
     * Criteria to use for [event filtering](https://docs.aws.amazon.com/lambda/latest/dg/invocation-eventfiltering.html) Kinesis stream, DynamoDB stream, SQS queue event sources. See below.
     */
    filterCriteria?: pulumi.Input<inputs.lambda.EventSourceMappingFilterCriteria>;
    /**
     * Name or ARN of the Lambda function that will be subscribing to events.
     *
     * The following arguments are optional:
     */
    functionName: pulumi.Input<string>;
    /**
     * List of current response type enums applied to the event source mapping for [AWS Lambda checkpointing](https://docs.aws.amazon.com/lambda/latest/dg/with-ddb.html#services-ddb-batchfailurereporting). Only available for SQS and stream sources (DynamoDB and Kinesis). Valid values: `ReportBatchItemFailures`.
     */
    functionResponseTypes?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * ARN of the Key Management Service (KMS) customer managed key that Lambda uses to encrypt your function's filter criteria.
     */
    kmsKeyArn?: pulumi.Input<string>;
    /**
     * Maximum amount of time to gather records before invoking the function, in seconds (between 0 and 300). Records will continue to buffer until either `maximumBatchingWindowInSeconds` expires or `batchSize` has been met. For streaming event sources, defaults to as soon as records are available in the stream. Only available for stream sources (DynamoDB and Kinesis) and SQS standard queues.
     */
    maximumBatchingWindowInSeconds?: pulumi.Input<number>;
    /**
     * Maximum age of a record that Lambda sends to a function for processing. Only available for stream sources (DynamoDB and Kinesis). Must be either -1 (forever, and the default value) or between 60 and 604800 (inclusive).
     */
    maximumRecordAgeInSeconds?: pulumi.Input<number>;
    /**
     * Maximum number of times to retry when the function returns an error. Only available for stream sources (DynamoDB and Kinesis). Minimum and default of -1 (forever), maximum of 10000.
     */
    maximumRetryAttempts?: pulumi.Input<number>;
    /**
     * CloudWatch metrics configuration of the event source. Only available for stream sources (DynamoDB and Kinesis) and SQS queues. See below.
     */
    metricsConfig?: pulumi.Input<inputs.lambda.EventSourceMappingMetricsConfig>;
    /**
     * Number of batches to process from each shard concurrently. Only available for stream sources (DynamoDB and Kinesis). Minimum and default of 1, maximum of 10.
     */
    parallelizationFactor?: pulumi.Input<number>;
    /**
     * Event poller configuration for the event source. Only valid for Amazon MSK or self-managed Apache Kafka sources. See below.
     */
    provisionedPollerConfig?: pulumi.Input<inputs.lambda.EventSourceMappingProvisionedPollerConfig>;
    /**
     * Name of the Amazon MQ broker destination queue to consume. Only available for MQ sources. The list must contain exactly one queue name.
     */
    queues?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Scaling configuration of the event source. Only available for SQS queues. See below.
     */
    scalingConfig?: pulumi.Input<inputs.lambda.EventSourceMappingScalingConfig>;
    /**
     * For Self Managed Kafka sources, the location of the self managed cluster. If set, configuration must also include `sourceAccessConfiguration`. See below.
     */
    selfManagedEventSource?: pulumi.Input<inputs.lambda.EventSourceMappingSelfManagedEventSource>;
    /**
     * Additional configuration block for Self Managed Kafka sources. Incompatible with `eventSourceArn` and `amazonManagedKafkaEventSourceConfig`. See below.
     */
    selfManagedKafkaEventSourceConfig?: pulumi.Input<inputs.lambda.EventSourceMappingSelfManagedKafkaEventSourceConfig>;
    /**
     * For Self Managed Kafka sources, the access configuration for the source. If set, configuration must also include `selfManagedEventSource`. See below.
     */
    sourceAccessConfigurations?: pulumi.Input<pulumi.Input<inputs.lambda.EventSourceMappingSourceAccessConfiguration>[]>;
    /**
     * Position in the stream where AWS Lambda should start reading. Must be one of `AT_TIMESTAMP` (Kinesis only), `LATEST` or `TRIM_HORIZON` if getting events from Kinesis, DynamoDB, MSK or Self Managed Apache Kafka. Must not be provided if getting events from SQS. More information about these positions can be found in the [AWS DynamoDB Streams API Reference](https://docs.aws.amazon.com/amazondynamodb/latest/APIReference/API_streams_GetShardIterator.html) and [AWS Kinesis API Reference](https://docs.aws.amazon.com/kinesis/latest/APIReference/API_GetShardIterator.html#Kinesis-GetShardIterator-request-ShardIteratorType).
     */
    startingPosition?: pulumi.Input<string>;
    /**
     * Timestamp in [RFC3339 format](https://tools.ietf.org/html/rfc3339#section-5.8) of the data record which to start reading when using `startingPosition` set to `AT_TIMESTAMP`. If a record with this exact timestamp does not exist, the next later record is chosen. If the timestamp is older than the current trim horizon, the oldest available record is chosen.
     */
    startingPositionTimestamp?: pulumi.Input<string>;
    /**
     * Map of tags to assign to the object. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Name of the Kafka topics. Only available for MSK sources. A single topic name must be specified.
     */
    topics?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Duration in seconds of a processing window for [AWS Lambda streaming analytics](https://docs.aws.amazon.com/lambda/latest/dg/with-kinesis.html#services-kinesis-windows). The range is between 1 second up to 900 seconds. Only available for stream sources (DynamoDB and Kinesis).
     */
    tumblingWindowInSeconds?: pulumi.Input<number>;
}
