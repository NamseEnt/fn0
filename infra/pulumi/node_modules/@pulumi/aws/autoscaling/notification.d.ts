import * as pulumi from "@pulumi/pulumi";
import * as enums from "../types/enums";
/**
 * Provides an AutoScaling Group with Notification support, via SNS Topics. Each of
 * the `notifications` map to a [Notification Configuration](https://docs.aws.amazon.com/AutoScaling/latest/APIReference/API_DescribeNotificationConfigurations.html) inside Amazon Web
 * Services, and are applied to each AutoScaling Group you supply.
 *
 * ## Example Usage
 *
 * Basic usage:
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.sns.Topic("example", {name: "example-topic"});
 * const bar = new aws.autoscaling.Group("bar", {name: "foobar1-test"});
 * const foo = new aws.autoscaling.Group("foo", {name: "barfoo-test"});
 * const exampleNotifications = new aws.autoscaling.Notification("example_notifications", {
 *     groupNames: [
 *         bar.name,
 *         foo.name,
 *     ],
 *     notifications: [
 *         aws.autoscaling.NotificationType.InstanceLaunch,
 *         aws.autoscaling.NotificationType.InstanceTerminate,
 *         aws.autoscaling.NotificationType.InstanceLaunchError,
 *         aws.autoscaling.NotificationType.InstanceTerminateError,
 *     ],
 *     topicArn: example.arn,
 * });
 * ```
 */
export declare class Notification extends pulumi.CustomResource {
    /**
     * Get an existing Notification resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: NotificationState, opts?: pulumi.CustomResourceOptions): Notification;
    /**
     * Returns true if the given object is an instance of Notification.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Notification;
    /**
     * List of AutoScaling Group Names
     */
    readonly groupNames: pulumi.Output<string[]>;
    /**
     * List of Notification Types that trigger
     * notifications. Acceptable values are documented [in the AWS documentation here](https://docs.aws.amazon.com/AutoScaling/latest/APIReference/API_NotificationConfiguration.html)
     */
    readonly notifications: pulumi.Output<enums.autoscaling.NotificationType[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Topic ARN for notifications to be sent through
     */
    readonly topicArn: pulumi.Output<string>;
    /**
     * Create a Notification resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: NotificationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Notification resources.
 */
export interface NotificationState {
    /**
     * List of AutoScaling Group Names
     */
    groupNames?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * List of Notification Types that trigger
     * notifications. Acceptable values are documented [in the AWS documentation here](https://docs.aws.amazon.com/AutoScaling/latest/APIReference/API_NotificationConfiguration.html)
     */
    notifications?: pulumi.Input<pulumi.Input<enums.autoscaling.NotificationType>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Topic ARN for notifications to be sent through
     */
    topicArn?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a Notification resource.
 */
export interface NotificationArgs {
    /**
     * List of AutoScaling Group Names
     */
    groupNames: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * List of Notification Types that trigger
     * notifications. Acceptable values are documented [in the AWS documentation here](https://docs.aws.amazon.com/AutoScaling/latest/APIReference/API_NotificationConfiguration.html)
     */
    notifications: pulumi.Input<pulumi.Input<enums.autoscaling.NotificationType>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Topic ARN for notifications to be sent through
     */
    topicArn: pulumi.Input<string>;
}
