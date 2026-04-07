import * as pulumi from "@pulumi/pulumi";
/**
 * > **NOTE:**: To reliably detect drift between customer managed policies listed in this resource and actual policies attached to the role in the cloud, you currently need to run Pulumi with `pulumi up --refresh`. See [#4766](https://github.com/pulumi/pulumi-aws/issues/4766) for tracking making this work with regular `pulumi up`
 *
 * Resource for maintaining exclusive management of managed IAM policies assigned to an AWS IAM (Identity & Access Management) role.
 *
 * !> This resource takes exclusive ownership over managed IAM policies attached to a role. This includes removal of managed IAM policies which are not explicitly configured. To prevent persistent drift, ensure any `aws.iam.RolePolicyAttachment` resources managed alongside this resource are included in the `policyArns` argument.
 *
 * > Destruction of this resource means Pulumi will no longer manage reconciliation of the configured policy attachments. It **will not** detach the configured policies from the role.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.iam.RolePolicyAttachmentsExclusive("example", {
 *     roleName: exampleAwsIamRole.name,
 *     policyArns: [exampleAwsIamPolicy.arn],
 * });
 * ```
 *
 * ### Disallow Managed IAM Policies
 *
 * To automatically remove any configured managed IAM policies, set the `policyArns` argument to an empty list.
 *
 * > This will not **prevent** managed IAM policies from being assigned to a role via Pulumi (or any other interface). This resource enables bringing managed IAM policy assignments into a configured state, however, this reconciliation happens only when `apply` is proactively run.
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.iam.RolePolicyAttachmentsExclusive("example", {
 *     roleName: exampleAwsIamRole.name,
 *     policyArns: [],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import exclusive management of managed IAM policy assignments using the `role_name`. For example:
 *
 * ```sh
 * $ pulumi import aws:iam/rolePolicyAttachmentsExclusive:RolePolicyAttachmentsExclusive example MyRole
 * ```
 */
export declare class RolePolicyAttachmentsExclusive extends pulumi.CustomResource {
    /**
     * Get an existing RolePolicyAttachmentsExclusive resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: RolePolicyAttachmentsExclusiveState, opts?: pulumi.CustomResourceOptions): RolePolicyAttachmentsExclusive;
    /**
     * Returns true if the given object is an instance of RolePolicyAttachmentsExclusive.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is RolePolicyAttachmentsExclusive;
    /**
     * A list of managed IAM policy ARNs to be attached to the role. Policies attached to this role but not configured in this argument will be removed.
     */
    readonly policyArns: pulumi.Output<string[]>;
    /**
     * IAM role name.
     */
    readonly roleName: pulumi.Output<string>;
    /**
     * Create a RolePolicyAttachmentsExclusive resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: RolePolicyAttachmentsExclusiveArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering RolePolicyAttachmentsExclusive resources.
 */
export interface RolePolicyAttachmentsExclusiveState {
    /**
     * A list of managed IAM policy ARNs to be attached to the role. Policies attached to this role but not configured in this argument will be removed.
     */
    policyArns?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * IAM role name.
     */
    roleName?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a RolePolicyAttachmentsExclusive resource.
 */
export interface RolePolicyAttachmentsExclusiveArgs {
    /**
     * A list of managed IAM policy ARNs to be attached to the role. Policies attached to this role but not configured in this argument will be removed.
     */
    policyArns: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * IAM role name.
     */
    roleName: pulumi.Input<string>;
}
