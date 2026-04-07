import * as pulumi from "@pulumi/pulumi";
/**
 * ## Import
 *
 * Using `pulumi import`, import exclusive management of inline policy assignments using the `user_name`. For example:
 *
 * ```sh
 * $ pulumi import aws:iam/userPoliciesExclusive:UserPoliciesExclusive example MyUser
 * ```
 */
export declare class UserPoliciesExclusive extends pulumi.CustomResource {
    /**
     * Get an existing UserPoliciesExclusive resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: UserPoliciesExclusiveState, opts?: pulumi.CustomResourceOptions): UserPoliciesExclusive;
    /**
     * Returns true if the given object is an instance of UserPoliciesExclusive.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is UserPoliciesExclusive;
    /**
     * A list of inline policy names to be assigned to the user. Policies attached to this user but not configured in this argument will be removed.
     */
    readonly policyNames: pulumi.Output<string[]>;
    /**
     * IAM user name.
     */
    readonly userName: pulumi.Output<string>;
    /**
     * Create a UserPoliciesExclusive resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: UserPoliciesExclusiveArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering UserPoliciesExclusive resources.
 */
export interface UserPoliciesExclusiveState {
    /**
     * A list of inline policy names to be assigned to the user. Policies attached to this user but not configured in this argument will be removed.
     */
    policyNames?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * IAM user name.
     */
    userName?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a UserPoliciesExclusive resource.
 */
export interface UserPoliciesExclusiveArgs {
    /**
     * A list of inline policy names to be assigned to the user. Policies attached to this user but not configured in this argument will be removed.
     */
    policyNames: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * IAM user name.
     */
    userName: pulumi.Input<string>;
}
