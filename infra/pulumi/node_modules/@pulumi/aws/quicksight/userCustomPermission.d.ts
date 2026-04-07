import * as pulumi from "@pulumi/pulumi";
/**
 * Manages the custom permissions profile for a user.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.quicksight.UserCustomPermission("example", {
 *     userName: exampleAwsQuicksightUser.userName,
 *     customPermissionsName: exampleAwsQuicksightCustomPermissions.customPermissionsName,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import QuickSight user custom permissions using a comma-delimited string combining the `aws_account_id`, `namespace`, and `user_name`. For example:
 *
 * ```sh
 * $ pulumi import aws:quicksight/userCustomPermission:UserCustomPermission example 012345678901,default,user1
 * ```
 */
export declare class UserCustomPermission extends pulumi.CustomResource {
    /**
     * Get an existing UserCustomPermission resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: UserCustomPermissionState, opts?: pulumi.CustomResourceOptions): UserCustomPermission;
    /**
     * Returns true if the given object is an instance of UserCustomPermission.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is UserCustomPermission;
    readonly awsAccountId: pulumi.Output<string>;
    /**
     * Custom permissions profile name.
     */
    readonly customPermissionsName: pulumi.Output<string>;
    /**
     * Namespace that the user belongs to. Defaults to `default`.
     */
    readonly namespace: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Username of the user.
     *
     * The following arguments are optional:
     */
    readonly userName: pulumi.Output<string>;
    /**
     * Create a UserCustomPermission resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: UserCustomPermissionArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering UserCustomPermission resources.
 */
export interface UserCustomPermissionState {
    awsAccountId?: pulumi.Input<string>;
    /**
     * Custom permissions profile name.
     */
    customPermissionsName?: pulumi.Input<string>;
    /**
     * Namespace that the user belongs to. Defaults to `default`.
     */
    namespace?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Username of the user.
     *
     * The following arguments are optional:
     */
    userName?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a UserCustomPermission resource.
 */
export interface UserCustomPermissionArgs {
    awsAccountId?: pulumi.Input<string>;
    /**
     * Custom permissions profile name.
     */
    customPermissionsName: pulumi.Input<string>;
    /**
     * Namespace that the user belongs to. Defaults to `default`.
     */
    namespace?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Username of the user.
     *
     * The following arguments are optional:
     */
    userName: pulumi.Input<string>;
}
