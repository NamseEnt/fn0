import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS Security Lake Subscriber Notification.
 *
 * ## Example Usage
 *
 * ### SQS Notification
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.securitylake.SubscriberNotification("example", {
 *     subscriberId: exampleAwsSecuritylakeSubscriber.id,
 *     configuration: {
 *         sqsNotificationConfiguration: {},
 *     },
 * });
 * ```
 *
 * ### HTTPS Notification
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.securitylake.SubscriberNotification("example", {
 *     subscriberId: exampleAwsSecuritylakeSubscriber.id,
 *     configuration: {
 *         httpsNotificationConfiguration: {
 *             endpoint: test.apiEndpoint,
 *             targetRoleArn: eventBridge.arn,
 *         },
 *     },
 * });
 * ```
 */
export declare class SubscriberNotification extends pulumi.CustomResource {
    /**
     * Get an existing SubscriberNotification resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: SubscriberNotificationState, opts?: pulumi.CustomResourceOptions): SubscriberNotification;
    /**
     * Returns true if the given object is an instance of SubscriberNotification.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is SubscriberNotification;
    /**
     * Specify the configuration using which you want to create the subscriber notification..
     */
    readonly configuration: pulumi.Output<outputs.securitylake.SubscriberNotificationConfiguration | undefined>;
    /**
     * (**Deprecated**) The subscriber endpoint to which exception messages are posted.
     *
     * @deprecated Use subscriberEndpoint instead
     */
    readonly endpointId: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The subscriber endpoint to which exception messages are posted.
     */
    readonly subscriberEndpoint: pulumi.Output<string>;
    /**
     * The subscriber ID for the notification subscription.
     */
    readonly subscriberId: pulumi.Output<string>;
    /**
     * Create a SubscriberNotification resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: SubscriberNotificationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering SubscriberNotification resources.
 */
export interface SubscriberNotificationState {
    /**
     * Specify the configuration using which you want to create the subscriber notification..
     */
    configuration?: pulumi.Input<inputs.securitylake.SubscriberNotificationConfiguration>;
    /**
     * (**Deprecated**) The subscriber endpoint to which exception messages are posted.
     *
     * @deprecated Use subscriberEndpoint instead
     */
    endpointId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The subscriber endpoint to which exception messages are posted.
     */
    subscriberEndpoint?: pulumi.Input<string>;
    /**
     * The subscriber ID for the notification subscription.
     */
    subscriberId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a SubscriberNotification resource.
 */
export interface SubscriberNotificationArgs {
    /**
     * Specify the configuration using which you want to create the subscriber notification..
     */
    configuration?: pulumi.Input<inputs.securitylake.SubscriberNotificationConfiguration>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The subscriber ID for the notification subscription.
     */
    subscriberId: pulumi.Input<string>;
}
