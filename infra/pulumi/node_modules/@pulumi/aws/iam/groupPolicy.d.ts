import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
/**
 * Provides an IAM policy attached to a group.
 *
 * > **NOTE:** We suggest using explicit JSON encoding or `aws.iam.getPolicyDocument` when assigning a value to `policy`. They seamlessly translate configuration to JSON, enabling you to maintain consistency within your configuration without the need for context switches. Also, you can sidestep potential complications arising from formatting discrepancies, whitespace inconsistencies, and other nuances inherent to JSON.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const myDevelopers = new aws.iam.Group("my_developers", {
 *     name: "developers",
 *     path: "/users/",
 * });
 * const myDeveloperPolicy = new aws.iam.GroupPolicy("my_developer_policy", {
 *     name: "my_developer_policy",
 *     group: myDevelopers.name,
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
 * Using `pulumi import`, import IAM Group Policies using the `group_name:group_policy_name`. For example:
 *
 * ```sh
 * $ pulumi import aws:iam/groupPolicy:GroupPolicy mypolicy group_of_mypolicy_name:mypolicy_name
 * ```
 */
export declare class GroupPolicy extends pulumi.CustomResource {
    /**
     * Get an existing GroupPolicy resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: GroupPolicyState, opts?: pulumi.CustomResourceOptions): GroupPolicy;
    /**
     * Returns true if the given object is an instance of GroupPolicy.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is GroupPolicy;
    /**
     * The IAM group to attach to the policy.
     */
    readonly group: pulumi.Output<string>;
    /**
     * The name of the policy. If omitted, the provider will
     * assign a random, unique name.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Creates a unique name beginning with the specified
     * prefix. Conflicts with `name`.
     */
    readonly namePrefix: pulumi.Output<string>;
    /**
     * The policy document. This is a JSON formatted string.
     */
    readonly policy: pulumi.Output<string>;
    /**
     * Create a GroupPolicy resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: GroupPolicyArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering GroupPolicy resources.
 */
export interface GroupPolicyState {
    /**
     * The IAM group to attach to the policy.
     */
    group?: pulumi.Input<string>;
    /**
     * The name of the policy. If omitted, the provider will
     * assign a random, unique name.
     */
    name?: pulumi.Input<string>;
    /**
     * Creates a unique name beginning with the specified
     * prefix. Conflicts with `name`.
     */
    namePrefix?: pulumi.Input<string>;
    /**
     * The policy document. This is a JSON formatted string.
     */
    policy?: pulumi.Input<string | inputs.iam.PolicyDocument>;
}
/**
 * The set of arguments for constructing a GroupPolicy resource.
 */
export interface GroupPolicyArgs {
    /**
     * The IAM group to attach to the policy.
     */
    group: pulumi.Input<string>;
    /**
     * The name of the policy. If omitted, the provider will
     * assign a random, unique name.
     */
    name?: pulumi.Input<string>;
    /**
     * Creates a unique name beginning with the specified
     * prefix. Conflicts with `name`.
     */
    namePrefix?: pulumi.Input<string>;
    /**
     * The policy document. This is a JSON formatted string.
     */
    policy: pulumi.Input<string | inputs.iam.PolicyDocument>;
}
