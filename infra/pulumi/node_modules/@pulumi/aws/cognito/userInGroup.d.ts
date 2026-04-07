import * as pulumi from "@pulumi/pulumi";
/**
 * Adds the specified user to the specified group.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.cognito.UserPool("example", {
 *     name: "example",
 *     passwordPolicy: {
 *         temporaryPasswordValidityDays: 7,
 *         minimumLength: 6,
 *         requireUppercase: false,
 *         requireSymbols: false,
 *         requireNumbers: false,
 *     },
 * });
 * const exampleUser = new aws.cognito.User("example", {
 *     userPoolId: example.id,
 *     username: "example",
 * });
 * const exampleUserGroup = new aws.cognito.UserGroup("example", {
 *     userPoolId: example.id,
 *     name: "example",
 * });
 * const exampleUserInGroup = new aws.cognito.UserInGroup("example", {
 *     userPoolId: example.id,
 *     groupName: exampleUserGroup.name,
 *     username: exampleUser.username,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import a Cognito Group User using a comma-delimited string concatenating the `user_pool_id`, `group_name`, and `username` arguments. For example:
 *
 * ```sh
 * $ pulumi import aws:cognito/userInGroup:UserInGroup example us-east-1_vG78M4goG,example-group,example-user
 * ```
 */
export declare class UserInGroup extends pulumi.CustomResource {
    /**
     * Get an existing UserInGroup resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: UserInGroupState, opts?: pulumi.CustomResourceOptions): UserInGroup;
    /**
     * Returns true if the given object is an instance of UserInGroup.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is UserInGroup;
    /**
     * The name of the group to which the user is to be added.
     */
    readonly groupName: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The user pool ID of the user and group.
     */
    readonly userPoolId: pulumi.Output<string>;
    /**
     * The username of the user to be added to the group.
     */
    readonly username: pulumi.Output<string>;
    /**
     * Create a UserInGroup resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: UserInGroupArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering UserInGroup resources.
 */
export interface UserInGroupState {
    /**
     * The name of the group to which the user is to be added.
     */
    groupName?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The user pool ID of the user and group.
     */
    userPoolId?: pulumi.Input<string>;
    /**
     * The username of the user to be added to the group.
     */
    username?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a UserInGroup resource.
 */
export interface UserInGroupArgs {
    /**
     * The name of the group to which the user is to be added.
     */
    groupName: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The user pool ID of the user and group.
     */
    userPoolId: pulumi.Input<string>;
    /**
     * The username of the user to be added to the group.
     */
    username: pulumi.Input<string>;
}
