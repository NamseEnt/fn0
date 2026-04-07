import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides an AWS Network Firewall Firewall Policy Resource
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const current = aws.getRegion({});
 * const currentGetPartition = aws.getPartition({});
 * const currentGetCallerIdentity = aws.getCallerIdentity({});
 * const example = new aws.networkfirewall.FirewallPolicy("example", {
 *     name: "example",
 *     firewallPolicy: {
 *         statelessDefaultActions: ["aws:pass"],
 *         statelessFragmentDefaultActions: ["aws:drop"],
 *         statelessRuleGroupReferences: [{
 *             priority: 1,
 *             resourceArn: exampleAwsNetworkfirewallRuleGroup.arn,
 *         }],
 *         tlsInspectionConfigurationArn: Promise.all([currentGetPartition, current, currentGetCallerIdentity]).then(([currentGetPartition, current, currentGetCallerIdentity]) => `arn:${currentGetPartition.partition}:network-firewall:${current.region}:${currentGetCallerIdentity.accountId}:tls-configuration/example`),
 *     },
 *     tags: {
 *         Tag1: "Value1",
 *         Tag2: "Value2",
 *     },
 * });
 * ```
 *
 * ## Policy with a HOME_NET Override
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.networkfirewall.FirewallPolicy("example", {
 *     name: "example",
 *     firewallPolicy: {
 *         policyVariables: {
 *             ruleVariables: [{
 *                 key: "HOME_NET",
 *                 ipSet: {
 *                     definitions: [
 *                         "10.0.0.0/16",
 *                         "10.1.0.0/24",
 *                     ],
 *                 },
 *             }],
 *         },
 *         statelessDefaultActions: ["aws:pass"],
 *         statelessFragmentDefaultActions: ["aws:drop"],
 *         statelessRuleGroupReferences: [{
 *             priority: 1,
 *             resourceArn: exampleAwsNetworkfirewallRuleGroup.arn,
 *         }],
 *     },
 *     tags: {
 *         Tag1: "Value1",
 *         Tag2: "Value2",
 *     },
 * });
 * ```
 *
 * ## Policy with a Custom Action for Stateless Inspection
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.networkfirewall.FirewallPolicy("example", {
 *     name: "example",
 *     firewallPolicy: {
 *         statelessDefaultActions: [
 *             "aws:pass",
 *             "ExampleCustomAction",
 *         ],
 *         statelessFragmentDefaultActions: ["aws:drop"],
 *         statelessCustomActions: [{
 *             actionDefinition: {
 *                 publishMetricAction: {
 *                     dimensions: [{
 *                         value: "1",
 *                     }],
 *                 },
 *             },
 *             actionName: "ExampleCustomAction",
 *         }],
 *     },
 * });
 * ```
 *
 * ## Policy with Active Threat Defense in Action Order
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const current = aws.getRegion({});
 * const currentGetPartition = aws.getPartition({});
 * const example = new aws.networkfirewall.FirewallPolicy("example", {
 *     name: "example",
 *     firewallPolicy: {
 *         statelessFragmentDefaultActions: ["aws:drop"],
 *         statelessDefaultActions: ["aws:pass"],
 *         statefulRuleGroupReferences: [{
 *             deepThreatInspection: "true",
 *             resourceArn: Promise.all([currentGetPartition, current]).then(([currentGetPartition, current]) => `arn:${currentGetPartition.partition}:network-firewall:${current.region}:aws-managed:stateful-rulegroup/AttackInfrastructureActionOrder`),
 *         }],
 *     },
 * });
 * ```
 *
 * ## Policy with Active Threat Defense in Strict Order
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const current = aws.getRegion({});
 * const currentGetPartition = aws.getPartition({});
 * const example = new aws.networkfirewall.FirewallPolicy("example", {
 *     name: "example",
 *     firewallPolicy: {
 *         statelessFragmentDefaultActions: ["aws:drop"],
 *         statelessDefaultActions: ["aws:pass"],
 *         statefulEngineOptions: {
 *             ruleOrder: "STRICT_ORDER",
 *         },
 *         statefulRuleGroupReferences: [{
 *             deepThreatInspection: "false",
 *             priority: 1,
 *             resourceArn: Promise.all([currentGetPartition, current]).then(([currentGetPartition, current]) => `arn:${currentGetPartition.partition}:network-firewall:${current.region}:aws-managed:stateful-rulegroup/AttackInfrastructureStrictOrder`),
 *         }],
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Network Firewall Policies using their `arn`. For example:
 *
 * ```sh
 * $ pulumi import aws:networkfirewall/firewallPolicy:FirewallPolicy example arn:aws:network-firewall:us-west-1:123456789012:firewall-policy/example
 * ```
 */
export declare class FirewallPolicy extends pulumi.CustomResource {
    /**
     * Get an existing FirewallPolicy resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: FirewallPolicyState, opts?: pulumi.CustomResourceOptions): FirewallPolicy;
    /**
     * Returns true if the given object is an instance of FirewallPolicy.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is FirewallPolicy;
    /**
     * The Amazon Resource Name (ARN) that identifies the firewall policy.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * A friendly description of the firewall policy.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * KMS encryption configuration settings. See Encryption Configuration below for details.
     */
    readonly encryptionConfiguration: pulumi.Output<outputs.networkfirewall.FirewallPolicyEncryptionConfiguration | undefined>;
    /**
     * A configuration block describing the rule groups and policy actions to use in the firewall policy. See Firewall Policy below for details.
     */
    readonly firewallPolicy: pulumi.Output<outputs.networkfirewall.FirewallPolicyFirewallPolicy>;
    /**
     * A friendly name of the firewall policy.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Map of resource tags to associate with the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * A string token used when updating a firewall policy.
     */
    readonly updateToken: pulumi.Output<string>;
    /**
     * Create a FirewallPolicy resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: FirewallPolicyArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering FirewallPolicy resources.
 */
export interface FirewallPolicyState {
    /**
     * The Amazon Resource Name (ARN) that identifies the firewall policy.
     */
    arn?: pulumi.Input<string>;
    /**
     * A friendly description of the firewall policy.
     */
    description?: pulumi.Input<string>;
    /**
     * KMS encryption configuration settings. See Encryption Configuration below for details.
     */
    encryptionConfiguration?: pulumi.Input<inputs.networkfirewall.FirewallPolicyEncryptionConfiguration>;
    /**
     * A configuration block describing the rule groups and policy actions to use in the firewall policy. See Firewall Policy below for details.
     */
    firewallPolicy?: pulumi.Input<inputs.networkfirewall.FirewallPolicyFirewallPolicy>;
    /**
     * A friendly name of the firewall policy.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Map of resource tags to associate with the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
    /**
     * A string token used when updating a firewall policy.
     */
    updateToken?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a FirewallPolicy resource.
 */
export interface FirewallPolicyArgs {
    /**
     * A friendly description of the firewall policy.
     */
    description?: pulumi.Input<string>;
    /**
     * KMS encryption configuration settings. See Encryption Configuration below for details.
     */
    encryptionConfiguration?: pulumi.Input<inputs.networkfirewall.FirewallPolicyEncryptionConfiguration>;
    /**
     * A configuration block describing the rule groups and policy actions to use in the firewall policy. See Firewall Policy below for details.
     */
    firewallPolicy: pulumi.Input<inputs.networkfirewall.FirewallPolicyFirewallPolicy>;
    /**
     * A friendly name of the firewall policy.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Map of resource tags to associate with the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
