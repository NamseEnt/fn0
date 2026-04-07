import * as pulumi from "@pulumi/pulumi";
/**
 * Manages the specified primary contact information associated with an AWS Account.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = new aws.account.PrimaryContact("test", {
 *     addressLine1: "123 Any Street",
 *     city: "Seattle",
 *     companyName: "Example Corp, Inc.",
 *     countryCode: "US",
 *     districtOrCounty: "King",
 *     fullName: "My Name",
 *     phoneNumber: "+64211111111",
 *     postalCode: "98101",
 *     stateOrRegion: "WA",
 *     websiteUrl: "https://www.examplecorp.com",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import the Primary Contact using the `account_id`. For example:
 *
 * ```sh
 * $ pulumi import aws:account/primaryContact:PrimaryContact test 1234567890
 * ```
 */
export declare class PrimaryContact extends pulumi.CustomResource {
    /**
     * Get an existing PrimaryContact resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: PrimaryContactState, opts?: pulumi.CustomResourceOptions): PrimaryContact;
    /**
     * Returns true if the given object is an instance of PrimaryContact.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is PrimaryContact;
    /**
     * The ID of the target account when managing member accounts. Will manage current user's account by default if omitted.
     */
    readonly accountId: pulumi.Output<string | undefined>;
    /**
     * The first line of the primary contact address.
     */
    readonly addressLine1: pulumi.Output<string>;
    /**
     * The second line of the primary contact address, if any.
     */
    readonly addressLine2: pulumi.Output<string | undefined>;
    /**
     * The third line of the primary contact address, if any.
     */
    readonly addressLine3: pulumi.Output<string | undefined>;
    /**
     * The city of the primary contact address.
     */
    readonly city: pulumi.Output<string>;
    /**
     * The name of the company associated with the primary contact information, if any.
     */
    readonly companyName: pulumi.Output<string | undefined>;
    /**
     * The ISO-3166 two-letter country code for the primary contact address.
     */
    readonly countryCode: pulumi.Output<string>;
    /**
     * The district or county of the primary contact address, if any.
     */
    readonly districtOrCounty: pulumi.Output<string | undefined>;
    /**
     * The full name of the primary contact address.
     */
    readonly fullName: pulumi.Output<string>;
    /**
     * The phone number of the primary contact information. The number will be validated and, in some countries, checked for activation.
     */
    readonly phoneNumber: pulumi.Output<string>;
    /**
     * The postal code of the primary contact address.
     */
    readonly postalCode: pulumi.Output<string>;
    /**
     * The state or region of the primary contact address. This field is required in selected countries.
     */
    readonly stateOrRegion: pulumi.Output<string | undefined>;
    /**
     * The URL of the website associated with the primary contact information, if any.
     */
    readonly websiteUrl: pulumi.Output<string | undefined>;
    /**
     * Create a PrimaryContact resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: PrimaryContactArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering PrimaryContact resources.
 */
export interface PrimaryContactState {
    /**
     * The ID of the target account when managing member accounts. Will manage current user's account by default if omitted.
     */
    accountId?: pulumi.Input<string>;
    /**
     * The first line of the primary contact address.
     */
    addressLine1?: pulumi.Input<string>;
    /**
     * The second line of the primary contact address, if any.
     */
    addressLine2?: pulumi.Input<string>;
    /**
     * The third line of the primary contact address, if any.
     */
    addressLine3?: pulumi.Input<string>;
    /**
     * The city of the primary contact address.
     */
    city?: pulumi.Input<string>;
    /**
     * The name of the company associated with the primary contact information, if any.
     */
    companyName?: pulumi.Input<string>;
    /**
     * The ISO-3166 two-letter country code for the primary contact address.
     */
    countryCode?: pulumi.Input<string>;
    /**
     * The district or county of the primary contact address, if any.
     */
    districtOrCounty?: pulumi.Input<string>;
    /**
     * The full name of the primary contact address.
     */
    fullName?: pulumi.Input<string>;
    /**
     * The phone number of the primary contact information. The number will be validated and, in some countries, checked for activation.
     */
    phoneNumber?: pulumi.Input<string>;
    /**
     * The postal code of the primary contact address.
     */
    postalCode?: pulumi.Input<string>;
    /**
     * The state or region of the primary contact address. This field is required in selected countries.
     */
    stateOrRegion?: pulumi.Input<string>;
    /**
     * The URL of the website associated with the primary contact information, if any.
     */
    websiteUrl?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a PrimaryContact resource.
 */
export interface PrimaryContactArgs {
    /**
     * The ID of the target account when managing member accounts. Will manage current user's account by default if omitted.
     */
    accountId?: pulumi.Input<string>;
    /**
     * The first line of the primary contact address.
     */
    addressLine1: pulumi.Input<string>;
    /**
     * The second line of the primary contact address, if any.
     */
    addressLine2?: pulumi.Input<string>;
    /**
     * The third line of the primary contact address, if any.
     */
    addressLine3?: pulumi.Input<string>;
    /**
     * The city of the primary contact address.
     */
    city: pulumi.Input<string>;
    /**
     * The name of the company associated with the primary contact information, if any.
     */
    companyName?: pulumi.Input<string>;
    /**
     * The ISO-3166 two-letter country code for the primary contact address.
     */
    countryCode: pulumi.Input<string>;
    /**
     * The district or county of the primary contact address, if any.
     */
    districtOrCounty?: pulumi.Input<string>;
    /**
     * The full name of the primary contact address.
     */
    fullName: pulumi.Input<string>;
    /**
     * The phone number of the primary contact information. The number will be validated and, in some countries, checked for activation.
     */
    phoneNumber: pulumi.Input<string>;
    /**
     * The postal code of the primary contact address.
     */
    postalCode: pulumi.Input<string>;
    /**
     * The state or region of the primary contact address. This field is required in selected countries.
     */
    stateOrRegion?: pulumi.Input<string>;
    /**
     * The URL of the website associated with the primary contact information, if any.
     */
    websiteUrl?: pulumi.Input<string>;
}
