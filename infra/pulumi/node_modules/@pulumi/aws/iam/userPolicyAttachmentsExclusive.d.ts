import * as pulumi from "@pulumi/pulumi";
/**
 * ## Import
 *
 * Using `pulumi import`, import exclusive management of managed IAM policy assignments using the `user_name`. For example:
 *
 * ```sh
 * $ pulumi import aws:iam/userPolicyAttachmentsExclusive:UserPolicyAttachmentsExclusive example MyUser
 * ```
 */
export declare class UserPolicyAttachmentsExclusive extends pulumi.CustomResource {
    /**
     * Get an existing UserPolicyAttachmentsExclusive resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: UserPolicyAttachmentsExclusiveState, opts?: pulumi.CustomResourceOptions): UserPolicyAttachmentsExclusive;
    /**
     * Returns true if the given object is an instance of UserPolicyAttachmentsExclusive.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is UserPolicyAttachmentsExclusive;
    /**
     * A list of managed IAM policy ARNs to be attached to the user. Policies attached to this user but not configured in this argument will be removed.
     */
    readonly policyArns: pulumi.Output<string[]>;
    /**
     * IAM user name.
     */
    readonly userName: pulumi.Output<string>;
    /**
     * Create a UserPolicyAttachmentsExclusive resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: UserPolicyAttachmentsExclusiveArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering UserPolicyAttachmentsExclusive resources.
 */
export interface UserPolicyAttachmentsExclusiveState {
    /**
     * A list of managed IAM policy ARNs to be attached to the user. Policies attached to this user but not configured in this argument will be removed.
     */
    policyArns?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * IAM user name.
     */
    userName?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a UserPolicyAttachmentsExclusive resource.
 */
export interface UserPolicyAttachmentsExclusiveArgs {
    /**
     * A list of managed IAM policy ARNs to be attached to the user. Policies attached to this user but not configured in this argument will be removed.
     */
    policyArns: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * IAM user name.
     */
    userName: pulumi.Input<string>;
}
