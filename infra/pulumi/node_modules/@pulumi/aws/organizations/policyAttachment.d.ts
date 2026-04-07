import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a resource to attach an AWS Organizations policy to an organization account, root, or unit.
 *
 * ## Example Usage
 *
 * ### Organization Account
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const account = new aws.organizations.PolicyAttachment("account", {
 *     policyId: example.id,
 *     targetId: "123456789012",
 * });
 * ```
 *
 * ### Organization Root
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const root = new aws.organizations.PolicyAttachment("root", {
 *     policyId: example.id,
 *     targetId: exampleAwsOrganizationsOrganization.roots[0].id,
 * });
 * ```
 *
 * ### Organization Unit
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const unit = new aws.organizations.PolicyAttachment("unit", {
 *     policyId: example.id,
 *     targetId: exampleAwsOrganizationsOrganizationalUnit.id,
 * });
 * ```
 *
 * ## Import
 *
 * ### Identity Schema
 *
 * #### Required
 *
 * * `policy_id` (String) Organizations policy ID.
 *
 * * `target_id` (String) Organizations target ID (account, OU, or root).
 *
 * #### Optional
 *
 * * `account_id` (String) AWS Account where this resource is managed.
 *
 * Using `pulumi import`, import `aws_organizations_policy_attachment` using the target ID and policy ID. For example:
 *
 * With an account target:
 *
 * console
 *
 * % pulumi import aws_organizations_policy_attachment.example 123456789012:p-12345678
 */
export declare class PolicyAttachment extends pulumi.CustomResource {
    /**
     * Get an existing PolicyAttachment resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: PolicyAttachmentState, opts?: pulumi.CustomResourceOptions): PolicyAttachment;
    /**
     * Returns true if the given object is an instance of PolicyAttachment.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is PolicyAttachment;
    /**
     * The unique identifier (ID) of the policy that you want to attach to the target.
     */
    readonly policyId: pulumi.Output<string>;
    /**
     * If set to `true`, destroy will **not** detach the policy and instead just remove the resource from state. This can be useful in situations where the attachment must be preserved to meet the AWS minimum requirement of 1 attached policy.
     */
    readonly skipDestroy: pulumi.Output<boolean | undefined>;
    /**
     * The unique identifier (ID) of the root, organizational unit, or account number that you want to attach the policy to.
     */
    readonly targetId: pulumi.Output<string>;
    /**
     * Create a PolicyAttachment resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: PolicyAttachmentArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering PolicyAttachment resources.
 */
export interface PolicyAttachmentState {
    /**
     * The unique identifier (ID) of the policy that you want to attach to the target.
     */
    policyId?: pulumi.Input<string>;
    /**
     * If set to `true`, destroy will **not** detach the policy and instead just remove the resource from state. This can be useful in situations where the attachment must be preserved to meet the AWS minimum requirement of 1 attached policy.
     */
    skipDestroy?: pulumi.Input<boolean>;
    /**
     * The unique identifier (ID) of the root, organizational unit, or account number that you want to attach the policy to.
     */
    targetId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a PolicyAttachment resource.
 */
export interface PolicyAttachmentArgs {
    /**
     * The unique identifier (ID) of the policy that you want to attach to the target.
     */
    policyId: pulumi.Input<string>;
    /**
     * If set to `true`, destroy will **not** detach the policy and instead just remove the resource from state. This can be useful in situations where the attachment must be preserved to meet the AWS minimum requirement of 1 attached policy.
     */
    skipDestroy?: pulumi.Input<boolean>;
    /**
     * The unique identifier (ID) of the root, organizational unit, or account number that you want to attach the policy to.
     */
    targetId: pulumi.Input<string>;
}
