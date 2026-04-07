import * as pulumi from "@pulumi/pulumi";
/**
 * Manages the specified alternate contact attached to an AWS Account.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const operations = new aws.account.AlternativeContact("operations", {
 *     alternateContactType: "OPERATIONS",
 *     name: "Example",
 *     title: "Example",
 *     emailAddress: "test@example.com",
 *     phoneNumber: "+1234567890",
 * });
 * ```
 *
 * ## Import
 *
 * Import the Alternate Contact for another account using the `account_id` and `alternate_contact_type` separated by a forward slash (`/`):
 *
 * __Using `pulumi import` to import__ the Alternate Contact for the current or another account using the `alternate_contact_type`. For example:
 *
 * Import the Alternate Contact for the current account:
 *
 * ```sh
 * $ pulumi import aws:account/alternativeContact:AlternativeContact operations OPERATIONS
 * ```
 * Import the Alternate Contact for another account using the `account_id` and `alternate_contact_type` separated by a forward slash (`/`):
 *
 * ```sh
 * $ pulumi import aws:account/alternativeContact:AlternativeContact operations 1234567890/OPERATIONS
 * ```
 */
export declare class AlternativeContact extends pulumi.CustomResource {
    /**
     * Get an existing AlternativeContact resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: AlternativeContactState, opts?: pulumi.CustomResourceOptions): AlternativeContact;
    /**
     * Returns true if the given object is an instance of AlternativeContact.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is AlternativeContact;
    /**
     * ID of the target account when managing member accounts. Will manage current user's account by default if omitted.
     */
    readonly accountId: pulumi.Output<string | undefined>;
    /**
     * Type of the alternate contact. Allowed values are: `BILLING`, `OPERATIONS`, `SECURITY`.
     */
    readonly alternateContactType: pulumi.Output<string>;
    /**
     * An email address for the alternate contact.
     */
    readonly emailAddress: pulumi.Output<string>;
    /**
     * Name of the alternate contact.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Phone number for the alternate contact.
     */
    readonly phoneNumber: pulumi.Output<string>;
    /**
     * Title for the alternate contact.
     */
    readonly title: pulumi.Output<string>;
    /**
     * Create a AlternativeContact resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: AlternativeContactArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering AlternativeContact resources.
 */
export interface AlternativeContactState {
    /**
     * ID of the target account when managing member accounts. Will manage current user's account by default if omitted.
     */
    accountId?: pulumi.Input<string>;
    /**
     * Type of the alternate contact. Allowed values are: `BILLING`, `OPERATIONS`, `SECURITY`.
     */
    alternateContactType?: pulumi.Input<string>;
    /**
     * An email address for the alternate contact.
     */
    emailAddress?: pulumi.Input<string>;
    /**
     * Name of the alternate contact.
     */
    name?: pulumi.Input<string>;
    /**
     * Phone number for the alternate contact.
     */
    phoneNumber?: pulumi.Input<string>;
    /**
     * Title for the alternate contact.
     */
    title?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a AlternativeContact resource.
 */
export interface AlternativeContactArgs {
    /**
     * ID of the target account when managing member accounts. Will manage current user's account by default if omitted.
     */
    accountId?: pulumi.Input<string>;
    /**
     * Type of the alternate contact. Allowed values are: `BILLING`, `OPERATIONS`, `SECURITY`.
     */
    alternateContactType: pulumi.Input<string>;
    /**
     * An email address for the alternate contact.
     */
    emailAddress: pulumi.Input<string>;
    /**
     * Name of the alternate contact.
     */
    name?: pulumi.Input<string>;
    /**
     * Phone number for the alternate contact.
     */
    phoneNumber: pulumi.Input<string>;
    /**
     * Title for the alternate contact.
     */
    title: pulumi.Input<string>;
}
