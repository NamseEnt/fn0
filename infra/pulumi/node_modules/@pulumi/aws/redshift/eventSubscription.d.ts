import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a Redshift event subscription resource.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const _default = new aws.redshift.Cluster("default", {
 *     clusterIdentifier: "default",
 *     databaseName: "default",
 * });
 * const defaultTopic = new aws.sns.Topic("default", {name: "redshift-events"});
 * const defaultEventSubscription = new aws.redshift.EventSubscription("default", {
 *     name: "redshift-event-sub",
 *     snsTopicArn: defaultTopic.arn,
 *     sourceType: "cluster",
 *     sourceIds: [_default.id],
 *     severity: "INFO",
 *     eventCategories: [
 *         "configuration",
 *         "management",
 *         "monitoring",
 *         "security",
 *     ],
 *     tags: {
 *         Name: "default",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Redshift Event Subscriptions using the `name`. For example:
 *
 * ```sh
 * $ pulumi import aws:redshift/eventSubscription:EventSubscription default redshift-event-sub
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
     * Amazon Resource Name (ARN) of the Redshift event notification subscription
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The AWS customer account associated with the Redshift event notification subscription
     */
    readonly customerAwsId: pulumi.Output<string>;
    /**
     * A boolean flag to enable/disable the subscription. Defaults to `true`.
     */
    readonly enabled: pulumi.Output<boolean | undefined>;
    /**
     * A list of event categories for a SourceType that you want to subscribe to. See https://docs.aws.amazon.com/redshift/latest/mgmt/working-with-event-notifications.html or run `aws redshift describe-event-categories`.
     */
    readonly eventCategories: pulumi.Output<string[] | undefined>;
    /**
     * The name of the Redshift event subscription.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The event severity to be published by the notification subscription. Valid options are `INFO` or `ERROR`. Default value of `INFO`.
     */
    readonly severity: pulumi.Output<string | undefined>;
    /**
     * The ARN of the SNS topic to send events to.
     */
    readonly snsTopicArn: pulumi.Output<string>;
    /**
     * A list of identifiers of the event sources for which events will be returned. If not specified, then all sources are included in the response. If specified, a `sourceType` must also be specified.
     */
    readonly sourceIds: pulumi.Output<string[] | undefined>;
    /**
     * The type of source that will be generating the events. Valid options are `cluster`, `cluster-parameter-group`, `cluster-security-group`, `cluster-snapshot`, or `scheduled-action`. If not set, all sources will be subscribed to.
     */
    readonly sourceType: pulumi.Output<string | undefined>;
    readonly status: pulumi.Output<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Amazon Resource Name (ARN) of the Redshift event notification subscription
     */
    arn?: pulumi.Input<string>;
    /**
     * The AWS customer account associated with the Redshift event notification subscription
     */
    customerAwsId?: pulumi.Input<string>;
    /**
     * A boolean flag to enable/disable the subscription. Defaults to `true`.
     */
    enabled?: pulumi.Input<boolean>;
    /**
     * A list of event categories for a SourceType that you want to subscribe to. See https://docs.aws.amazon.com/redshift/latest/mgmt/working-with-event-notifications.html or run `aws redshift describe-event-categories`.
     */
    eventCategories?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The name of the Redshift event subscription.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The event severity to be published by the notification subscription. Valid options are `INFO` or `ERROR`. Default value of `INFO`.
     */
    severity?: pulumi.Input<string>;
    /**
     * The ARN of the SNS topic to send events to.
     */
    snsTopicArn?: pulumi.Input<string>;
    /**
     * A list of identifiers of the event sources for which events will be returned. If not specified, then all sources are included in the response. If specified, a `sourceType` must also be specified.
     */
    sourceIds?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The type of source that will be generating the events. Valid options are `cluster`, `cluster-parameter-group`, `cluster-security-group`, `cluster-snapshot`, or `scheduled-action`. If not set, all sources will be subscribed to.
     */
    sourceType?: pulumi.Input<string>;
    status?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * A boolean flag to enable/disable the subscription. Defaults to `true`.
     */
    enabled?: pulumi.Input<boolean>;
    /**
     * A list of event categories for a SourceType that you want to subscribe to. See https://docs.aws.amazon.com/redshift/latest/mgmt/working-with-event-notifications.html or run `aws redshift describe-event-categories`.
     */
    eventCategories?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The name of the Redshift event subscription.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The event severity to be published by the notification subscription. Valid options are `INFO` or `ERROR`. Default value of `INFO`.
     */
    severity?: pulumi.Input<string>;
    /**
     * The ARN of the SNS topic to send events to.
     */
    snsTopicArn: pulumi.Input<string>;
    /**
     * A list of identifiers of the event sources for which events will be returned. If not specified, then all sources are included in the response. If specified, a `sourceType` must also be specified.
     */
    sourceIds?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The type of source that will be generating the events. Valid options are `cluster`, `cluster-parameter-group`, `cluster-security-group`, `cluster-snapshot`, or `scheduled-action`. If not set, all sources will be subscribed to.
     */
    sourceType?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
