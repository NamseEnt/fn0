import * as pulumi from "@pulumi/pulumi";
/**
 * Provides an AWS Network Firewall Resource Policy Resource for a rule group or firewall policy.
 *
 * ## Example Usage
 *
 * ### For a Firewall Policy resource
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.networkfirewall.ResourcePolicy("example", {
 *     resourceArn: exampleAwsNetworkfirewallFirewallPolicy.arn,
 *     policy: JSON.stringify({
 *         Statement: [{
 *             Action: [
 *                 "network-firewall:ListFirewallPolicies",
 *                 "network-firewall:CreateFirewall",
 *                 "network-firewall:UpdateFirewall",
 *                 "network-firewall:AssociateFirewallPolicy",
 *             ],
 *             Effect: "Allow",
 *             Resource: exampleAwsNetworkfirewallFirewallPolicy.arn,
 *             Principal: {
 *                 AWS: "arn:aws:iam::123456789012:root",
 *             },
 *         }],
 *         Version: "2012-10-17",
 *     }),
 * });
 * ```
 *
 * ### For a Rule Group resource
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.networkfirewall.ResourcePolicy("example", {
 *     resourceArn: exampleAwsNetworkfirewallRuleGroup.arn,
 *     policy: JSON.stringify({
 *         Statement: [{
 *             Action: [
 *                 "network-firewall:ListRuleGroups",
 *                 "network-firewall:CreateFirewallPolicy",
 *                 "network-firewall:UpdateFirewallPolicy",
 *             ],
 *             Effect: "Allow",
 *             Resource: exampleAwsNetworkfirewallRuleGroup.arn,
 *             Principal: {
 *                 AWS: "arn:aws:iam::123456789012:root",
 *             },
 *         }],
 *         Version: "2012-10-17",
 *     }),
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Network Firewall Resource Policies using the `resource arn`. For example:
 * ```sh
 * $ pulumi import aws:networkfirewall/resourcePolicy:ResourcePolicy example arn:aws:network-firewall:us-west-1:123456789012:stateful-rulegroup/example
 * ```
 */
export declare class ResourcePolicy extends pulumi.CustomResource {
    /**
     * Get an existing ResourcePolicy resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ResourcePolicyState, opts?: pulumi.CustomResourceOptions): ResourcePolicy;
    /**
     * Returns true if the given object is an instance of ResourcePolicy.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ResourcePolicy;
    /**
     * JSON formatted policy document that controls access to the Network Firewall resource. The policy must be provided **without whitespaces**.  We recommend using jsonencode for formatting as seen in the examples above. For more details, including available policy statement Actions, see the [Policy](https://docs.aws.amazon.com/network-firewall/latest/APIReference/API_PutResourcePolicy.html#API_PutResourcePolicy_RequestSyntax) parameter in the AWS API documentation.
     */
    readonly policy: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The Amazon Resource Name (ARN) of the rule group or firewall policy.
     */
    readonly resourceArn: pulumi.Output<string>;
    /**
     * Create a ResourcePolicy resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ResourcePolicyArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ResourcePolicy resources.
 */
export interface ResourcePolicyState {
    /**
     * JSON formatted policy document that controls access to the Network Firewall resource. The policy must be provided **without whitespaces**.  We recommend using jsonencode for formatting as seen in the examples above. For more details, including available policy statement Actions, see the [Policy](https://docs.aws.amazon.com/network-firewall/latest/APIReference/API_PutResourcePolicy.html#API_PutResourcePolicy_RequestSyntax) parameter in the AWS API documentation.
     */
    policy?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) of the rule group or firewall policy.
     */
    resourceArn?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a ResourcePolicy resource.
 */
export interface ResourcePolicyArgs {
    /**
     * JSON formatted policy document that controls access to the Network Firewall resource. The policy must be provided **without whitespaces**.  We recommend using jsonencode for formatting as seen in the examples above. For more details, including available policy statement Actions, see the [Policy](https://docs.aws.amazon.com/network-firewall/latest/APIReference/API_PutResourcePolicy.html#API_PutResourcePolicy_RequestSyntax) parameter in the AWS API documentation.
     */
    policy: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) of the rule group or firewall policy.
     */
    resourceArn: pulumi.Input<string>;
}
