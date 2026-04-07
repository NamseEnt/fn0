import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * This resource manages a User resource within an Identity Store.
 *
 * > **Note:** If you use an external identity provider or Active Directory as your identity source,
 * use this resource with caution. IAM Identity Center does not support outbound synchronization,
 * so your identity source does not automatically update with the changes that you make to
 * users using this resource.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.identitystore.User("example", {
 *     identityStoreId: exampleAwsSsoadminInstances.identityStoreIds[0],
 *     displayName: "John Doe",
 *     userName: "johndoe",
 *     name: {
 *         givenName: "John",
 *         familyName: "Doe",
 *     },
 *     emails: {
 *         value: "john@example.com",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import an Identity Store User using the combination `identity_store_id/user_id`. For example:
 *
 * ```sh
 * $ pulumi import aws:identitystore/user:User example d-9c6705e95c/065212b4-9061-703b-5876-13a517ae2a7c
 * ```
 */
export declare class User extends pulumi.CustomResource {
    /**
     * Get an existing User resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: UserState, opts?: pulumi.CustomResourceOptions): User;
    /**
     * Returns true if the given object is an instance of User.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is User;
    /**
     * Details about the user's address. At most 1 address is allowed. Detailed below.
     */
    readonly addresses: pulumi.Output<outputs.identitystore.UserAddresses | undefined>;
    /**
     * The name that is typically displayed when the user is referenced.
     */
    readonly displayName: pulumi.Output<string>;
    /**
     * Details about the user's email. At most 1 email is allowed. Detailed below.
     */
    readonly emails: pulumi.Output<outputs.identitystore.UserEmails | undefined>;
    /**
     * A list of identifiers issued to this resource by an external identity provider.
     */
    readonly externalIds: pulumi.Output<outputs.identitystore.UserExternalId[]>;
    /**
     * The globally unique identifier for the identity store that this user is in.
     */
    readonly identityStoreId: pulumi.Output<string>;
    /**
     * The user's geographical region or location.
     */
    readonly locale: pulumi.Output<string | undefined>;
    /**
     * Details about the user's full name. Detailed below.
     */
    readonly name: pulumi.Output<outputs.identitystore.UserName>;
    /**
     * An alternate name for the user.
     */
    readonly nickname: pulumi.Output<string | undefined>;
    /**
     * Details about the user's phone number. At most 1 phone number is allowed. Detailed below.
     */
    readonly phoneNumbers: pulumi.Output<outputs.identitystore.UserPhoneNumbers | undefined>;
    /**
     * The preferred language of the user.
     */
    readonly preferredLanguage: pulumi.Output<string | undefined>;
    /**
     * An URL that may be associated with the user.
     */
    readonly profileUrl: pulumi.Output<string | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The user's time zone.
     */
    readonly timezone: pulumi.Output<string | undefined>;
    /**
     * The user's title.
     */
    readonly title: pulumi.Output<string | undefined>;
    /**
     * The identifier for this user in the identity store.
     */
    readonly userId: pulumi.Output<string>;
    /**
     * A unique string used to identify the user. This value can consist of letters, accented characters, symbols, numbers, and punctuation. This value is specified at the time the user is created and stored as an attribute of the user object in the identity store. The limit is 128 characters.
     *
     * The following arguments are optional:
     */
    readonly userName: pulumi.Output<string>;
    /**
     * The user type.
     *
     * > Unless specified otherwise, all fields can contain up to 1024 characters of free-form text.
     */
    readonly userType: pulumi.Output<string | undefined>;
    /**
     * Create a User resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: UserArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering User resources.
 */
export interface UserState {
    /**
     * Details about the user's address. At most 1 address is allowed. Detailed below.
     */
    addresses?: pulumi.Input<inputs.identitystore.UserAddresses>;
    /**
     * The name that is typically displayed when the user is referenced.
     */
    displayName?: pulumi.Input<string>;
    /**
     * Details about the user's email. At most 1 email is allowed. Detailed below.
     */
    emails?: pulumi.Input<inputs.identitystore.UserEmails>;
    /**
     * A list of identifiers issued to this resource by an external identity provider.
     */
    externalIds?: pulumi.Input<pulumi.Input<inputs.identitystore.UserExternalId>[]>;
    /**
     * The globally unique identifier for the identity store that this user is in.
     */
    identityStoreId?: pulumi.Input<string>;
    /**
     * The user's geographical region or location.
     */
    locale?: pulumi.Input<string>;
    /**
     * Details about the user's full name. Detailed below.
     */
    name?: pulumi.Input<inputs.identitystore.UserName>;
    /**
     * An alternate name for the user.
     */
    nickname?: pulumi.Input<string>;
    /**
     * Details about the user's phone number. At most 1 phone number is allowed. Detailed below.
     */
    phoneNumbers?: pulumi.Input<inputs.identitystore.UserPhoneNumbers>;
    /**
     * The preferred language of the user.
     */
    preferredLanguage?: pulumi.Input<string>;
    /**
     * An URL that may be associated with the user.
     */
    profileUrl?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The user's time zone.
     */
    timezone?: pulumi.Input<string>;
    /**
     * The user's title.
     */
    title?: pulumi.Input<string>;
    /**
     * The identifier for this user in the identity store.
     */
    userId?: pulumi.Input<string>;
    /**
     * A unique string used to identify the user. This value can consist of letters, accented characters, symbols, numbers, and punctuation. This value is specified at the time the user is created and stored as an attribute of the user object in the identity store. The limit is 128 characters.
     *
     * The following arguments are optional:
     */
    userName?: pulumi.Input<string>;
    /**
     * The user type.
     *
     * > Unless specified otherwise, all fields can contain up to 1024 characters of free-form text.
     */
    userType?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a User resource.
 */
export interface UserArgs {
    /**
     * Details about the user's address. At most 1 address is allowed. Detailed below.
     */
    addresses?: pulumi.Input<inputs.identitystore.UserAddresses>;
    /**
     * The name that is typically displayed when the user is referenced.
     */
    displayName: pulumi.Input<string>;
    /**
     * Details about the user's email. At most 1 email is allowed. Detailed below.
     */
    emails?: pulumi.Input<inputs.identitystore.UserEmails>;
    /**
     * The globally unique identifier for the identity store that this user is in.
     */
    identityStoreId: pulumi.Input<string>;
    /**
     * The user's geographical region or location.
     */
    locale?: pulumi.Input<string>;
    /**
     * Details about the user's full name. Detailed below.
     */
    name?: pulumi.Input<inputs.identitystore.UserName>;
    /**
     * An alternate name for the user.
     */
    nickname?: pulumi.Input<string>;
    /**
     * Details about the user's phone number. At most 1 phone number is allowed. Detailed below.
     */
    phoneNumbers?: pulumi.Input<inputs.identitystore.UserPhoneNumbers>;
    /**
     * The preferred language of the user.
     */
    preferredLanguage?: pulumi.Input<string>;
    /**
     * An URL that may be associated with the user.
     */
    profileUrl?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The user's time zone.
     */
    timezone?: pulumi.Input<string>;
    /**
     * The user's title.
     */
    title?: pulumi.Input<string>;
    /**
     * A unique string used to identify the user. This value can consist of letters, accented characters, symbols, numbers, and punctuation. This value is specified at the time the user is created and stored as an attribute of the user object in the identity store. The limit is 128 characters.
     *
     * The following arguments are optional:
     */
    userName: pulumi.Input<string>;
    /**
     * The user type.
     *
     * > Unless specified otherwise, all fields can contain up to 1024 characters of free-form text.
     */
    userType?: pulumi.Input<string>;
}
