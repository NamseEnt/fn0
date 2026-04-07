import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a Cognito User Group resource.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const main = new aws.cognito.UserPool("main", {name: "identity pool"});
 * const groupRole = aws.iam.getPolicyDocument({
 *     statements: [{
 *         effect: "Allow",
 *         principals: [{
 *             type: "Federated",
 *             identifiers: ["cognito-identity.amazonaws.com"],
 *         }],
 *         actions: ["sts:AssumeRoleWithWebIdentity"],
 *         conditions: [
 *             {
 *                 test: "StringEquals",
 *                 variable: "cognito-identity.amazonaws.com:aud",
 *                 values: ["us-east-1:12345678-dead-beef-cafe-123456790ab"],
 *             },
 *             {
 *                 test: "ForAnyValue:StringLike",
 *                 variable: "cognito-identity.amazonaws.com:amr",
 *                 values: ["authenticated"],
 *             },
 *         ],
 *     }],
 * });
 * const groupRoleRole = new aws.iam.Role("group_role", {
 *     name: "user-group-role",
 *     assumeRolePolicy: groupRole.then(groupRole => groupRole.json),
 * });
 * const mainUserGroup = new aws.cognito.UserGroup("main", {
 *     name: "user-group",
 *     userPoolId: main.id,
 *     description: "Managed by Pulumi",
 *     precedence: 42,
 *     roleArn: groupRoleRole.arn,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Cognito User Groups using the `user_pool_id`/`name` attributes concatenated. For example:
 *
 * ```sh
 * $ pulumi import aws:cognito/userGroup:UserGroup group us-east-1_vG78M4goG/user-group
 * ```
 */
export declare class UserGroup extends pulumi.CustomResource {
    /**
     * Get an existing UserGroup resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: UserGroupState, opts?: pulumi.CustomResourceOptions): UserGroup;
    /**
     * Returns true if the given object is an instance of UserGroup.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is UserGroup;
    /**
     * The description of the user group.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * The name of the user group.
     */
    readonly name: pulumi.Output<string>;
    /**
     * The precedence of the user group.
     */
    readonly precedence: pulumi.Output<number | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The ARN of the IAM role to be associated with the user group.
     */
    readonly roleArn: pulumi.Output<string | undefined>;
    /**
     * The user pool ID.
     */
    readonly userPoolId: pulumi.Output<string>;
    /**
     * Create a UserGroup resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: UserGroupArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering UserGroup resources.
 */
export interface UserGroupState {
    /**
     * The description of the user group.
     */
    description?: pulumi.Input<string>;
    /**
     * The name of the user group.
     */
    name?: pulumi.Input<string>;
    /**
     * The precedence of the user group.
     */
    precedence?: pulumi.Input<number>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The ARN of the IAM role to be associated with the user group.
     */
    roleArn?: pulumi.Input<string>;
    /**
     * The user pool ID.
     */
    userPoolId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a UserGroup resource.
 */
export interface UserGroupArgs {
    /**
     * The description of the user group.
     */
    description?: pulumi.Input<string>;
    /**
     * The name of the user group.
     */
    name?: pulumi.Input<string>;
    /**
     * The precedence of the user group.
     */
    precedence?: pulumi.Input<number>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The ARN of the IAM role to be associated with the user group.
     */
    roleArn?: pulumi.Input<string>;
    /**
     * The user pool ID.
     */
    userPoolId: pulumi.Input<string>;
}
