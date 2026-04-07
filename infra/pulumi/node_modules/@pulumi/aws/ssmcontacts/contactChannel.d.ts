import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS SSM Contacts Contact Channel.
 *
 * > **NOTE:** The contact channel needs to be activated in the AWS Systems Manager console, otherwise it can't be used to engage the contact. See the [Contacts section of the Incident Manager User Guide](https://docs.aws.amazon.com/incident-manager/latest/userguide/contacts.html) for more information.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.ssmcontacts.ContactChannel("example", {
 *     contactId: "arn:aws:ssm-contacts:us-west-2:123456789012:contact/contactalias",
 *     deliveryAddress: {
 *         simpleAddress: "email@example.com",
 *     },
 *     name: "Example contact channel",
 *     type: "EMAIL",
 * });
 * ```
 *
 * ### Usage with SSM Contact
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const exampleContact = new aws.ssmcontacts.Contact("example_contact", {
 *     alias: "example_contact",
 *     type: "PERSONAL",
 * });
 * const example = new aws.ssmcontacts.ContactChannel("example", {
 *     contactId: exampleContact.arn,
 *     deliveryAddress: {
 *         simpleAddress: "email@example.com",
 *     },
 *     name: "Example contact channel",
 *     type: "EMAIL",
 * });
 * ```
 *
 * ## Import
 *
 * ### Identity Schema
 *
 * #### Required
 *
 * - `arn` (String) Amazon Resource Name (ARN) of the contact channel.
 *
 * Using `pulumi import`, import SSM Contact Channel using the `arn`. For example:
 *
 * console
 *
 * % pulumi import aws_ssmcontacts_contact_channel.example arn:aws:ssm-contacts:us-west-2:123456789012:contact-channel/example
 */
export declare class ContactChannel extends pulumi.CustomResource {
    /**
     * Get an existing ContactChannel resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ContactChannelState, opts?: pulumi.CustomResourceOptions): ContactChannel;
    /**
     * Returns true if the given object is an instance of ContactChannel.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ContactChannel;
    /**
     * Whether the contact channel is activated. The contact channel must be activated to use it to engage the contact. One of `ACTIVATED` or `NOT_ACTIVATED`.
     */
    readonly activationStatus: pulumi.Output<string>;
    /**
     * Amazon Resource Name (ARN) of the contact channel.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Amazon Resource Name (ARN) of the AWS SSM Contact that the contact channel belongs to.
     */
    readonly contactId: pulumi.Output<string>;
    /**
     * Block that contains contact engagement details. See details below.
     */
    readonly deliveryAddress: pulumi.Output<outputs.ssmcontacts.ContactChannelDeliveryAddress>;
    /**
     * Name of the contact channel. Must be between 1 and 255 characters, and may contain alphanumerics, underscores (`_`), hyphens (`-`), periods (`.`), and spaces.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Type of the contact channel. One of `SMS`, `VOICE` or `EMAIL`.
     */
    readonly type: pulumi.Output<string>;
    /**
     * Create a ContactChannel resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ContactChannelArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ContactChannel resources.
 */
export interface ContactChannelState {
    /**
     * Whether the contact channel is activated. The contact channel must be activated to use it to engage the contact. One of `ACTIVATED` or `NOT_ACTIVATED`.
     */
    activationStatus?: pulumi.Input<string>;
    /**
     * Amazon Resource Name (ARN) of the contact channel.
     */
    arn?: pulumi.Input<string>;
    /**
     * Amazon Resource Name (ARN) of the AWS SSM Contact that the contact channel belongs to.
     */
    contactId?: pulumi.Input<string>;
    /**
     * Block that contains contact engagement details. See details below.
     */
    deliveryAddress?: pulumi.Input<inputs.ssmcontacts.ContactChannelDeliveryAddress>;
    /**
     * Name of the contact channel. Must be between 1 and 255 characters, and may contain alphanumerics, underscores (`_`), hyphens (`-`), periods (`.`), and spaces.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Type of the contact channel. One of `SMS`, `VOICE` or `EMAIL`.
     */
    type?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a ContactChannel resource.
 */
export interface ContactChannelArgs {
    /**
     * Amazon Resource Name (ARN) of the AWS SSM Contact that the contact channel belongs to.
     */
    contactId: pulumi.Input<string>;
    /**
     * Block that contains contact engagement details. See details below.
     */
    deliveryAddress: pulumi.Input<inputs.ssmcontacts.ContactChannelDeliveryAddress>;
    /**
     * Name of the contact channel. Must be between 1 and 255 characters, and may contain alphanumerics, underscores (`_`), hyphens (`-`), periods (`.`), and spaces.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Type of the contact channel. One of `SMS`, `VOICE` or `EMAIL`.
     */
    type: pulumi.Input<string>;
}
