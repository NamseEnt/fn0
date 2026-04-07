import * as pulumi from "@pulumi/pulumi";
/**
 * Resource for managing an AWS QuickSight Role Membership.
 *
 * > The role membership APIs are disabled for identities managed by QuickSight. This resource can only be used when the QuickSight account subscription uses the Active Directory or IAM Identity Center authentication method.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.quicksight.RoleMembership("example", {
 *     memberName: "example-group",
 *     role: "READER",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import QuickSight Role Membership using a comma-delimited string combining the `aws_account_id`, `namespace`, `role`, and `member_name`. For example:
 *
 * ```sh
 * $ pulumi import aws:quicksight/roleMembership:RoleMembership example 012345678901,default,READER,example-group
 * ```
 */
export declare class RoleMembership extends pulumi.CustomResource {
    /**
     * Get an existing RoleMembership resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: RoleMembershipState, opts?: pulumi.CustomResourceOptions): RoleMembership;
    /**
     * Returns true if the given object is an instance of RoleMembership.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is RoleMembership;
    readonly awsAccountId: pulumi.Output<string>;
    /**
     * Name of the group to be added to the role.
     */
    readonly memberName: pulumi.Output<string>;
    /**
     * Name of the namespace. Defaults to `default`.
     */
    readonly namespace: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Role to add the group to. Valid values are `ADMIN`, `AUTHOR`, `READER`, `ADMIN_PRO`, `AUTHOR_PRO`, and `READER_PRO`.
     *
     * The following arguments are optional:
     */
    readonly role: pulumi.Output<string>;
    /**
     * Create a RoleMembership resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: RoleMembershipArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering RoleMembership resources.
 */
export interface RoleMembershipState {
    awsAccountId?: pulumi.Input<string>;
    /**
     * Name of the group to be added to the role.
     */
    memberName?: pulumi.Input<string>;
    /**
     * Name of the namespace. Defaults to `default`.
     */
    namespace?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Role to add the group to. Valid values are `ADMIN`, `AUTHOR`, `READER`, `ADMIN_PRO`, `AUTHOR_PRO`, and `READER_PRO`.
     *
     * The following arguments are optional:
     */
    role?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a RoleMembership resource.
 */
export interface RoleMembershipArgs {
    awsAccountId?: pulumi.Input<string>;
    /**
     * Name of the group to be added to the role.
     */
    memberName: pulumi.Input<string>;
    /**
     * Name of the namespace. Defaults to `default`.
     */
    namespace?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Role to add the group to. Valid values are `ADMIN`, `AUTHOR`, `READER`, `ADMIN_PRO`, `AUTHOR_PRO`, and `READER_PRO`.
     *
     * The following arguments are optional:
     */
    role: pulumi.Input<string>;
}
