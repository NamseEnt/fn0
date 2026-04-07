import * as pulumi from "@pulumi/pulumi";
/**
 * Resource for managing QuickSight User
 *
 * ## Example Usage
 *
 * ### Create User With IAM Identity Type Using an IAM Role
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.quicksight.User("example", {
 *     email: "author1@example.com",
 *     identityType: "IAM",
 *     userRole: "AUTHOR",
 *     iamArn: "arn:aws:iam::123456789012:role/AuthorRole",
 *     sessionName: "author1",
 * });
 * ```
 *
 * ### Create User With IAM Identity Type Using an IAM User
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.quicksight.User("example", {
 *     email: "authorpro1@example.com",
 *     identityType: "IAM",
 *     userRole: "AUTHOR_PRO",
 *     iamArn: "arn:aws:iam::123456789012:user/authorpro1",
 * });
 * ```
 *
 * ### Create User With QuickSight Identity Type in Non-Default Namespace
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.quicksight.User("example", {
 *     email: "reader1@example.com",
 *     identityType: "QUICKSIGHT",
 *     userRole: "READER",
 *     namespace: "example",
 *     userName: "reader1",
 * });
 * ```
 *
 * ## Import
 *
 * You cannot import this resource.
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
     * Amazon Resource Name (ARN) for the user.
     */
    readonly arn: pulumi.Output<string>;
    readonly awsAccountId: pulumi.Output<string>;
    /**
     * Email address of the user that you want to register.
     */
    readonly email: pulumi.Output<string>;
    /**
     * ARN of the IAM user or role that you are registering with Amazon QuickSight. Required only for users with an identity type of `IAM`.
     */
    readonly iamArn: pulumi.Output<string | undefined>;
    /**
     * Identity type that your Amazon QuickSight account uses to manage the identity of users. Valid values: `IAM`, `QUICKSIGHT`, `IAM_IDENTITY_CENTER`.
     */
    readonly identityType: pulumi.Output<string>;
    /**
     * The Amazon Quicksight namespace to create the user in. Defaults to `default`.
     */
    readonly namespace: pulumi.Output<string | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Name of the IAM session to use when assuming roles that can embed QuickSight dashboards. Only valid for registering users using an assumed IAM role. Additionally, if registering multiple users using the same IAM role, each user needs to have a unique session name.
     */
    readonly sessionName: pulumi.Output<string | undefined>;
    /**
     * URL the user visits to complete registration and provide a password. Returned only for users with an identity type of `QUICKSIGHT`.
     */
    readonly userInvitationUrl: pulumi.Output<string>;
    /**
     * Amazon QuickSight user name that you want to create for the user you are registering. Required only for users with an identity type of `QUICKSIGHT`.
     */
    readonly userName: pulumi.Output<string>;
    /**
     * Amazon QuickSight role for the user. Valid values: `READER`, `AUTHOR`, `ADMIN`, `READER_PRO`, `AUTHOR_PRO`, `ADMIN_PRO`, `RESTRICTED_AUTHOR`, `RESTRICTED_READER`.
     *
     * The following arguments are optional:
     */
    readonly userRole: pulumi.Output<string>;
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
     * Amazon Resource Name (ARN) for the user.
     */
    arn?: pulumi.Input<string>;
    awsAccountId?: pulumi.Input<string>;
    /**
     * Email address of the user that you want to register.
     */
    email?: pulumi.Input<string>;
    /**
     * ARN of the IAM user or role that you are registering with Amazon QuickSight. Required only for users with an identity type of `IAM`.
     */
    iamArn?: pulumi.Input<string>;
    /**
     * Identity type that your Amazon QuickSight account uses to manage the identity of users. Valid values: `IAM`, `QUICKSIGHT`, `IAM_IDENTITY_CENTER`.
     */
    identityType?: pulumi.Input<string>;
    /**
     * The Amazon Quicksight namespace to create the user in. Defaults to `default`.
     */
    namespace?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Name of the IAM session to use when assuming roles that can embed QuickSight dashboards. Only valid for registering users using an assumed IAM role. Additionally, if registering multiple users using the same IAM role, each user needs to have a unique session name.
     */
    sessionName?: pulumi.Input<string>;
    /**
     * URL the user visits to complete registration and provide a password. Returned only for users with an identity type of `QUICKSIGHT`.
     */
    userInvitationUrl?: pulumi.Input<string>;
    /**
     * Amazon QuickSight user name that you want to create for the user you are registering. Required only for users with an identity type of `QUICKSIGHT`.
     */
    userName?: pulumi.Input<string>;
    /**
     * Amazon QuickSight role for the user. Valid values: `READER`, `AUTHOR`, `ADMIN`, `READER_PRO`, `AUTHOR_PRO`, `ADMIN_PRO`, `RESTRICTED_AUTHOR`, `RESTRICTED_READER`.
     *
     * The following arguments are optional:
     */
    userRole?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a User resource.
 */
export interface UserArgs {
    awsAccountId?: pulumi.Input<string>;
    /**
     * Email address of the user that you want to register.
     */
    email: pulumi.Input<string>;
    /**
     * ARN of the IAM user or role that you are registering with Amazon QuickSight. Required only for users with an identity type of `IAM`.
     */
    iamArn?: pulumi.Input<string>;
    /**
     * Identity type that your Amazon QuickSight account uses to manage the identity of users. Valid values: `IAM`, `QUICKSIGHT`, `IAM_IDENTITY_CENTER`.
     */
    identityType: pulumi.Input<string>;
    /**
     * The Amazon Quicksight namespace to create the user in. Defaults to `default`.
     */
    namespace?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Name of the IAM session to use when assuming roles that can embed QuickSight dashboards. Only valid for registering users using an assumed IAM role. Additionally, if registering multiple users using the same IAM role, each user needs to have a unique session name.
     */
    sessionName?: pulumi.Input<string>;
    /**
     * Amazon QuickSight user name that you want to create for the user you are registering. Required only for users with an identity type of `QUICKSIGHT`.
     */
    userName?: pulumi.Input<string>;
    /**
     * Amazon QuickSight role for the user. Valid values: `READER`, `AUTHOR`, `ADMIN`, `READER_PRO`, `AUTHOR_PRO`, `ADMIN_PRO`, `RESTRICTED_AUTHOR`, `RESTRICTED_READER`.
     *
     * The following arguments are optional:
     */
    userRole: pulumi.Input<string>;
}
