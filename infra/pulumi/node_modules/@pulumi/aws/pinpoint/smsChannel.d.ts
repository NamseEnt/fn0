import * as pulumi from "@pulumi/pulumi";
/**
 * Use the `aws.pinpoint.SmsChannel` resource to manage Pinpoint SMS Channels.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const app = new aws.pinpoint.App("app", {});
 * const sms = new aws.pinpoint.SmsChannel("sms", {applicationId: app.applicationId});
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import the Pinpoint SMS Channel using the `application_id`. For example:
 *
 * ```sh
 * $ pulumi import aws:pinpoint/smsChannel:SmsChannel sms application-id
 * ```
 */
export declare class SmsChannel extends pulumi.CustomResource {
    /**
     * Get an existing SmsChannel resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: SmsChannelState, opts?: pulumi.CustomResourceOptions): SmsChannel;
    /**
     * Returns true if the given object is an instance of SmsChannel.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is SmsChannel;
    /**
     * ID of the application.
     */
    readonly applicationId: pulumi.Output<string>;
    /**
     * Whether the channel is enabled or disabled. By default, it is set to `true`.
     */
    readonly enabled: pulumi.Output<boolean | undefined>;
    /**
     * Maximum number of promotional messages that can be sent per second.
     */
    readonly promotionalMessagesPerSecond: pulumi.Output<number>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Identifier of the sender for your messages.
     */
    readonly senderId: pulumi.Output<string | undefined>;
    /**
     * Short Code registered with the phone provider.
     */
    readonly shortCode: pulumi.Output<string | undefined>;
    /**
     * Maximum number of transactional messages per second that can be sent.
     */
    readonly transactionalMessagesPerSecond: pulumi.Output<number>;
    /**
     * Create a SmsChannel resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: SmsChannelArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering SmsChannel resources.
 */
export interface SmsChannelState {
    /**
     * ID of the application.
     */
    applicationId?: pulumi.Input<string>;
    /**
     * Whether the channel is enabled or disabled. By default, it is set to `true`.
     */
    enabled?: pulumi.Input<boolean>;
    /**
     * Maximum number of promotional messages that can be sent per second.
     */
    promotionalMessagesPerSecond?: pulumi.Input<number>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Identifier of the sender for your messages.
     */
    senderId?: pulumi.Input<string>;
    /**
     * Short Code registered with the phone provider.
     */
    shortCode?: pulumi.Input<string>;
    /**
     * Maximum number of transactional messages per second that can be sent.
     */
    transactionalMessagesPerSecond?: pulumi.Input<number>;
}
/**
 * The set of arguments for constructing a SmsChannel resource.
 */
export interface SmsChannelArgs {
    /**
     * ID of the application.
     */
    applicationId: pulumi.Input<string>;
    /**
     * Whether the channel is enabled or disabled. By default, it is set to `true`.
     */
    enabled?: pulumi.Input<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Identifier of the sender for your messages.
     */
    senderId?: pulumi.Input<string>;
    /**
     * Short Code registered with the phone provider.
     */
    shortCode?: pulumi.Input<string>;
}
