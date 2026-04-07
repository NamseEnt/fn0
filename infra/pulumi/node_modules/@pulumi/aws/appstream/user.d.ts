import * as pulumi from "@pulumi/pulumi";
/**
 * Provides an AppStream user.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.appstream.User("example", {
 *     authenticationType: "USERPOOL",
 *     userName: "EMAIL",
 *     firstName: "FIRST NAME",
 *     lastName: "LAST NAME",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_appstream_user` using the `user_name` and `authentication_type` separated by a slash (`/`). For example:
 *
 * ```sh
 * $ pulumi import aws:appstream/user:User example UserName/AuthenticationType
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
     * ARN of the appstream user.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Authentication type for the user. You must specify USERPOOL. Valid values: `API`, `SAML`, `USERPOOL`
     */
    readonly authenticationType: pulumi.Output<string>;
    /**
     * Date and time, in UTC and extended RFC 3339 format, when the user was created.
     */
    readonly createdTime: pulumi.Output<string>;
    /**
     * Whether the user in the user pool is enabled.
     */
    readonly enabled: pulumi.Output<boolean | undefined>;
    /**
     * First name, or given name, of the user.
     */
    readonly firstName: pulumi.Output<string | undefined>;
    /**
     * Last name, or surname, of the user.
     */
    readonly lastName: pulumi.Output<string | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Send an email notification.
     */
    readonly sendEmailNotification: pulumi.Output<boolean | undefined>;
    /**
     * Email address of the user.
     *
     * The following arguments are optional:
     */
    readonly userName: pulumi.Output<string>;
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
     * ARN of the appstream user.
     */
    arn?: pulumi.Input<string>;
    /**
     * Authentication type for the user. You must specify USERPOOL. Valid values: `API`, `SAML`, `USERPOOL`
     */
    authenticationType?: pulumi.Input<string>;
    /**
     * Date and time, in UTC and extended RFC 3339 format, when the user was created.
     */
    createdTime?: pulumi.Input<string>;
    /**
     * Whether the user in the user pool is enabled.
     */
    enabled?: pulumi.Input<boolean>;
    /**
     * First name, or given name, of the user.
     */
    firstName?: pulumi.Input<string>;
    /**
     * Last name, or surname, of the user.
     */
    lastName?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Send an email notification.
     */
    sendEmailNotification?: pulumi.Input<boolean>;
    /**
     * Email address of the user.
     *
     * The following arguments are optional:
     */
    userName?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a User resource.
 */
export interface UserArgs {
    /**
     * Authentication type for the user. You must specify USERPOOL. Valid values: `API`, `SAML`, `USERPOOL`
     */
    authenticationType: pulumi.Input<string>;
    /**
     * Whether the user in the user pool is enabled.
     */
    enabled?: pulumi.Input<boolean>;
    /**
     * First name, or given name, of the user.
     */
    firstName?: pulumi.Input<string>;
    /**
     * Last name, or surname, of the user.
     */
    lastName?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Send an email notification.
     */
    sendEmailNotification?: pulumi.Input<boolean>;
    /**
     * Email address of the user.
     *
     * The following arguments are optional:
     */
    userName: pulumi.Input<string>;
}
