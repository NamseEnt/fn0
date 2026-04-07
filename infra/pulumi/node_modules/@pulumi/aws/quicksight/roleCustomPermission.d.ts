import * as pulumi from "@pulumi/pulumi";
/**
 * Manages the custom permissions that are associated with a role.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.quicksight.RoleCustomPermission("example", {
 *     role: "READER",
 *     customPermissionsName: exampleAwsQuicksightCustomPermissions.customPermissionsName,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import QuickSight role custom permissions using a comma-delimited string combining the `aws_account_id`, `namespace`, and `role`. For example:
 *
 * ```sh
 * $ pulumi import aws:quicksight/roleCustomPermission:RoleCustomPermission example 012345678901,default,READER
 * ```
 */
export declare class RoleCustomPermission extends pulumi.CustomResource {
    /**
     * Get an existing RoleCustomPermission resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: RoleCustomPermissionState, opts?: pulumi.CustomResourceOptions): RoleCustomPermission;
    /**
     * Returns true if the given object is an instance of RoleCustomPermission.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is RoleCustomPermission;
    readonly awsAccountId: pulumi.Output<string>;
    /**
     * Custom permissions profile name.
     */
    readonly customPermissionsName: pulumi.Output<string>;
    /**
     * Namespace containing the role. Defaults to `default`.
     */
    readonly namespace: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Role. Valid values are `ADMIN`, `AUTHOR`, `READER`, `ADMIN_PRO`, `AUTHOR_PRO`, and `READER_PRO`.
     *
     * The following arguments are optional:
     */
    readonly role: pulumi.Output<string>;
    /**
     * Create a RoleCustomPermission resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: RoleCustomPermissionArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering RoleCustomPermission resources.
 */
export interface RoleCustomPermissionState {
    awsAccountId?: pulumi.Input<string>;
    /**
     * Custom permissions profile name.
     */
    customPermissionsName?: pulumi.Input<string>;
    /**
     * Namespace containing the role. Defaults to `default`.
     */
    namespace?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Role. Valid values are `ADMIN`, `AUTHOR`, `READER`, `ADMIN_PRO`, `AUTHOR_PRO`, and `READER_PRO`.
     *
     * The following arguments are optional:
     */
    role?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a RoleCustomPermission resource.
 */
export interface RoleCustomPermissionArgs {
    awsAccountId?: pulumi.Input<string>;
    /**
     * Custom permissions profile name.
     */
    customPermissionsName: pulumi.Input<string>;
    /**
     * Namespace containing the role. Defaults to `default`.
     */
    namespace?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Role. Valid values are `ADMIN`, `AUTHOR`, `READER`, `ADMIN_PRO`, `AUTHOR_PRO`, and `READER_PRO`.
     *
     * The following arguments are optional:
     */
    role: pulumi.Input<string>;
}
