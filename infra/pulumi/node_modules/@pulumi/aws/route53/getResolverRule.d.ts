import * as pulumi from "@pulumi/pulumi";
/**
 * `aws.route53.ResolverRule` provides details about a specific Route53 Resolver rule.
 *
 * ## Example Usage
 *
 * The following example shows how to get a Route53 Resolver rule based on its associated domain name and rule type.
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.route53.getResolverRule({
 *     domainName: "subdomain.example.com",
 *     ruleType: "SYSTEM",
 * });
 * ```
 */
export declare function getResolverRule(args?: GetResolverRuleArgs, opts?: pulumi.InvokeOptions): Promise<GetResolverRuleResult>;
/**
 * A collection of arguments for invoking getResolverRule.
 */
export interface GetResolverRuleArgs {
    /**
     * Domain name the desired resolver rule forwards DNS queries for. Conflicts with `resolverRuleId`.
     */
    domainName?: string;
    /**
     * Friendly name of the desired resolver rule. Conflicts with `resolverRuleId`.
     */
    name?: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
    /**
     * ID of the outbound resolver endpoint of the desired resolver rule. Conflicts with `resolverRuleId`.
     */
    resolverEndpointId?: string;
    /**
     * ID of the desired resolver rule. Conflicts with `domainName`, `name`, `resolverEndpointId` and `ruleType`.
     */
    resolverRuleId?: string;
    /**
     * Rule type of the desired resolver rule. Valid values are `FORWARD`, `SYSTEM` and `RECURSIVE`. Conflicts with `resolverRuleId`.
     */
    ruleType?: string;
    /**
     * Map of tags assigned to the resolver rule.
     */
    tags?: {
        [key: string]: string;
    };
}
/**
 * A collection of values returned by getResolverRule.
 */
export interface GetResolverRuleResult {
    /**
     * ARN (Amazon Resource Name) for the resolver rule.
     */
    readonly arn: string;
    readonly domainName: string;
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly name: string;
    /**
     * When a rule is shared with another AWS account, the account ID of the account that the rule is shared with.
     */
    readonly ownerId: string;
    readonly region: string;
    readonly resolverEndpointId: string;
    readonly resolverRuleId: string;
    readonly ruleType: string;
    /**
     * Whether the rules is shared and, if so, whether the current account is sharing the rule with another account, or another account is sharing the rule with the current account.
     * Values are `NOT_SHARED`, `SHARED_BY_ME` or `SHARED_WITH_ME`
     */
    readonly shareStatus: string;
    /**
     * Map of tags assigned to the resolver rule.
     */
    readonly tags: {
        [key: string]: string;
    };
}
/**
 * `aws.route53.ResolverRule` provides details about a specific Route53 Resolver rule.
 *
 * ## Example Usage
 *
 * The following example shows how to get a Route53 Resolver rule based on its associated domain name and rule type.
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.route53.getResolverRule({
 *     domainName: "subdomain.example.com",
 *     ruleType: "SYSTEM",
 * });
 * ```
 */
export declare function getResolverRuleOutput(args?: GetResolverRuleOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetResolverRuleResult>;
/**
 * A collection of arguments for invoking getResolverRule.
 */
export interface GetResolverRuleOutputArgs {
    /**
     * Domain name the desired resolver rule forwards DNS queries for. Conflicts with `resolverRuleId`.
     */
    domainName?: pulumi.Input<string>;
    /**
     * Friendly name of the desired resolver rule. Conflicts with `resolverRuleId`.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ID of the outbound resolver endpoint of the desired resolver rule. Conflicts with `resolverRuleId`.
     */
    resolverEndpointId?: pulumi.Input<string>;
    /**
     * ID of the desired resolver rule. Conflicts with `domainName`, `name`, `resolverEndpointId` and `ruleType`.
     */
    resolverRuleId?: pulumi.Input<string>;
    /**
     * Rule type of the desired resolver rule. Valid values are `FORWARD`, `SYSTEM` and `RECURSIVE`. Conflicts with `resolverRuleId`.
     */
    ruleType?: pulumi.Input<string>;
    /**
     * Map of tags assigned to the resolver rule.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
