import * as pulumi from "@pulumi/pulumi";
/**
 * Resource for managing an AWS User Notifications Event Rule.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.notifications.NotificationConfiguration("example", {
 *     name: "example",
 *     description: "example configuration",
 * });
 * const exampleEventRule = new aws.notifications.EventRule("example", {
 *     eventPattern: JSON.stringify({
 *         detail: {
 *             state: {
 *                 value: ["ALARM"],
 *             },
 *         },
 *     }),
 *     eventType: "CloudWatch Alarm State Change",
 *     notificationConfigurationArn: example.arn,
 *     regions: [
 *         "us-east-1",
 *         "us-west-2",
 *     ],
 *     source: "aws.cloudwatch",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import User Notifications Event Rule using the `arn`. For example:
 *
 * ```sh
 * $ pulumi import aws:notifications/eventRule:EventRule example arn:aws:notifications::123456789012:configuration/abc123def456ghi789jkl012mno345/rule/abc123def456ghi789jkl012mno345
 * ```
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
     * ARN of the Event Rule.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * JSON string defining the event pattern to match. Maximum length is 4096 characters.
     */
    readonly eventPattern: pulumi.Output<string | undefined>;
    /**
     * Type of event to match. Must be between 1 and 128 characters, and match the pattern `([a-zA-Z0-9 \-\(\)])+`.
     */
    readonly eventType: pulumi.Output<string>;
    /**
     * ARN of the notification configuration to associate with this event rule. Must match the pattern `arn:aws:notifications::[0-9]{12}:configuration/[a-z0-9]{27}`.
     */
    readonly notificationConfigurationArn: pulumi.Output<string>;
    /**
     * Set of AWS regions where the event rule will be applied. Each region must be between 2 and 25 characters, and match the pattern `([a-z]{1,2})-([a-z]{1,15}-)+([0-9])`.
     */
    readonly regions: pulumi.Output<string[]>;
    /**
     * Source of the event. Must be between 1 and 36 characters, and match the pattern `aws.([a-z0-9\-])+`.
     *
     * The following arguments are optional:
     */
    readonly source: pulumi.Output<string>;
    /**
     * Create a EventRule resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: EventRuleArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering EventRule resources.
 */
export interface EventRuleState {
    /**
     * ARN of the Event Rule.
     */
    arn?: pulumi.Input<string>;
    /**
     * JSON string defining the event pattern to match. Maximum length is 4096 characters.
     */
    eventPattern?: pulumi.Input<string>;
    /**
     * Type of event to match. Must be between 1 and 128 characters, and match the pattern `([a-zA-Z0-9 \-\(\)])+`.
     */
    eventType?: pulumi.Input<string>;
    /**
     * ARN of the notification configuration to associate with this event rule. Must match the pattern `arn:aws:notifications::[0-9]{12}:configuration/[a-z0-9]{27}`.
     */
    notificationConfigurationArn?: pulumi.Input<string>;
    /**
     * Set of AWS regions where the event rule will be applied. Each region must be between 2 and 25 characters, and match the pattern `([a-z]{1,2})-([a-z]{1,15}-)+([0-9])`.
     */
    regions?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Source of the event. Must be between 1 and 36 characters, and match the pattern `aws.([a-z0-9\-])+`.
     *
     * The following arguments are optional:
     */
    source?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a EventRule resource.
 */
export interface EventRuleArgs {
    /**
     * JSON string defining the event pattern to match. Maximum length is 4096 characters.
     */
    eventPattern?: pulumi.Input<string>;
    /**
     * Type of event to match. Must be between 1 and 128 characters, and match the pattern `([a-zA-Z0-9 \-\(\)])+`.
     */
    eventType: pulumi.Input<string>;
    /**
     * ARN of the notification configuration to associate with this event rule. Must match the pattern `arn:aws:notifications::[0-9]{12}:configuration/[a-z0-9]{27}`.
     */
    notificationConfigurationArn: pulumi.Input<string>;
    /**
     * Set of AWS regions where the event rule will be applied. Each region must be between 2 and 25 characters, and match the pattern `([a-z]{1,2})-([a-z]{1,15}-)+([0-9])`.
     */
    regions: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Source of the event. Must be between 1 and 36 characters, and match the pattern `aws.([a-z0-9\-])+`.
     *
     * The following arguments are optional:
     */
    source: pulumi.Input<string>;
}
