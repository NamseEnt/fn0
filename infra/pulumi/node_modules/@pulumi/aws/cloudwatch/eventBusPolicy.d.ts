import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a resource to create an EventBridge resource policy to support cross-account events.
 *
 * > **Note:** EventBridge was formerly known as CloudWatch Events. The functionality is identical.
 *
 * > **Note:** The EventBridge bus policy resource  (`aws.cloudwatch.EventBusPolicy`) is incompatible with the EventBridge permission resource (`aws.cloudwatch.EventPermission`) and will overwrite permissions.
 *
 * ## Example Usage
 *
 * ### Account Access
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = aws.iam.getPolicyDocument({
 *     statements: [{
 *         sid: "DevAccountAccess",
 *         effect: "Allow",
 *         actions: ["events:PutEvents"],
 *         resources: ["arn:aws:events:eu-west-1:123456789012:event-bus/default"],
 *         principals: [{
 *             type: "AWS",
 *             identifiers: ["123456789012"],
 *         }],
 *     }],
 * });
 * const testEventBusPolicy = new aws.cloudwatch.EventBusPolicy("test", {
 *     policy: test.then(test => test.json),
 *     eventBusName: testAwsCloudwatchEventBus.name,
 * });
 * ```
 *
 * ### Organization Access
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = aws.iam.getPolicyDocument({
 *     statements: [{
 *         sid: "OrganizationAccess",
 *         effect: "Allow",
 *         actions: [
 *             "events:DescribeRule",
 *             "events:ListRules",
 *             "events:ListTargetsByRule",
 *             "events:ListTagsForResource",
 *         ],
 *         resources: [
 *             "arn:aws:events:eu-west-1:123456789012:rule/*",
 *             "arn:aws:events:eu-west-1:123456789012:event-bus/default",
 *         ],
 *         principals: [{
 *             type: "AWS",
 *             identifiers: ["*"],
 *         }],
 *         conditions: [{
 *             test: "StringEquals",
 *             variable: "aws:PrincipalOrgID",
 *             values: [example.id],
 *         }],
 *     }],
 * });
 * const testEventBusPolicy = new aws.cloudwatch.EventBusPolicy("test", {
 *     policy: test.then(test => test.json),
 *     eventBusName: testAwsCloudwatchEventBus.name,
 * });
 * ```
 *
 * ### Multiple Statements
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = aws.iam.getPolicyDocument({
 *     statements: [
 *         {
 *             sid: "DevAccountAccess",
 *             effect: "Allow",
 *             actions: ["events:PutEvents"],
 *             resources: ["arn:aws:events:eu-west-1:123456789012:event-bus/default"],
 *             principals: [{
 *                 type: "AWS",
 *                 identifiers: ["123456789012"],
 *             }],
 *         },
 *         {
 *             sid: "OrganizationAccess",
 *             effect: "Allow",
 *             actions: [
 *                 "events:DescribeRule",
 *                 "events:ListRules",
 *                 "events:ListTargetsByRule",
 *                 "events:ListTagsForResource",
 *             ],
 *             resources: [
 *                 "arn:aws:events:eu-west-1:123456789012:rule/*",
 *                 "arn:aws:events:eu-west-1:123456789012:event-bus/default",
 *             ],
 *             principals: [{
 *                 type: "AWS",
 *                 identifiers: ["*"],
 *             }],
 *             conditions: [{
 *                 test: "StringEquals",
 *                 variable: "aws:PrincipalOrgID",
 *                 values: [example.id],
 *             }],
 *         },
 *     ],
 * });
 * const testEventBusPolicy = new aws.cloudwatch.EventBusPolicy("test", {
 *     policy: test.then(test => test.json),
 *     eventBusName: testAwsCloudwatchEventBus.name,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import an EventBridge policy using the `event_bus_name`. For example:
 *
 * ```sh
 * $ pulumi import aws:cloudwatch/eventBusPolicy:EventBusPolicy DevAccountAccess example-event-bus
 * ```
 */
export declare class EventBusPolicy extends pulumi.CustomResource {
    /**
     * Get an existing EventBusPolicy resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: EventBusPolicyState, opts?: pulumi.CustomResourceOptions): EventBusPolicy;
    /**
     * Returns true if the given object is an instance of EventBusPolicy.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is EventBusPolicy;
    /**
     * The name of the event bus to set the permissions on.
     * If you omit this, the permissions are set on the `default` event bus.
     */
    readonly eventBusName: pulumi.Output<string | undefined>;
    /**
     * The text of the policy.
     */
    readonly policy: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Create a EventBusPolicy resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: EventBusPolicyArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering EventBusPolicy resources.
 */
export interface EventBusPolicyState {
    /**
     * The name of the event bus to set the permissions on.
     * If you omit this, the permissions are set on the `default` event bus.
     */
    eventBusName?: pulumi.Input<string>;
    /**
     * The text of the policy.
     */
    policy?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a EventBusPolicy resource.
 */
export interface EventBusPolicyArgs {
    /**
     * The name of the event bus to set the permissions on.
     * If you omit this, the permissions are set on the `default` event bus.
     */
    eventBusName?: pulumi.Input<string>;
    /**
     * The text of the policy.
     */
    policy: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
