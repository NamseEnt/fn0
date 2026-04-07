import * as pulumi from "@pulumi/pulumi";
/**
 * Resource for managing AWS User Notifications Contacts Email Contact.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.notifications.ContactsEmailContact("example", {
 *     name: "example-contact",
 *     emailAddress: "example@example.com",
 *     tags: {
 *         Environment: "Production",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import User Notifications Contacts Email Contact using the `arn`. For example:
 *
 * ```sh
 * $ pulumi import aws:notifications/contactsEmailContact:ContactsEmailContact example arn:aws:notificationscontacts:us-west-2:123456789012:emailcontact:example-contact
 * ```
 */
export declare class ContactsEmailContact extends pulumi.CustomResource {
    /**
     * Get an existing ContactsEmailContact resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ContactsEmailContactState, opts?: pulumi.CustomResourceOptions): ContactsEmailContact;
    /**
     * Returns true if the given object is an instance of ContactsEmailContact.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ContactsEmailContact;
    /**
     * ARN of the Email Contact.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Email address for the contact. Must be between 6 and 254 characters and match an email
     * pattern.
     */
    readonly emailAddress: pulumi.Output<string>;
    /**
     * Name of the email contact. Must be between 1 and 64 characters and can contain alphanumeric
     * characters, underscores, tildes, periods, and hyphens.
     *
     * The following arguments are optional:
     */
    readonly name: pulumi.Output<string>;
    /**
     * Map of tags to assign to the resource. If configured with a provider
     * `defaultTags` configuration block
     * present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * Map of tags assigned to the resource, including those inherited from the provider
     * `defaultTags` configuration block.
     */
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * Create a ContactsEmailContact resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ContactsEmailContactArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ContactsEmailContact resources.
 */
export interface ContactsEmailContactState {
    /**
     * ARN of the Email Contact.
     */
    arn?: pulumi.Input<string>;
    /**
     * Email address for the contact. Must be between 6 and 254 characters and match an email
     * pattern.
     */
    emailAddress?: pulumi.Input<string>;
    /**
     * Name of the email contact. Must be between 1 and 64 characters and can contain alphanumeric
     * characters, underscores, tildes, periods, and hyphens.
     *
     * The following arguments are optional:
     */
    name?: pulumi.Input<string>;
    /**
     * Map of tags to assign to the resource. If configured with a provider
     * `defaultTags` configuration block
     * present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Map of tags assigned to the resource, including those inherited from the provider
     * `defaultTags` configuration block.
     */
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
/**
 * The set of arguments for constructing a ContactsEmailContact resource.
 */
export interface ContactsEmailContactArgs {
    /**
     * Email address for the contact. Must be between 6 and 254 characters and match an email
     * pattern.
     */
    emailAddress: pulumi.Input<string>;
    /**
     * Name of the email contact. Must be between 1 and 64 characters and can contain alphanumeric
     * characters, underscores, tildes, periods, and hyphens.
     *
     * The following arguments are optional:
     */
    name?: pulumi.Input<string>;
    /**
     * Map of tags to assign to the resource. If configured with a provider
     * `defaultTags` configuration block
     * present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
