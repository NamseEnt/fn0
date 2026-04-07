import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import { Role } from "./index";
/**
 * Provides an IAM role inline policy.
 *
 * > **NOTE:** For a given role, this resource is incompatible with using the `aws.iam.Role` resource `inlinePolicy` argument. When using that argument and this resource, both will attempt to manage the role's inline policies and the provider will show a permanent difference.
 *
 * > **NOTE:** We suggest using explicit JSON encoding or `aws.iam.getPolicyDocument` when assigning a value to `policy`. They seamlessly translate configuration to JSON, enabling you to maintain consistency within your configuration without the need for context switches. Also, you can sidestep potential complications arising from formatting discrepancies, whitespace inconsistencies, and other nuances inherent to JSON.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const testRole = new aws.iam.Role("test_role", {
 *     name: "test_role",
 *     assumeRolePolicy: JSON.stringify({
 *         Version: "2012-10-17",
 *         Statement: [{
 *             Action: "sts:AssumeRole",
 *             Effect: "Allow",
 *             Sid: "",
 *             Principal: {
 *                 Service: "ec2.amazonaws.com",
 *             },
 *         }],
 *     }),
 * });
 * const testPolicy = new aws.iam.RolePolicy("test_policy", {
 *     name: "test_policy",
 *     role: testRole.id,
 *     policy: JSON.stringify({
 *         Version: "2012-10-17",
 *         Statement: [{
 *             Action: ["ec2:Describe*"],
 *             Effect: "Allow",
 *             Resource: "*",
 *         }],
 *     }),
 * });
 * ```
 *
 * ## Import
 *
 * ### Identity Schema
 *
 * #### Required
 *
 * * `role` (String) Name of the IAM role.
 *
 * * `name` (String) Name of the role policy.
 *
 * #### Optional
 *
 * * `account_id` (String) AWS Account where this resource is managed.
 *
 * Using `pulumi import`, import IAM Role Policies using the `role_name:role_policy_name`. For example:
 *
 * console
 *
 * % pulumi import aws_iam_role_policy.example role_of_mypolicy_name:mypolicy_name
 */
export declare class RolePolicy extends pulumi.CustomResource {
    /**
     * Get an existing RolePolicy resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: RolePolicyState, opts?: pulumi.CustomResourceOptions): RolePolicy;
    /**
     * Returns true if the given object is an instance of RolePolicy.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is RolePolicy;
    /**
     * The name of the role policy.
     * If omitted, the provider will assign a random, unique name.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Creates a unique name beginning with the specified prefix.
     * Conflicts with `name`.
     */
    readonly namePrefix: pulumi.Output<string>;
    /**
     * The inline policy document.
     * This is a JSON formatted string.
     * For more information about building IAM policy documents with Pulumi, see the AWS IAM Policy Document Guide
     */
    readonly policy: pulumi.Output<string>;
    /**
     * The name of the IAM role to attach to the policy.
     */
    readonly role: pulumi.Output<string>;
    /**
     * Create a RolePolicy resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: RolePolicyArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering RolePolicy resources.
 */
export interface RolePolicyState {
    /**
     * The name of the role policy.
     * If omitted, the provider will assign a random, unique name.
     */
    name?: pulumi.Input<string>;
    /**
     * Creates a unique name beginning with the specified prefix.
     * Conflicts with `name`.
     */
    namePrefix?: pulumi.Input<string>;
    /**
     * The inline policy document.
     * This is a JSON formatted string.
     * For more information about building IAM policy documents with Pulumi, see the AWS IAM Policy Document Guide
     */
    policy?: pulumi.Input<string | inputs.iam.PolicyDocument>;
    /**
     * The name of the IAM role to attach to the policy.
     */
    role?: pulumi.Input<string | Role>;
}
/**
 * The set of arguments for constructing a RolePolicy resource.
 */
export interface RolePolicyArgs {
    /**
     * The name of the role policy.
     * If omitted, the provider will assign a random, unique name.
     */
    name?: pulumi.Input<string>;
    /**
     * Creates a unique name beginning with the specified prefix.
     * Conflicts with `name`.
     */
    namePrefix?: pulumi.Input<string>;
    /**
     * The inline policy document.
     * This is a JSON formatted string.
     * For more information about building IAM policy documents with Pulumi, see the AWS IAM Policy Document Guide
     */
    policy: pulumi.Input<string | inputs.iam.PolicyDocument>;
    /**
     * The name of the IAM role to attach to the policy.
     */
    role: pulumi.Input<string | Role>;
}
