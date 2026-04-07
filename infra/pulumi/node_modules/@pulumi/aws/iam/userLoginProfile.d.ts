import * as pulumi from "@pulumi/pulumi";
/**
 * Manages an IAM User Login Profile with limited support for password creation during this provider resource creation. Uses PGP to encrypt the password for safe transport to the user. PGP keys can be obtained from Keybase.
 *
 * > To reset an IAM User login password via this provider, you can use delete and recreate this resource or change any of the arguments.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.iam.User("example", {
 *     name: "example",
 *     path: "/",
 *     forceDestroy: true,
 * });
 * const exampleUserLoginProfile = new aws.iam.UserLoginProfile("example", {
 *     user: example.name,
 *     pgpKey: "keybase:some_person_that_exists",
 * });
 * export const password = exampleUserLoginProfile.encryptedPassword;
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import IAM User Login Profiles without password information via the IAM User name. For example:
 *
 * ```sh
 * $ pulumi import aws:iam/userLoginProfile:UserLoginProfile example myusername
 * ```
 * Since Pulumi has no method to read the PGP or password information during import, use the resource options `ignore_changes` argument to ignore them (unless you want to recreate a password). For example:
 */
export declare class UserLoginProfile extends pulumi.CustomResource {
    /**
     * Get an existing UserLoginProfile resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: UserLoginProfileState, opts?: pulumi.CustomResourceOptions): UserLoginProfile;
    /**
     * Returns true if the given object is an instance of UserLoginProfile.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is UserLoginProfile;
    /**
     * The encrypted password, base64 encoded. Only available if password was handled on resource creation, not import.
     */
    readonly encryptedPassword: pulumi.Output<string>;
    /**
     * The fingerprint of the PGP key used to encrypt the password. Only available if password was handled on this provider resource creation, not import.
     */
    readonly keyFingerprint: pulumi.Output<string>;
    /**
     * The plain text password, only available when `pgpKey` is not provided.
     */
    readonly password: pulumi.Output<string>;
    /**
     * The length of the generated password on resource creation. Only applies on resource creation. Drift detection is not possible with this argument. Default value is `20`.
     */
    readonly passwordLength: pulumi.Output<number | undefined>;
    /**
     * Whether the user should be forced to reset the generated password on resource creation. Only applies on resource creation.
     */
    readonly passwordResetRequired: pulumi.Output<boolean>;
    /**
     * Either a base-64 encoded PGP public key, or a keybase username in the form `keybase:username`. Only applies on resource creation. Drift detection is not possible with this argument.
     */
    readonly pgpKey: pulumi.Output<string | undefined>;
    /**
     * The IAM user's name.
     */
    readonly user: pulumi.Output<string>;
    /**
     * Create a UserLoginProfile resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: UserLoginProfileArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering UserLoginProfile resources.
 */
export interface UserLoginProfileState {
    /**
     * The encrypted password, base64 encoded. Only available if password was handled on resource creation, not import.
     */
    encryptedPassword?: pulumi.Input<string>;
    /**
     * The fingerprint of the PGP key used to encrypt the password. Only available if password was handled on this provider resource creation, not import.
     */
    keyFingerprint?: pulumi.Input<string>;
    /**
     * The plain text password, only available when `pgpKey` is not provided.
     */
    password?: pulumi.Input<string>;
    /**
     * The length of the generated password on resource creation. Only applies on resource creation. Drift detection is not possible with this argument. Default value is `20`.
     */
    passwordLength?: pulumi.Input<number>;
    /**
     * Whether the user should be forced to reset the generated password on resource creation. Only applies on resource creation.
     */
    passwordResetRequired?: pulumi.Input<boolean>;
    /**
     * Either a base-64 encoded PGP public key, or a keybase username in the form `keybase:username`. Only applies on resource creation. Drift detection is not possible with this argument.
     */
    pgpKey?: pulumi.Input<string>;
    /**
     * The IAM user's name.
     */
    user?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a UserLoginProfile resource.
 */
export interface UserLoginProfileArgs {
    /**
     * The length of the generated password on resource creation. Only applies on resource creation. Drift detection is not possible with this argument. Default value is `20`.
     */
    passwordLength?: pulumi.Input<number>;
    /**
     * Whether the user should be forced to reset the generated password on resource creation. Only applies on resource creation.
     */
    passwordResetRequired?: pulumi.Input<boolean>;
    /**
     * Either a base-64 encoded PGP public key, or a keybase username in the form `keybase:username`. Only applies on resource creation. Drift detection is not possible with this argument.
     */
    pgpKey?: pulumi.Input<string>;
    /**
     * The IAM user's name.
     */
    user: pulumi.Input<string>;
}
