import * as pulumi from "@pulumi/pulumi";
/**
 * Provides an SNS topic resource
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const userUpdates = new aws.sns.Topic("user_updates", {name: "user-updates-topic"});
 * ```
 *
 * ## Example with Delivery Policy
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const userUpdates = new aws.sns.Topic("user_updates", {
 *     name: "user-updates-topic",
 *     deliveryPolicy: `{
 *   \\"http\\": {
 *     \\"defaultHealthyRetryPolicy\\": {
 *       \\"minDelayTarget\\": 20,
 *       \\"maxDelayTarget\\": 20,
 *       \\"numRetries\\": 3,
 *       \\"numMaxDelayRetries\\": 0,
 *       \\"numNoDelayRetries\\": 0,
 *       \\"numMinDelayRetries\\": 0,
 *       \\"backoffFunction\\": \\"linear\\"
 *     },
 *     \\"disableSubscriptionOverrides\\": false,
 *     \\"defaultThrottlePolicy\\": {
 *       \\"maxReceivesPerSecond\\": 1
 *     }
 *   }
 * }
 * `,
 * });
 * ```
 *
 * ## Example with Server-side encryption (SSE)
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const userUpdates = new aws.sns.Topic("user_updates", {
 *     name: "user-updates-topic",
 *     kmsMasterKeyId: "alias/aws/sns",
 * });
 * ```
 *
 * ## Example with First-In-First-Out (FIFO)
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const userUpdates = new aws.sns.Topic("user_updates", {
 *     name: "user-updates-topic.fifo",
 *     fifoTopic: true,
 *     contentBasedDeduplication: true,
 * });
 * ```
 *
 * ## Message Delivery Status Arguments
 *
 * The `<endpoint>_success_feedback_role_arn` and `<endpoint>_failure_feedback_role_arn` arguments are used to give Amazon SNS write access to use CloudWatch Logs on your behalf. The `<endpoint>_success_feedback_sample_rate` argument is for specifying the sample rate percentage (0-100) of successfully delivered messages. After you configure the  `<endpoint>_failure_feedback_role_arn` argument, then all failed message deliveries generate CloudWatch Logs.
 *
 * ## Import
 *
 * ### Identity Schema
 *
 * #### Required
 *
 * - `arn` (String) Amazon Resource Name (ARN) of the SNS topic.
 *
 * Using `pulumi import`, import SNS Topics using the topic `arn`. For example:
 *
 * console
 *
 * % pulumi import aws_sns_topic.user_updates arn:aws:sns:us-west-2:123456789012:my-topic
 */
export declare class Topic extends pulumi.CustomResource {
    /**
     * Get an existing Topic resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: TopicState, opts?: pulumi.CustomResourceOptions): Topic;
    /**
     * Returns true if the given object is an instance of Topic.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Topic;
    /**
     * IAM role for failure feedback
     */
    readonly applicationFailureFeedbackRoleArn: pulumi.Output<string | undefined>;
    /**
     * The IAM role permitted to receive success feedback for this topic
     */
    readonly applicationSuccessFeedbackRoleArn: pulumi.Output<string | undefined>;
    /**
     * Percentage of success to sample
     */
    readonly applicationSuccessFeedbackSampleRate: pulumi.Output<number | undefined>;
    /**
     * The message archive policy for FIFO topics. More details in the [AWS documentation](https://docs.aws.amazon.com/sns/latest/dg/message-archiving-and-replay-topic-owner.html).
     */
    readonly archivePolicy: pulumi.Output<string | undefined>;
    /**
     * The ARN of the SNS topic, as a more obvious property (clone of id)
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The oldest timestamp at which a FIFO topic subscriber can start a replay.
     */
    readonly beginningArchiveTime: pulumi.Output<string>;
    /**
     * Enables content-based deduplication for FIFO topics. For more information, see the [related documentation](https://docs.aws.amazon.com/sns/latest/dg/fifo-message-dedup.html)
     */
    readonly contentBasedDeduplication: pulumi.Output<boolean | undefined>;
    /**
     * The SNS delivery policy. More details in the [AWS documentation](https://docs.aws.amazon.com/sns/latest/dg/DeliveryPolicies.html).
     */
    readonly deliveryPolicy: pulumi.Output<string | undefined>;
    /**
     * The display name for the topic
     */
    readonly displayName: pulumi.Output<string | undefined>;
    /**
     * Enables higher throughput for FIFO topics by adjusting the scope of deduplication. This attribute has two possible values, `Topic` and `MessageGroup`. For more information, see the [related documentation](https://docs.aws.amazon.com/sns/latest/dg/fifo-high-throughput.html#enable-high-throughput-on-fifo-topic).
     */
    readonly fifoThroughputScope: pulumi.Output<string>;
    /**
     * Boolean indicating whether or not to create a FIFO (first-in-first-out) topic. FIFO topics can't deliver messages to customer managed endpoints, such as email addresses, mobile apps, SMS, or HTTP(S) endpoints. These endpoint types aren't guaranteed to preserve strict message ordering. Default is `false`.
     */
    readonly fifoTopic: pulumi.Output<boolean | undefined>;
    /**
     * IAM role for failure feedback
     */
    readonly firehoseFailureFeedbackRoleArn: pulumi.Output<string | undefined>;
    /**
     * The IAM role permitted to receive success feedback for this topic
     */
    readonly firehoseSuccessFeedbackRoleArn: pulumi.Output<string | undefined>;
    /**
     * Percentage of success to sample
     */
    readonly firehoseSuccessFeedbackSampleRate: pulumi.Output<number | undefined>;
    /**
     * IAM role for failure feedback
     */
    readonly httpFailureFeedbackRoleArn: pulumi.Output<string | undefined>;
    /**
     * The IAM role permitted to receive success feedback for this topic
     */
    readonly httpSuccessFeedbackRoleArn: pulumi.Output<string | undefined>;
    /**
     * Percentage of success to sample
     */
    readonly httpSuccessFeedbackSampleRate: pulumi.Output<number | undefined>;
    /**
     * The ID of an AWS-managed customer master key (CMK) for Amazon SNS or a custom CMK. For more information, see [Key Terms](https://docs.aws.amazon.com/sns/latest/dg/sns-server-side-encryption.html#sse-key-terms)
     */
    readonly kmsMasterKeyId: pulumi.Output<string | undefined>;
    /**
     * IAM role for failure feedback
     */
    readonly lambdaFailureFeedbackRoleArn: pulumi.Output<string | undefined>;
    /**
     * The IAM role permitted to receive success feedback for this topic
     */
    readonly lambdaSuccessFeedbackRoleArn: pulumi.Output<string | undefined>;
    /**
     * Percentage of success to sample
     */
    readonly lambdaSuccessFeedbackSampleRate: pulumi.Output<number | undefined>;
    /**
     * The name of the topic. Topic names must be made up of only uppercase and lowercase ASCII letters, numbers, underscores, and hyphens, and must be between 1 and 256 characters long. For a FIFO (first-in-first-out) topic, the name must end with the `.fifo` suffix. If omitted, the provider will assign a random, unique name. Conflicts with `namePrefix`
     */
    readonly name: pulumi.Output<string>;
    /**
     * Creates a unique name beginning with the specified prefix. Conflicts with `name`
     */
    readonly namePrefix: pulumi.Output<string>;
    /**
     * The AWS Account ID of the SNS topic owner
     */
    readonly owner: pulumi.Output<string>;
    /**
     * The fully-formed AWS policy as JSON.
     */
    readonly policy: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * If `SignatureVersion` should be [1 (SHA1) or 2 (SHA256)](https://docs.aws.amazon.com/sns/latest/dg/sns-verify-signature-of-message.html). The signature version corresponds to the hashing algorithm used while creating the signature of the notifications, subscription confirmations, or unsubscribe confirmation messages sent by Amazon SNS.
     */
    readonly signatureVersion: pulumi.Output<number>;
    /**
     * IAM role for failure feedback
     */
    readonly sqsFailureFeedbackRoleArn: pulumi.Output<string | undefined>;
    /**
     * The IAM role permitted to receive success feedback for this topic
     */
    readonly sqsSuccessFeedbackRoleArn: pulumi.Output<string | undefined>;
    /**
     * Percentage of success to sample
     */
    readonly sqsSuccessFeedbackSampleRate: pulumi.Output<number | undefined>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Tracing mode of an Amazon SNS topic. Valid values: `"PassThrough"`, `"Active"`.
     */
    readonly tracingConfig: pulumi.Output<string>;
    /**
     * Create a Topic resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: TopicArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Topic resources.
 */
export interface TopicState {
    /**
     * IAM role for failure feedback
     */
    applicationFailureFeedbackRoleArn?: pulumi.Input<string>;
    /**
     * The IAM role permitted to receive success feedback for this topic
     */
    applicationSuccessFeedbackRoleArn?: pulumi.Input<string>;
    /**
     * Percentage of success to sample
     */
    applicationSuccessFeedbackSampleRate?: pulumi.Input<number>;
    /**
     * The message archive policy for FIFO topics. More details in the [AWS documentation](https://docs.aws.amazon.com/sns/latest/dg/message-archiving-and-replay-topic-owner.html).
     */
    archivePolicy?: pulumi.Input<string>;
    /**
     * The ARN of the SNS topic, as a more obvious property (clone of id)
     */
    arn?: pulumi.Input<string>;
    /**
     * The oldest timestamp at which a FIFO topic subscriber can start a replay.
     */
    beginningArchiveTime?: pulumi.Input<string>;
    /**
     * Enables content-based deduplication for FIFO topics. For more information, see the [related documentation](https://docs.aws.amazon.com/sns/latest/dg/fifo-message-dedup.html)
     */
    contentBasedDeduplication?: pulumi.Input<boolean>;
    /**
     * The SNS delivery policy. More details in the [AWS documentation](https://docs.aws.amazon.com/sns/latest/dg/DeliveryPolicies.html).
     */
    deliveryPolicy?: pulumi.Input<string>;
    /**
     * The display name for the topic
     */
    displayName?: pulumi.Input<string>;
    /**
     * Enables higher throughput for FIFO topics by adjusting the scope of deduplication. This attribute has two possible values, `Topic` and `MessageGroup`. For more information, see the [related documentation](https://docs.aws.amazon.com/sns/latest/dg/fifo-high-throughput.html#enable-high-throughput-on-fifo-topic).
     */
    fifoThroughputScope?: pulumi.Input<string>;
    /**
     * Boolean indicating whether or not to create a FIFO (first-in-first-out) topic. FIFO topics can't deliver messages to customer managed endpoints, such as email addresses, mobile apps, SMS, or HTTP(S) endpoints. These endpoint types aren't guaranteed to preserve strict message ordering. Default is `false`.
     */
    fifoTopic?: pulumi.Input<boolean>;
    /**
     * IAM role for failure feedback
     */
    firehoseFailureFeedbackRoleArn?: pulumi.Input<string>;
    /**
     * The IAM role permitted to receive success feedback for this topic
     */
    firehoseSuccessFeedbackRoleArn?: pulumi.Input<string>;
    /**
     * Percentage of success to sample
     */
    firehoseSuccessFeedbackSampleRate?: pulumi.Input<number>;
    /**
     * IAM role for failure feedback
     */
    httpFailureFeedbackRoleArn?: pulumi.Input<string>;
    /**
     * The IAM role permitted to receive success feedback for this topic
     */
    httpSuccessFeedbackRoleArn?: pulumi.Input<string>;
    /**
     * Percentage of success to sample
     */
    httpSuccessFeedbackSampleRate?: pulumi.Input<number>;
    /**
     * The ID of an AWS-managed customer master key (CMK) for Amazon SNS or a custom CMK. For more information, see [Key Terms](https://docs.aws.amazon.com/sns/latest/dg/sns-server-side-encryption.html#sse-key-terms)
     */
    kmsMasterKeyId?: pulumi.Input<string>;
    /**
     * IAM role for failure feedback
     */
    lambdaFailureFeedbackRoleArn?: pulumi.Input<string>;
    /**
     * The IAM role permitted to receive success feedback for this topic
     */
    lambdaSuccessFeedbackRoleArn?: pulumi.Input<string>;
    /**
     * Percentage of success to sample
     */
    lambdaSuccessFeedbackSampleRate?: pulumi.Input<number>;
    /**
     * The name of the topic. Topic names must be made up of only uppercase and lowercase ASCII letters, numbers, underscores, and hyphens, and must be between 1 and 256 characters long. For a FIFO (first-in-first-out) topic, the name must end with the `.fifo` suffix. If omitted, the provider will assign a random, unique name. Conflicts with `namePrefix`
     */
    name?: pulumi.Input<string>;
    /**
     * Creates a unique name beginning with the specified prefix. Conflicts with `name`
     */
    namePrefix?: pulumi.Input<string>;
    /**
     * The AWS Account ID of the SNS topic owner
     */
    owner?: pulumi.Input<string>;
    /**
     * The fully-formed AWS policy as JSON.
     */
    policy?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * If `SignatureVersion` should be [1 (SHA1) or 2 (SHA256)](https://docs.aws.amazon.com/sns/latest/dg/sns-verify-signature-of-message.html). The signature version corresponds to the hashing algorithm used while creating the signature of the notifications, subscription confirmations, or unsubscribe confirmation messages sent by Amazon SNS.
     */
    signatureVersion?: pulumi.Input<number>;
    /**
     * IAM role for failure feedback
     */
    sqsFailureFeedbackRoleArn?: pulumi.Input<string>;
    /**
     * The IAM role permitted to receive success feedback for this topic
     */
    sqsSuccessFeedbackRoleArn?: pulumi.Input<string>;
    /**
     * Percentage of success to sample
     */
    sqsSuccessFeedbackSampleRate?: pulumi.Input<number>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Tracing mode of an Amazon SNS topic. Valid values: `"PassThrough"`, `"Active"`.
     */
    tracingConfig?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a Topic resource.
 */
export interface TopicArgs {
    /**
     * IAM role for failure feedback
     */
    applicationFailureFeedbackRoleArn?: pulumi.Input<string>;
    /**
     * The IAM role permitted to receive success feedback for this topic
     */
    applicationSuccessFeedbackRoleArn?: pulumi.Input<string>;
    /**
     * Percentage of success to sample
     */
    applicationSuccessFeedbackSampleRate?: pulumi.Input<number>;
    /**
     * The message archive policy for FIFO topics. More details in the [AWS documentation](https://docs.aws.amazon.com/sns/latest/dg/message-archiving-and-replay-topic-owner.html).
     */
    archivePolicy?: pulumi.Input<string>;
    /**
     * Enables content-based deduplication for FIFO topics. For more information, see the [related documentation](https://docs.aws.amazon.com/sns/latest/dg/fifo-message-dedup.html)
     */
    contentBasedDeduplication?: pulumi.Input<boolean>;
    /**
     * The SNS delivery policy. More details in the [AWS documentation](https://docs.aws.amazon.com/sns/latest/dg/DeliveryPolicies.html).
     */
    deliveryPolicy?: pulumi.Input<string>;
    /**
     * The display name for the topic
     */
    displayName?: pulumi.Input<string>;
    /**
     * Enables higher throughput for FIFO topics by adjusting the scope of deduplication. This attribute has two possible values, `Topic` and `MessageGroup`. For more information, see the [related documentation](https://docs.aws.amazon.com/sns/latest/dg/fifo-high-throughput.html#enable-high-throughput-on-fifo-topic).
     */
    fifoThroughputScope?: pulumi.Input<string>;
    /**
     * Boolean indicating whether or not to create a FIFO (first-in-first-out) topic. FIFO topics can't deliver messages to customer managed endpoints, such as email addresses, mobile apps, SMS, or HTTP(S) endpoints. These endpoint types aren't guaranteed to preserve strict message ordering. Default is `false`.
     */
    fifoTopic?: pulumi.Input<boolean>;
    /**
     * IAM role for failure feedback
     */
    firehoseFailureFeedbackRoleArn?: pulumi.Input<string>;
    /**
     * The IAM role permitted to receive success feedback for this topic
     */
    firehoseSuccessFeedbackRoleArn?: pulumi.Input<string>;
    /**
     * Percentage of success to sample
     */
    firehoseSuccessFeedbackSampleRate?: pulumi.Input<number>;
    /**
     * IAM role for failure feedback
     */
    httpFailureFeedbackRoleArn?: pulumi.Input<string>;
    /**
     * The IAM role permitted to receive success feedback for this topic
     */
    httpSuccessFeedbackRoleArn?: pulumi.Input<string>;
    /**
     * Percentage of success to sample
     */
    httpSuccessFeedbackSampleRate?: pulumi.Input<number>;
    /**
     * The ID of an AWS-managed customer master key (CMK) for Amazon SNS or a custom CMK. For more information, see [Key Terms](https://docs.aws.amazon.com/sns/latest/dg/sns-server-side-encryption.html#sse-key-terms)
     */
    kmsMasterKeyId?: pulumi.Input<string>;
    /**
     * IAM role for failure feedback
     */
    lambdaFailureFeedbackRoleArn?: pulumi.Input<string>;
    /**
     * The IAM role permitted to receive success feedback for this topic
     */
    lambdaSuccessFeedbackRoleArn?: pulumi.Input<string>;
    /**
     * Percentage of success to sample
     */
    lambdaSuccessFeedbackSampleRate?: pulumi.Input<number>;
    /**
     * The name of the topic. Topic names must be made up of only uppercase and lowercase ASCII letters, numbers, underscores, and hyphens, and must be between 1 and 256 characters long. For a FIFO (first-in-first-out) topic, the name must end with the `.fifo` suffix. If omitted, the provider will assign a random, unique name. Conflicts with `namePrefix`
     */
    name?: pulumi.Input<string>;
    /**
     * Creates a unique name beginning with the specified prefix. Conflicts with `name`
     */
    namePrefix?: pulumi.Input<string>;
    /**
     * The fully-formed AWS policy as JSON.
     */
    policy?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * If `SignatureVersion` should be [1 (SHA1) or 2 (SHA256)](https://docs.aws.amazon.com/sns/latest/dg/sns-verify-signature-of-message.html). The signature version corresponds to the hashing algorithm used while creating the signature of the notifications, subscription confirmations, or unsubscribe confirmation messages sent by Amazon SNS.
     */
    signatureVersion?: pulumi.Input<number>;
    /**
     * IAM role for failure feedback
     */
    sqsFailureFeedbackRoleArn?: pulumi.Input<string>;
    /**
     * The IAM role permitted to receive success feedback for this topic
     */
    sqsSuccessFeedbackRoleArn?: pulumi.Input<string>;
    /**
     * Percentage of success to sample
     */
    sqsSuccessFeedbackSampleRate?: pulumi.Input<number>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Tracing mode of an Amazon SNS topic. Valid values: `"PassThrough"`, `"Active"`.
     */
    tracingConfig?: pulumi.Input<string>;
}
