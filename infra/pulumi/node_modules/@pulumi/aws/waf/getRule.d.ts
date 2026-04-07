import * as pulumi from "@pulumi/pulumi";
/**
 * `aws.waf.Rule` Retrieves a WAF Rule Resource Id.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.waf.getRule({
 *     name: "tfWAFRule",
 * });
 * ```
 */
export declare function getRule(args: GetRuleArgs, opts?: pulumi.InvokeOptions): Promise<GetRuleResult>;
/**
 * A collection of arguments for invoking getRule.
 */
export interface GetRuleArgs {
    /**
     * Name of the WAF rule.
     */
    name: string;
}
/**
 * A collection of values returned by getRule.
 */
export interface GetRuleResult {
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly name: string;
}
/**
 * `aws.waf.Rule` Retrieves a WAF Rule Resource Id.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.waf.getRule({
 *     name: "tfWAFRule",
 * });
 * ```
 */
export declare function getRuleOutput(args: GetRuleOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetRuleResult>;
/**
 * A collection of arguments for invoking getRule.
 */
export interface GetRuleOutputArgs {
    /**
     * Name of the WAF rule.
     */
    name: pulumi.Input<string>;
}
