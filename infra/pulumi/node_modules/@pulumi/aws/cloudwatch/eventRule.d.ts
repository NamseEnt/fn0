import * as pulumi from "@pulumi/pulumi";
/**
 * Provides an EventBridge Rule resource.
 *
 * > **Note:** EventBridge was formerly known as CloudWatch Events. The functionality is identical.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const console = new aws.cloudwatch.EventRule("console", {
 *     name: "capture-aws-sign-in",
 *     description: "Capture each AWS Console Sign In",
 *     eventPattern: JSON.stringify({
 *         "detail-type": ["AWS Console Sign In via CloudTrail"],
 *     }),
 * });
 * const awsLogins = new aws.sns.Topic("aws_logins", {name: "aws-console-logins"});
 * const sns = new aws.cloudwatch.EventTarget("sns", {
 *     rule: console.name,
 *     targetId: "SendToSNS",
 *     arn: awsLogins.arn,
 * });
 * const snsTopicPolicy = aws.iam.getPolicyDocumentOutput({
 *     statements: [{
 *         effect: "Allow",
 *         actions: ["SNS:Publish"],
 *         principals: [{
 *             type: "Service",
 *             identifiers: ["events.amazonaws.com"],
 *         }],
 *         resources: [awsLogins.arn],
 *     }],
 * });
 * const _default = new aws.sns.TopicPolicy("default", {
 *     arn: awsLogins.arn,
 *     policy: snsTopicPolicy.apply(snsTopicPolicy => snsTopicPolicy.json),
 * });
 * ```
 *
 * ## Import
 *
 * ### Identity Schema
 *
 * #### Required
 *
 * * `name` (String) Name of the EventBridge rule.
 *
 * #### Optional
 *
 * * `account_id` (String) AWS Account where this resource is managed.
 *
 * * `event_bus_name` (String) Name of the event bus.
 *
 * * `region` (String) Region where this resource is managed.
 *
 * Using `pulumi import`, import EventBridge Rules using the `event_bus_name/rule_name` (if you omit `event_bus_name`, the `default` event bus will be used). For example:
 *
 * console
 *
 * % pulumi import aws_cloudwatch_event_rule.example example-event-bus/capture-console-sign-in
 */
export declare class EventRule extends pulumi.CustomResource {
    /**
     * Get an existing EventRule resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: EventRuleState, opts?: pulumi.CustomResourceOptions): EventRule;
    /**
     * Returns true if the given object is an instance of EventRule.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is EventRule;
    /**
     * The Amazon Resource Name (ARN) of the rule.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The description of the rule.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * The name or ARN of the event bus to associate with this rule. If you omit this, the `default` event bus is used.
     */
    readonly eventBusName: pulumi.Output<string | undefined>;
    /**
     * The event pattern described a JSON object. At least one of `scheduleExpression` or `eventPattern` is required. See full documentation of [Events and Event Patterns in EventBridge](https://docs.aws.amazon.com/eventbridge/latest/userguide/eventbridge-and-event-patterns.html) for details. **Note**: The event pattern size is 2048 by default but it is adjustable up to 4096 characters by submitting a service quota increase request. See [Amazon EventBridge quotas](https://docs.aws.amazon.com/eventbridge/latest/userguide/eb-quota.html) for details.
     */
    readonly eventPattern: pulumi.Output<string | undefined>;
    /**
     * Used to delete managed rules created by AWS. Defaults to `false`.
     */
    readonly forceDestroy: pulumi.Output<boolean | undefined>;
    /**
     * Whether the rule should be enabled. Defaults to `true`. Conflicts with `state`.
     *
     * @deprecated is_enabled is deprecated. Use state instead.
     */
    readonly isEnabled: pulumi.Output<boolean | undefined>;
    /**
     * The name of the rule. If omitted, this provider will assign a random, unique name. Conflicts with `namePrefix`.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Creates a unique name beginning with the specified prefix. Conflicts with `name`. **Note**: Due to the length of the generated suffix, must be 38 characters or less.
     */
    readonly namePrefix: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The Amazon Resource Name (ARN) associated with the role that is used for target invocation.
     */
    readonly roleArn: pulumi.Output<string | undefined>;
    /**
     * The scheduling expression. For example, `cron(0 20 * * ? *)` or `rate(5 minutes)`. At least one of `scheduleExpression` or `eventPattern` is required. Can only be used on the default event bus. For more information, refer to the AWS documentation [Schedule Expressions for Rules](https://docs.aws.amazon.com/AmazonCloudWatch/latest/events/ScheduledEvents.html).
     */
    readonly scheduleExpression: pulumi.Output<string | undefined>;
    /**
     * State of the rule. Valid values are `DISABLED`, `ENABLED`, and `ENABLED_WITH_ALL_CLOUDTRAIL_MANAGEMENT_EVENTS`. When state is `ENABLED`, the rule is enabled for all events except those delivered by CloudTrail. To also enable the rule for events delivered by CloudTrail, set `state` to `ENABLED_WITH_ALL_CLOUDTRAIL_MANAGEMENT_EVENTS`. Defaults to `ENABLED`. Conflicts with `isEnabled`.
     */
    readonly state: pulumi.Output<string | undefined>;
    /**
     * A map of tags to assign to the resource. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     *
     * **NOTE:** The rule state `ENABLED_WITH_ALL_CLOUDTRAIL_MANAGEMENT_EVENTS` cannot be used in conjunction with the `scheduleExpression` argument.
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
     * Create a EventRule resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: EventRuleArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering EventRule resources.
 */
export interface EventRuleState {
    /**
     * The Amazon Resource Name (ARN) of the rule.
     */
    arn?: pulumi.Input<string>;
    /**
     * The description of the rule.
     */
    description?: pulumi.Input<string>;
    /**
     * The name or ARN of the event bus to associate with this rule. If you omit this, the `default` event bus is used.
     */
    eventBusName?: pulumi.Input<string>;
    /**
     * The event pattern described a JSON object. At least one of `scheduleExpression` or `eventPattern` is required. See full documentation of [Events and Event Patterns in EventBridge](https://docs.aws.amazon.com/eventbridge/latest/userguide/eventbridge-and-event-patterns.html) for details. **Note**: The event pattern size is 2048 by default but it is adjustable up to 4096 characters by submitting a service quota increase request. See [Amazon EventBridge quotas](https://docs.aws.amazon.com/eventbridge/latest/userguide/eb-quota.html) for details.
     */
    eventPattern?: pulumi.Input<string>;
    /**
     * Used to delete managed rules created by AWS. Defaults to `false`.
     */
    forceDestroy?: pulumi.Input<boolean>;
    /**
     * Whether the rule should be enabled. Defaults to `true`. Conflicts with `state`.
     *
     * @deprecated is_enabled is deprecated. Use state instead.
     */
    isEnabled?: pulumi.Input<boolean>;
    /**
     * The name of the rule. If omitted, this provider will assign a random, unique name. Conflicts with `namePrefix`.
     */
    name?: pulumi.Input<string>;
    /**
     * Creates a unique name beginning with the specified prefix. Conflicts with `name`. **Note**: Due to the length of the generated suffix, must be 38 characters or less.
     */
    namePrefix?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) associated with the role that is used for target invocation.
     */
    roleArn?: pulumi.Input<string>;
    /**
     * The scheduling expression. For example, `cron(0 20 * * ? *)` or `rate(5 minutes)`. At least one of `scheduleExpression` or `eventPattern` is required. Can only be used on the default event bus. For more information, refer to the AWS documentation [Schedule Expressions for Rules](https://docs.aws.amazon.com/AmazonCloudWatch/latest/events/ScheduledEvents.html).
     */
    scheduleExpression?: pulumi.Input<string>;
    /**
     * State of the rule. Valid values are `DISABLED`, `ENABLED`, and `ENABLED_WITH_ALL_CLOUDTRAIL_MANAGEMENT_EVENTS`. When state is `ENABLED`, the rule is enabled for all events except those delivered by CloudTrail. To also enable the rule for events delivered by CloudTrail, set `state` to `ENABLED_WITH_ALL_CLOUDTRAIL_MANAGEMENT_EVENTS`. Defaults to `ENABLED`. Conflicts with `isEnabled`.
     */
    state?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     *
     * **NOTE:** The rule state `ENABLED_WITH_ALL_CLOUDTRAIL_MANAGEMENT_EVENTS` cannot be used in conjunction with the `scheduleExpression` argument.
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
 * The set of arguments for constructing a EventRule resource.
 */
export interface EventRuleArgs {
    /**
     * The description of the rule.
     */
    description?: pulumi.Input<string>;
    /**
     * The name or ARN of the event bus to associate with this rule. If you omit this, the `default` event bus is used.
     */
    eventBusName?: pulumi.Input<string>;
    /**
     * The event pattern described a JSON object. At least one of `scheduleExpression` or `eventPattern` is required. See full documentation of [Events and Event Patterns in EventBridge](https://docs.aws.amazon.com/eventbridge/latest/userguide/eventbridge-and-event-patterns.html) for details. **Note**: The event pattern size is 2048 by default but it is adjustable up to 4096 characters by submitting a service quota increase request. See [Amazon EventBridge quotas](https://docs.aws.amazon.com/eventbridge/latest/userguide/eb-quota.html) for details.
     */
    eventPattern?: pulumi.Input<string>;
    /**
     * Used to delete managed rules created by AWS. Defaults to `false`.
     */
    forceDestroy?: pulumi.Input<boolean>;
    /**
     * Whether the rule should be enabled. Defaults to `true`. Conflicts with `state`.
     *
     * @deprecated is_enabled is deprecated. Use state instead.
     */
    isEnabled?: pulumi.Input<boolean>;
    /**
     * The name of the rule. If omitted, this provider will assign a random, unique name. Conflicts with `namePrefix`.
     */
    name?: pulumi.Input<string>;
    /**
     * Creates a unique name beginning with the specified prefix. Conflicts with `name`. **Note**: Due to the length of the generated suffix, must be 38 characters or less.
     */
    namePrefix?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) associated with the role that is used for target invocation.
     */
    roleArn?: pulumi.Input<string>;
    /**
     * The scheduling expression. For example, `cron(0 20 * * ? *)` or `rate(5 minutes)`. At least one of `scheduleExpression` or `eventPattern` is required. Can only be used on the default event bus. For more information, refer to the AWS documentation [Schedule Expressions for Rules](https://docs.aws.amazon.com/AmazonCloudWatch/latest/events/ScheduledEvents.html).
     */
    scheduleExpression?: pulumi.Input<string>;
    /**
     * State of the rule. Valid values are `DISABLED`, `ENABLED`, and `ENABLED_WITH_ALL_CLOUDTRAIL_MANAGEMENT_EVENTS`. When state is `ENABLED`, the rule is enabled for all events except those delivered by CloudTrail. To also enable the rule for events delivered by CloudTrail, set `state` to `ENABLED_WITH_ALL_CLOUDTRAIL_MANAGEMENT_EVENTS`. Defaults to `ENABLED`. Conflicts with `isEnabled`.
     */
    state?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     *
     * **NOTE:** The rule state `ENABLED_WITH_ALL_CLOUDTRAIL_MANAGEMENT_EVENTS` cannot be used in conjunction with the `scheduleExpression` argument.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
