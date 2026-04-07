import * as pulumi from "@pulumi/pulumi";
/**
 * Provides an AutoScaling Lifecycle Hook resource.
 *
 * > **NOTE:** This provider has two types of ways you can add lifecycle hooks - via
 * the `initialLifecycleHook` attribute from the
 * `aws.autoscaling.Group`
 * resource, or via this one. Hooks added via this resource will not be added
 * until the autoscaling group has been created, and depending on your
 * capacity
 * settings, after the initial instances have been launched, creating unintended
 * behavior. If you need hooks to run on all instances, add them with
 * `initialLifecycleHook` in
 * `aws.autoscaling.Group`,
 * but take care to not duplicate those hooks with this resource.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const foobar = new aws.autoscaling.Group("foobar", {
 *     availabilityZones: ["us-west-2a"],
 *     name: "test-foobar5",
 *     healthCheckType: "EC2",
 *     terminationPolicies: ["OldestInstance"],
 *     tags: [{
 *         key: "Foo",
 *         value: "foo-bar",
 *         propagateAtLaunch: true,
 *     }],
 * });
 * const foobarLifecycleHook = new aws.autoscaling.LifecycleHook("foobar", {
 *     name: "foobar",
 *     autoscalingGroupName: foobar.name,
 *     defaultResult: "CONTINUE",
 *     heartbeatTimeout: 2000,
 *     lifecycleTransition: "autoscaling:EC2_INSTANCE_LAUNCHING",
 *     notificationMetadata: JSON.stringify({
 *         foo: "bar",
 *     }),
 *     notificationTargetArn: "arn:aws:sqs:us-east-1:444455556666:queue1*",
 *     roleArn: "arn:aws:iam::123456789012:role/S3Access",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import AutoScaling Lifecycle Hooks using the role autoscaling_group_name and name separated by `/`. For example:
 *
 * ```sh
 * $ pulumi import aws:autoscaling/lifecycleHook:LifecycleHook test-lifecycle-hook asg-name/lifecycle-hook-name
 * ```
 */
export declare class LifecycleHook extends pulumi.CustomResource {
    /**
     * Get an existing LifecycleHook resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: LifecycleHookState, opts?: pulumi.CustomResourceOptions): LifecycleHook;
    /**
     * Returns true if the given object is an instance of LifecycleHook.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is LifecycleHook;
    /**
     * Name of the Auto Scaling group to which you want to assign the lifecycle hook
     */
    readonly autoscalingGroupName: pulumi.Output<string>;
    /**
     * Defines the action the Auto Scaling group should take when the lifecycle hook timeout elapses or if an unexpected failure occurs. The value for this parameter can be either CONTINUE or ABANDON. The default value for this parameter is ABANDON.
     */
    readonly defaultResult: pulumi.Output<string>;
    /**
     * Defines the amount of time, in seconds, that can elapse before the lifecycle hook times out. When the lifecycle hook times out, Auto Scaling performs the action defined in the DefaultResult parameter
     */
    readonly heartbeatTimeout: pulumi.Output<number | undefined>;
    /**
     * Instance state to which you want to attach the lifecycle hook. For a list of lifecycle hook types, see [describe-lifecycle-hook-types](https://docs.aws.amazon.com/cli/latest/reference/autoscaling/describe-lifecycle-hook-types.html#examples)
     */
    readonly lifecycleTransition: pulumi.Output<string>;
    /**
     * Name of the lifecycle hook.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Contains additional information that you want to include any time Auto Scaling sends a message to the notification target.
     */
    readonly notificationMetadata: pulumi.Output<string | undefined>;
    /**
     * ARN of the notification target that Auto Scaling will use to notify you when an instance is in the transition state for the lifecycle hook. This ARN target can be either an SQS queue, an SNS topic, or a Lambda function.
     */
    readonly notificationTargetArn: pulumi.Output<string | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * ARN of the IAM role that allows the Auto Scaling group to publish to the specified notification target.
     */
    readonly roleArn: pulumi.Output<string | undefined>;
    /**
     * Create a LifecycleHook resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: LifecycleHookArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering LifecycleHook resources.
 */
export interface LifecycleHookState {
    /**
     * Name of the Auto Scaling group to which you want to assign the lifecycle hook
     */
    autoscalingGroupName?: pulumi.Input<string>;
    /**
     * Defines the action the Auto Scaling group should take when the lifecycle hook timeout elapses or if an unexpected failure occurs. The value for this parameter can be either CONTINUE or ABANDON. The default value for this parameter is ABANDON.
     */
    defaultResult?: pulumi.Input<string>;
    /**
     * Defines the amount of time, in seconds, that can elapse before the lifecycle hook times out. When the lifecycle hook times out, Auto Scaling performs the action defined in the DefaultResult parameter
     */
    heartbeatTimeout?: pulumi.Input<number>;
    /**
     * Instance state to which you want to attach the lifecycle hook. For a list of lifecycle hook types, see [describe-lifecycle-hook-types](https://docs.aws.amazon.com/cli/latest/reference/autoscaling/describe-lifecycle-hook-types.html#examples)
     */
    lifecycleTransition?: pulumi.Input<string>;
    /**
     * Name of the lifecycle hook.
     */
    name?: pulumi.Input<string>;
    /**
     * Contains additional information that you want to include any time Auto Scaling sends a message to the notification target.
     */
    notificationMetadata?: pulumi.Input<string>;
    /**
     * ARN of the notification target that Auto Scaling will use to notify you when an instance is in the transition state for the lifecycle hook. This ARN target can be either an SQS queue, an SNS topic, or a Lambda function.
     */
    notificationTargetArn?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ARN of the IAM role that allows the Auto Scaling group to publish to the specified notification target.
     */
    roleArn?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a LifecycleHook resource.
 */
export interface LifecycleHookArgs {
    /**
     * Name of the Auto Scaling group to which you want to assign the lifecycle hook
     */
    autoscalingGroupName: pulumi.Input<string>;
    /**
     * Defines the action the Auto Scaling group should take when the lifecycle hook timeout elapses or if an unexpected failure occurs. The value for this parameter can be either CONTINUE or ABANDON. The default value for this parameter is ABANDON.
     */
    defaultResult?: pulumi.Input<string>;
    /**
     * Defines the amount of time, in seconds, that can elapse before the lifecycle hook times out. When the lifecycle hook times out, Auto Scaling performs the action defined in the DefaultResult parameter
     */
    heartbeatTimeout?: pulumi.Input<number>;
    /**
     * Instance state to which you want to attach the lifecycle hook. For a list of lifecycle hook types, see [describe-lifecycle-hook-types](https://docs.aws.amazon.com/cli/latest/reference/autoscaling/describe-lifecycle-hook-types.html#examples)
     */
    lifecycleTransition: pulumi.Input<string>;
    /**
     * Name of the lifecycle hook.
     */
    name?: pulumi.Input<string>;
    /**
     * Contains additional information that you want to include any time Auto Scaling sends a message to the notification target.
     */
    notificationMetadata?: pulumi.Input<string>;
    /**
     * ARN of the notification target that Auto Scaling will use to notify you when an instance is in the transition state for the lifecycle hook. This ARN target can be either an SQS queue, an SNS topic, or a Lambda function.
     */
    notificationTargetArn?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ARN of the IAM role that allows the Auto Scaling group to publish to the specified notification target.
     */
    roleArn?: pulumi.Input<string>;
}
