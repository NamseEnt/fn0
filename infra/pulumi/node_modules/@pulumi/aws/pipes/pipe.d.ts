import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS EventBridge Pipes Pipe.
 *
 * You can find out more about EventBridge Pipes in the [User Guide](https://docs.aws.amazon.com/eventbridge/latest/userguide/eb-pipes.html).
 *
 * EventBridge Pipes are very configurable, and may require IAM permissions to work correctly. More information on the configuration options and IAM permissions can be found in the [User Guide](https://docs.aws.amazon.com/eventbridge/latest/userguide/eb-pipes.html).
 *
 * > **Note:** EventBridge was formerly known as CloudWatch Events. The functionality is identical.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const main = aws.getCallerIdentity({});
 * const example = new aws.iam.Role("example", {assumeRolePolicy: JSON.stringify({
 *     Version: "2012-10-17",
 *     Statement: {
 *         Effect: "Allow",
 *         Action: "sts:AssumeRole",
 *         Principal: {
 *             Service: "pipes.amazonaws.com",
 *         },
 *         Condition: {
 *             StringEquals: {
 *                 "aws:SourceAccount": main.then(main => main.accountId),
 *             },
 *         },
 *     },
 * })});
 * const sourceQueue = new aws.sqs.Queue("source", {});
 * const source = new aws.iam.RolePolicy("source", {
 *     role: example.id,
 *     policy: pulumi.jsonStringify({
 *         Version: "2012-10-17",
 *         Statement: [{
 *             Effect: "Allow",
 *             Action: [
 *                 "sqs:DeleteMessage",
 *                 "sqs:GetQueueAttributes",
 *                 "sqs:ReceiveMessage",
 *             ],
 *             Resource: [sourceQueue.arn],
 *         }],
 *     }),
 * });
 * const targetQueue = new aws.sqs.Queue("target", {});
 * const target = new aws.iam.RolePolicy("target", {
 *     role: example.id,
 *     policy: pulumi.jsonStringify({
 *         Version: "2012-10-17",
 *         Statement: [{
 *             Effect: "Allow",
 *             Action: ["sqs:SendMessage"],
 *             Resource: [targetQueue.arn],
 *         }],
 *     }),
 * });
 * const examplePipe = new aws.pipes.Pipe("example", {
 *     name: "example-pipe",
 *     roleArn: example.arn,
 *     source: sourceQueue.arn,
 *     target: targetQueue.arn,
 * }, {
 *     dependsOn: [
 *         source,
 *         target,
 *     ],
 * });
 * ```
 *
 * ### Enrichment Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.pipes.Pipe("example", {
 *     name: "example-pipe",
 *     roleArn: exampleAwsIamRole.arn,
 *     source: source.arn,
 *     target: target.arn,
 *     enrichment: exampleAwsCloudwatchEventApiDestination.arn,
 *     enrichmentParameters: {
 *         httpParameters: {
 *             pathParameterValues: "example-path-param",
 *             headerParameters: {
 *                 "example-header": "example-value",
 *                 "second-example-header": "second-example-value",
 *             },
 *             queryStringParameters: {
 *                 "example-query-string": "example-value",
 *                 "second-example-query-string": "second-example-value",
 *             },
 *         },
 *     },
 * });
 * ```
 *
 * ### Filter Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.pipes.Pipe("example", {
 *     name: "example-pipe",
 *     roleArn: exampleAwsIamRole.arn,
 *     source: source.arn,
 *     target: target.arn,
 *     sourceParameters: {
 *         filterCriteria: {
 *             filters: [{
 *                 pattern: JSON.stringify({
 *                     source: ["event-source"],
 *                 }),
 *             }],
 *         },
 *     },
 * });
 * ```
 *
 * ### CloudWatch Logs Logging Configuration Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.cloudwatch.LogGroup("example", {name: "example-pipe-target"});
 * const examplePipe = new aws.pipes.Pipe("example", {
 *     name: "example-pipe",
 *     roleArn: exampleAwsIamRole.arn,
 *     source: sourceAwsSqsQueue.arn,
 *     target: targetAwsSqsQueue.arn,
 *     logConfiguration: {
 *         includeExecutionDatas: ["ALL"],
 *         level: "INFO",
 *         cloudwatchLogsLogDestination: {
 *             logGroupArn: targetAwsCloudwatchLogGroup.arn,
 *         },
 *     },
 * }, {
 *     dependsOn: [
 *         source,
 *         target,
 *     ],
 * });
 * ```
 *
 * ### SQS Source and Target Configuration Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.pipes.Pipe("example", {
 *     name: "example-pipe",
 *     roleArn: exampleAwsIamRole.arn,
 *     source: source.arn,
 *     target: target.arn,
 *     sourceParameters: {
 *         sqsQueueParameters: {
 *             batchSize: 1,
 *             maximumBatchingWindowInSeconds: 2,
 *         },
 *     },
 *     targetParameters: {
 *         sqsQueueParameters: {
 *             messageDeduplicationId: "example-dedupe",
 *             messageGroupId: "example-group",
 *         },
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import pipes using the `name`. For example:
 *
 * ```sh
 * $ pulumi import aws:pipes/pipe:Pipe example my-pipe
 * ```
 */
export declare class Pipe extends pulumi.CustomResource {
    /**
     * Get an existing Pipe resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: PipeState, opts?: pulumi.CustomResourceOptions): Pipe;
    /**
     * Returns true if the given object is an instance of Pipe.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Pipe;
    /**
     * ARN of this pipe.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * A description of the pipe. At most 512 characters.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * The state the pipe should be in. One of: `RUNNING`, `STOPPED`.
     */
    readonly desiredState: pulumi.Output<string | undefined>;
    /**
     * Enrichment resource of the pipe (typically an ARN). Read more about enrichment in the [User Guide](https://docs.aws.amazon.com/eventbridge/latest/userguide/eb-pipes.html#pipes-enrichment).
     */
    readonly enrichment: pulumi.Output<string | undefined>;
    /**
     * Parameters to configure enrichment for your pipe. Detailed below.
     */
    readonly enrichmentParameters: pulumi.Output<outputs.pipes.PipeEnrichmentParameters | undefined>;
    /**
     * Identifier of the AWS KMS customer managed key for EventBridge to use, if you choose to use a customer managed key to encrypt pipe data. The identifier can be the key Amazon Resource Name (ARN), KeyId, key alias, or key alias ARN. If not set, EventBridge uses an AWS owned key to encrypt pipe data.
     */
    readonly kmsKeyIdentifier: pulumi.Output<string | undefined>;
    /**
     * Logging configuration settings for the pipe. Detailed below.
     */
    readonly logConfiguration: pulumi.Output<outputs.pipes.PipeLogConfiguration | undefined>;
    /**
     * Name of the pipe. If omitted, the provider will assign a random, unique name. Conflicts with `namePrefix`.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Creates a unique name beginning with the specified prefix. Conflicts with `name`.
     */
    readonly namePrefix: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * ARN of the role that allows the pipe to send data to the target.
     */
    readonly roleArn: pulumi.Output<string>;
    /**
     * Source resource of the pipe. This field typically requires an ARN (Amazon Resource Name). However, when using a self-managed Kafka cluster, you should use a different format. Instead of an ARN, use 'smk://' followed by the bootstrap server's address.
     */
    readonly source: pulumi.Output<string>;
    /**
     * Parameters to configure a source for the pipe. Detailed below.
     */
    readonly sourceParameters: pulumi.Output<outputs.pipes.PipeSourceParameters>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Target resource of the pipe (typically an ARN).
     *
     * The following arguments are optional:
     */
    readonly target: pulumi.Output<string>;
    /**
     * Parameters to configure a target for your pipe. Detailed below.
     */
    readonly targetParameters: pulumi.Output<outputs.pipes.PipeTargetParameters | undefined>;
    /**
     * Create a Pipe resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: PipeArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Pipe resources.
 */
export interface PipeState {
    /**
     * ARN of this pipe.
     */
    arn?: pulumi.Input<string>;
    /**
     * A description of the pipe. At most 512 characters.
     */
    description?: pulumi.Input<string>;
    /**
     * The state the pipe should be in. One of: `RUNNING`, `STOPPED`.
     */
    desiredState?: pulumi.Input<string>;
    /**
     * Enrichment resource of the pipe (typically an ARN). Read more about enrichment in the [User Guide](https://docs.aws.amazon.com/eventbridge/latest/userguide/eb-pipes.html#pipes-enrichment).
     */
    enrichment?: pulumi.Input<string>;
    /**
     * Parameters to configure enrichment for your pipe. Detailed below.
     */
    enrichmentParameters?: pulumi.Input<inputs.pipes.PipeEnrichmentParameters>;
    /**
     * Identifier of the AWS KMS customer managed key for EventBridge to use, if you choose to use a customer managed key to encrypt pipe data. The identifier can be the key Amazon Resource Name (ARN), KeyId, key alias, or key alias ARN. If not set, EventBridge uses an AWS owned key to encrypt pipe data.
     */
    kmsKeyIdentifier?: pulumi.Input<string>;
    /**
     * Logging configuration settings for the pipe. Detailed below.
     */
    logConfiguration?: pulumi.Input<inputs.pipes.PipeLogConfiguration>;
    /**
     * Name of the pipe. If omitted, the provider will assign a random, unique name. Conflicts with `namePrefix`.
     */
    name?: pulumi.Input<string>;
    /**
     * Creates a unique name beginning with the specified prefix. Conflicts with `name`.
     */
    namePrefix?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ARN of the role that allows the pipe to send data to the target.
     */
    roleArn?: pulumi.Input<string>;
    /**
     * Source resource of the pipe. This field typically requires an ARN (Amazon Resource Name). However, when using a self-managed Kafka cluster, you should use a different format. Instead of an ARN, use 'smk://' followed by the bootstrap server's address.
     */
    source?: pulumi.Input<string>;
    /**
     * Parameters to configure a source for the pipe. Detailed below.
     */
    sourceParameters?: pulumi.Input<inputs.pipes.PipeSourceParameters>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Target resource of the pipe (typically an ARN).
     *
     * The following arguments are optional:
     */
    target?: pulumi.Input<string>;
    /**
     * Parameters to configure a target for your pipe. Detailed below.
     */
    targetParameters?: pulumi.Input<inputs.pipes.PipeTargetParameters>;
}
/**
 * The set of arguments for constructing a Pipe resource.
 */
export interface PipeArgs {
    /**
     * A description of the pipe. At most 512 characters.
     */
    description?: pulumi.Input<string>;
    /**
     * The state the pipe should be in. One of: `RUNNING`, `STOPPED`.
     */
    desiredState?: pulumi.Input<string>;
    /**
     * Enrichment resource of the pipe (typically an ARN). Read more about enrichment in the [User Guide](https://docs.aws.amazon.com/eventbridge/latest/userguide/eb-pipes.html#pipes-enrichment).
     */
    enrichment?: pulumi.Input<string>;
    /**
     * Parameters to configure enrichment for your pipe. Detailed below.
     */
    enrichmentParameters?: pulumi.Input<inputs.pipes.PipeEnrichmentParameters>;
    /**
     * Identifier of the AWS KMS customer managed key for EventBridge to use, if you choose to use a customer managed key to encrypt pipe data. The identifier can be the key Amazon Resource Name (ARN), KeyId, key alias, or key alias ARN. If not set, EventBridge uses an AWS owned key to encrypt pipe data.
     */
    kmsKeyIdentifier?: pulumi.Input<string>;
    /**
     * Logging configuration settings for the pipe. Detailed below.
     */
    logConfiguration?: pulumi.Input<inputs.pipes.PipeLogConfiguration>;
    /**
     * Name of the pipe. If omitted, the provider will assign a random, unique name. Conflicts with `namePrefix`.
     */
    name?: pulumi.Input<string>;
    /**
     * Creates a unique name beginning with the specified prefix. Conflicts with `name`.
     */
    namePrefix?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ARN of the role that allows the pipe to send data to the target.
     */
    roleArn: pulumi.Input<string>;
    /**
     * Source resource of the pipe. This field typically requires an ARN (Amazon Resource Name). However, when using a self-managed Kafka cluster, you should use a different format. Instead of an ARN, use 'smk://' followed by the bootstrap server's address.
     */
    source: pulumi.Input<string>;
    /**
     * Parameters to configure a source for the pipe. Detailed below.
     */
    sourceParameters?: pulumi.Input<inputs.pipes.PipeSourceParameters>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Target resource of the pipe (typically an ARN).
     *
     * The following arguments are optional:
     */
    target: pulumi.Input<string>;
    /**
     * Parameters to configure a target for your pipe. Detailed below.
     */
    targetParameters?: pulumi.Input<inputs.pipes.PipeTargetParameters>;
}
