import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
/**
 * Provides an IAM policy.
 *
 * > **NOTE:** We suggest using explicit JSON encoding or `aws.iam.getPolicyDocument` when assigning a value to `policy`. They seamlessly translate configuration to JSON, enabling you to maintain consistency within your configuration without the need for context switches. Also, you can sidestep potential complications arising from formatting discrepancies, whitespace inconsistencies, and other nuances inherent to JSON.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const policy = new aws.iam.Policy("policy", {
 *     name: "test_policy",
 *     path: "/",
 *     description: "My test policy",
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
 * - `arn` (String) Amazon Resource Name (ARN) of the IAM policy.
 *
 * Using `pulumi import`, import IAM Policies using the `arn`. For example:
 *
 * console
 *
 * % pulumi import aws_iam_policy.administrator arn:aws:iam::123456789012:policy/UsersManageOwnCredentials
 */
export declare class Policy extends pulumi.CustomResource {
    /**
     * Get an existing Policy resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: PolicyState, opts?: pulumi.CustomResourceOptions): Policy;
    /**
     * Returns true if the given object is an instance of Policy.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Policy;
    /**
     * ARN assigned by AWS to this policy.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Number of entities (users, groups, and roles) that the policy is attached to.
     */
    readonly attachmentCount: pulumi.Output<number>;
    /**
     * Description of the IAM policy.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * Name of the policy. If omitted, the provider will assign a random, unique name.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Creates a unique name beginning with the specified prefix. Conflicts with `name`.
     */
    readonly namePrefix: pulumi.Output<string>;
    /**
     * Path in which to create the policy. See [IAM Identifiers](https://docs.aws.amazon.com/IAM/latest/UserGuide/Using_Identifiers.html) for more information.
     */
    readonly path: pulumi.Output<string | undefined>;
    /**
     * Policy document. This is a JSON formatted string. For more information about building AWS IAM policy documents, see the AWS IAM Policy Document Guide
     */
    readonly policy: pulumi.Output<string>;
    /**
     * Policy's ID.
     */
    readonly policyId: pulumi.Output<string>;
    /**
     * Map of resource tags for the IAM Policy. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * A map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * Create a Policy resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: PolicyArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Policy resources.
 */
export interface PolicyState {
    /**
     * ARN assigned by AWS to this policy.
     */
    arn?: pulumi.Input<string>;
    /**
     * Number of entities (users, groups, and roles) that the policy is attached to.
     */
    attachmentCount?: pulumi.Input<number>;
    /**
     * Description of the IAM policy.
     */
    description?: pulumi.Input<string>;
    /**
     * Name of the policy. If omitted, the provider will assign a random, unique name.
     */
    name?: pulumi.Input<string>;
    /**
     * Creates a unique name beginning with the specified prefix. Conflicts with `name`.
     */
    namePrefix?: pulumi.Input<string>;
    /**
     * Path in which to create the policy. See [IAM Identifiers](https://docs.aws.amazon.com/IAM/latest/UserGuide/Using_Identifiers.html) for more information.
     */
    path?: pulumi.Input<string>;
    /**
     * Policy document. This is a JSON formatted string. For more information about building AWS IAM policy documents, see the AWS IAM Policy Document Guide
     */
    policy?: pulumi.Input<string | inputs.iam.PolicyDocument>;
    /**
     * Policy's ID.
     */
    policyId?: pulumi.Input<string>;
    /**
     * Map of resource tags for the IAM Policy. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * A map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
/**
 * The set of arguments for constructing a Policy resource.
 */
export interface PolicyArgs {
    /**
     * Description of the IAM policy.
     */
    description?: pulumi.Input<string>;
    /**
     * Name of the policy. If omitted, the provider will assign a random, unique name.
     */
    name?: pulumi.Input<string>;
    /**
     * Creates a unique name beginning with the specified prefix. Conflicts with `name`.
     */
    namePrefix?: pulumi.Input<string>;
    /**
     * Path in which to create the policy. See [IAM Identifiers](https://docs.aws.amazon.com/IAM/latest/UserGuide/Using_Identifiers.html) for more information.
     */
    path?: pulumi.Input<string>;
    /**
     * Policy document. This is a JSON formatted string. For more information about building AWS IAM policy documents, see the AWS IAM Policy Document Guide
     */
    policy: pulumi.Input<string | inputs.iam.PolicyDocument>;
    /**
     * Map of resource tags for the IAM Policy. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
