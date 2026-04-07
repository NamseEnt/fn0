import * as pulumi from "@pulumi/pulumi";
import * as outputs from "../types/output";
/**
 * `aws.route53.getResolverFirewallRules` Provides details about rules in a specific Route53 Resolver Firewall rule group.
 *
 * ## Example Usage
 *
 * The following example shows how to get Route53 Resolver Firewall rules based on its associated firewall group id.
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.route53.getResolverFirewallRules({
 *     firewallRuleGroupId: exampleAwsRoute53ResolverFirewallRuleGroup.id,
 * });
 * ```
 */
export declare function getResolverFirewallRules(args: GetResolverFirewallRulesArgs, opts?: pulumi.InvokeOptions): Promise<GetResolverFirewallRulesResult>;
/**
 * A collection of arguments for invoking getResolverFirewallRules.
 */
export interface GetResolverFirewallRulesArgs {
    /**
     * The action that DNS Firewall should take on a DNS query when it matches one of the domains in the rule's domain list.
     */
    action?: string;
    /**
     * The unique identifier of the firewall rule group that you want to retrieve the rules for.
     */
    firewallRuleGroupId: string;
    /**
     * The setting that determines the processing order of the rules in a rule group.
     */
    priority?: number;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
}
/**
 * A collection of values returned by getResolverFirewallRules.
 */
export interface GetResolverFirewallRulesResult {
    readonly action?: string;
    readonly firewallRuleGroupId: string;
    /**
     * List with information about the firewall rules. See details below.
     */
    readonly firewallRules: outputs.route53.GetResolverFirewallRulesFirewallRule[];
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly priority?: number;
    readonly region: string;
}
/**
 * `aws.route53.getResolverFirewallRules` Provides details about rules in a specific Route53 Resolver Firewall rule group.
 *
 * ## Example Usage
 *
 * The following example shows how to get Route53 Resolver Firewall rules based on its associated firewall group id.
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.route53.getResolverFirewallRules({
 *     firewallRuleGroupId: exampleAwsRoute53ResolverFirewallRuleGroup.id,
 * });
 * ```
 */
export declare function getResolverFirewallRulesOutput(args: GetResolverFirewallRulesOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetResolverFirewallRulesResult>;
/**
 * A collection of arguments for invoking getResolverFirewallRules.
 */
export interface GetResolverFirewallRulesOutputArgs {
    /**
     * The action that DNS Firewall should take on a DNS query when it matches one of the domains in the rule's domain list.
     */
    action?: pulumi.Input<string>;
    /**
     * The unique identifier of the firewall rule group that you want to retrieve the rules for.
     */
    firewallRuleGroupId: pulumi.Input<string>;
    /**
     * The setting that determines the processing order of the rules in a rule group.
     */
    priority?: pulumi.Input<number>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
