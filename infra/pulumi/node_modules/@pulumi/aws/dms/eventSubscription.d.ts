import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a DMS (Data Migration Service) event subscription resource.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.dms.EventSubscription("example", {
 *     enabled: true,
 *     eventCategories: [
 *         "creation",
 *         "failure",
 *     ],
 *     name: "my-favorite-event-subscription",
 *     snsTopicArn: exampleAwsSnsTopic.arn,
 *     sourceIds: [exampleAwsDmsReplicationTask.replicationTaskId],
 *     sourceType: "replication-task",
 *     tags: {
 *         Name: "example",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import event subscriptions using the `name`. For example:
 *
 * ```sh
 * $ pulumi import aws:dms/eventSubscription:EventSubscription test my-awesome-event-subscription
 * ```
 */
export declare class EventSubscription extends pulumi.CustomResource {
    /**
     * Get an existing EventSubscription resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: EventSubscriptionState, opts?: pulumi.CustomResourceOptions): EventSubscription;
    /**
     * Returns true if the given object is an instance of EventSubscription.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is EventSubscription;
    /**
     * Amazon Resource Name (ARN) of the DMS Event Subscription.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Whether the event subscription should be enabled.
     */
    readonly enabled: pulumi.Output<boolean | undefined>;
    /**
     * List of event categories to listen for, see `DescribeEventCategories` for a canonical list.
     */
    readonly eventCategories: pulumi.Output<string[]>;
    /**
     * Name of event subscription.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * SNS topic arn to send events on.
     */
    readonly snsTopicArn: pulumi.Output<string>;
    /**
     * Ids of sources to listen to. If you don't specify a value, notifications are provided for all sources.
     */
    readonly sourceIds: pulumi.Output<string[] | undefined>;
    /**
     * Type of source for events. Valid values: `replication-instance` or `replication-task`
     */
    readonly sourceType: pulumi.Output<string>;
    /**
     * Map of resource tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Create a EventSubscription resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: EventSubscriptionArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering EventSubscription resources.
 */
export interface EventSubscriptionState {
    /**
     * Amazon Resource Name (ARN) of the DMS Event Subscription.
     */
    arn?: pulumi.Input<string>;
    /**
     * Whether the event subscription should be enabled.
     */
    enabled?: pulumi.Input<boolean>;
    /**
     * List of event categories to listen for, see `DescribeEventCategories` for a canonical list.
     */
    eventCategories?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Name of event subscription.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * SNS topic arn to send events on.
     */
    snsTopicArn?: pulumi.Input<string>;
    /**
     * Ids of sources to listen to. If you don't specify a value, notifications are provided for all sources.
     */
    sourceIds?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Type of source for events. Valid values: `replication-instance` or `replication-task`
     */
    sourceType?: pulumi.Input<string>;
    /**
     * Map of resource tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
 * The set of arguments for constructing a EventSubscription resource.
 */
export interface EventSubscriptionArgs {
    /**
     * Whether the event subscription should be enabled.
     */
    enabled?: pulumi.Input<boolean>;
    /**
     * List of event categories to listen for, see `DescribeEventCategories` for a canonical list.
     */
    eventCategories: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Name of event subscription.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * SNS topic arn to send events on.
     */
    snsTopicArn: pulumi.Input<string>;
    /**
     * Ids of sources to listen to. If you don't specify a value, notifications are provided for all sources.
     */
    sourceIds?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Type of source for events. Valid values: `replication-instance` or `replication-task`
     */
    sourceType: pulumi.Input<string>;
    /**
     * Map of resource tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
