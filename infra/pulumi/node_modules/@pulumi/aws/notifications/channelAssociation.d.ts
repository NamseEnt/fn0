import * as pulumi from "@pulumi/pulumi";
/**
 * Resource for managing an AWS User Notifications Channel Association. This resource associates a channel (such as an email contact) with a notification configuration.
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
 *     name: "example-notification-config",
 *     description: "Example notification configuration",
 * });
 * const exampleContactsEmailContact = new aws.notifications.ContactsEmailContact("example", {
 *     name: "example-contact",
 *     emailAddress: "example@example.com",
 * });
 * const exampleChannelAssociation = new aws.notifications.ChannelAssociation("example", {
 *     arn: exampleContactsEmailContact.arn,
 *     notificationConfigurationArn: example.arn,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import User Notifications Channel Association using the `notification_configuration_arn,channel_arn` format. For example:
 *
 * ```sh
 * $ pulumi import aws:notifications/channelAssociation:ChannelAssociation example arn:aws:notifications:us-west-2:123456789012:configuration:example-notification-config,arn:aws:notificationscontacts:us-west-2:123456789012:emailcontact:example-contact
 * ```
 */
export declare class ChannelAssociation extends pulumi.CustomResource {
    /**
     * Get an existing ChannelAssociation resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ChannelAssociationState, opts?: pulumi.CustomResourceOptions): ChannelAssociation;
    /**
     * Returns true if the given object is an instance of ChannelAssociation.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ChannelAssociation;
    /**
     * ARN of the channel to associate with the notification configuration. This can be an email contact ARN.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * ARN of the notification configuration to associate the channel with.
     */
    readonly notificationConfigurationArn: pulumi.Output<string>;
    /**
     * Create a ChannelAssociation resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ChannelAssociationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ChannelAssociation resources.
 */
export interface ChannelAssociationState {
    /**
     * ARN of the channel to associate with the notification configuration. This can be an email contact ARN.
     */
    arn?: pulumi.Input<string>;
    /**
     * ARN of the notification configuration to associate the channel with.
     */
    notificationConfigurationArn?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a ChannelAssociation resource.
 */
export interface ChannelAssociationArgs {
    /**
     * ARN of the channel to associate with the notification configuration. This can be an email contact ARN.
     */
    arn: pulumi.Input<string>;
    /**
     * ARN of the notification configuration to associate the channel with.
     */
    notificationConfigurationArn: pulumi.Input<string>;
}
