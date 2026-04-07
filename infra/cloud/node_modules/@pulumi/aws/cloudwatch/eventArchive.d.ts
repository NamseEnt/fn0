import * as pulumi from "@pulumi/pulumi";
/**
 * Provides an EventBridge event archive resource.
 *
 * > **Note:** EventBridge was formerly known as CloudWatch Events. The functionality is identical.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const order = new aws.cloudwatch.EventBus("order", {name: "orders"});
 * const orderEventArchive = new aws.cloudwatch.EventArchive("order", {
 *     name: "order-archive",
 *     eventSourceArn: order.arn,
 * });
 * ```
 *
 * ### Optional Arguments
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const order = new aws.cloudwatch.EventBus("order", {name: "orders"});
 * const orderEventArchive = new aws.cloudwatch.EventArchive("order", {
 *     name: "order-archive",
 *     description: "Archived events from order service",
 *     eventSourceArn: order.arn,
 *     retentionDays: 7,
 *     eventPattern: JSON.stringify({
 *         source: ["company.team.order"],
 *     }),
 * });
 * ```
 *
 * ### CMK Encryption
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const current = aws.getCallerIdentity({});
 * const currentGetPartition = aws.getPartition({});
 * const example = new aws.cloudwatch.EventBus("example", {name: "example"});
 * const exampleKey = new aws.kms.Key("example", {
 *     deletionWindowInDays: 7,
 *     policy: pulumi.jsonStringify({
 *         Version: "2012-10-17",
 *         Id: "key-policy-example",
 *         Statement: [
 *             {
 *                 Sid: "Enable IAM User Permissions",
 *                 Effect: "Allow",
 *                 Principal: {
 *                     AWS: Promise.all([currentGetPartition, current]).then(([currentGetPartition, current]) => `arn:${currentGetPartition.partition}:iam::${current.accountId}:root`),
 *                 },
 *                 Action: "kms:*",
 *                 Resource: "*",
 *             },
 *             {
 *                 Sid: "Allow describing of the key",
 *                 Effect: "Allow",
 *                 Principal: {
 *                     Service: "events.amazonaws.com",
 *                 },
 *                 Action: ["kms:DescribeKey"],
 *                 Resource: "*",
 *             },
 *             {
 *                 Sid: "Allow use of the key",
 *                 Effect: "Allow",
 *                 Principal: {
 *                     Service: "events.amazonaws.com",
 *                 },
 *                 Action: [
 *                     "kms:GenerateDataKey",
 *                     "kms:Decrypt",
 *                     "kms:ReEncrypt*",
 *                 ],
 *                 Resource: "*",
 *                 Condition: {
 *                     StringEquals: {
 *                         "kms:EncryptionContext:aws:events:event-bus:arn": example.arn,
 *                     },
 *                 },
 *             },
 *         ],
 *     }),
 *     tags: {
 *         EventBridgeApiDestinations: "true",
 *     },
 * });
 * const exampleEventArchive = new aws.cloudwatch.EventArchive("example", {
 *     name: "example",
 *     eventSourceArn: example.arn,
 *     kmsKeyIdentifier: exampleKey.id,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import an EventBridge archive using the `name`. For example:
 *
 * ```sh
 * $ pulumi import aws:cloudwatch/eventArchive:EventArchive imported_event_archive order-archive
 * ```
 */
export declare class EventArchive extends pulumi.CustomResource {
    /**
     * Get an existing EventArchive resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: EventArchiveState, opts?: pulumi.CustomResourceOptions): EventArchive;
    /**
     * Returns true if the given object is an instance of EventArchive.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is EventArchive;
    /**
     * ARN of the archive.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Description for the archive.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * Event pattern to use to filter events sent to the archive. By default, it attempts to archive every event received in the `eventSourceArn`.
     */
    readonly eventPattern: pulumi.Output<string | undefined>;
    /**
     * ARN of the event bus associated with the archive. Only events from this event bus are sent to the archive.
     */
    readonly eventSourceArn: pulumi.Output<string>;
    /**
     * Identifier of the AWS KMS customer managed key for EventBridge to use, if you choose to use a customer managed key to encrypt this archive. The identifier can be the key Amazon Resource Name (ARN), KeyId, key alias, or key alias ARN.
     */
    readonly kmsKeyIdentifier: pulumi.Output<string | undefined>;
    /**
     * Name of the archive. The archive name cannot exceed 48 characters.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The maximum number of days to retain events in the new event archive. By default, it archives indefinitely.
     */
    readonly retentionDays: pulumi.Output<number | undefined>;
    /**
     * Create a EventArchive resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: EventArchiveArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering EventArchive resources.
 */
export interface EventArchiveState {
    /**
     * ARN of the archive.
     */
    arn?: pulumi.Input<string>;
    /**
     * Description for the archive.
     */
    description?: pulumi.Input<string>;
    /**
     * Event pattern to use to filter events sent to the archive. By default, it attempts to archive every event received in the `eventSourceArn`.
     */
    eventPattern?: pulumi.Input<string>;
    /**
     * ARN of the event bus associated with the archive. Only events from this event bus are sent to the archive.
     */
    eventSourceArn?: pulumi.Input<string>;
    /**
     * Identifier of the AWS KMS customer managed key for EventBridge to use, if you choose to use a customer managed key to encrypt this archive. The identifier can be the key Amazon Resource Name (ARN), KeyId, key alias, or key alias ARN.
     */
    kmsKeyIdentifier?: pulumi.Input<string>;
    /**
     * Name of the archive. The archive name cannot exceed 48 characters.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The maximum number of days to retain events in the new event archive. By default, it archives indefinitely.
     */
    retentionDays?: pulumi.Input<number>;
}
/**
 * The set of arguments for constructing a EventArchive resource.
 */
export interface EventArchiveArgs {
    /**
     * Description for the archive.
     */
    description?: pulumi.Input<string>;
    /**
     * Event pattern to use to filter events sent to the archive. By default, it attempts to archive every event received in the `eventSourceArn`.
     */
    eventPattern?: pulumi.Input<string>;
    /**
     * ARN of the event bus associated with the archive. Only events from this event bus are sent to the archive.
     */
    eventSourceArn: pulumi.Input<string>;
    /**
     * Identifier of the AWS KMS customer managed key for EventBridge to use, if you choose to use a customer managed key to encrypt this archive. The identifier can be the key Amazon Resource Name (ARN), KeyId, key alias, or key alias ARN.
     */
    kmsKeyIdentifier?: pulumi.Input<string>;
    /**
     * Name of the archive. The archive name cannot exceed 48 characters.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The maximum number of days to retain events in the new event archive. By default, it archives indefinitely.
     */
    retentionDays?: pulumi.Input<number>;
}
