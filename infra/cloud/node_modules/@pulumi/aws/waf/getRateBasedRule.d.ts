import * as pulumi from "@pulumi/pulumi";
/**
 * `aws.waf.RateBasedRule` Retrieves a WAF Rate Based Rule Resource Id.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.waf.getRateBasedRule({
 *     name: "tfWAFRateBasedRule",
 * });
 * ```
 */
export declare function getRateBasedRule(args: GetRateBasedRuleArgs, opts?: pulumi.InvokeOptions): Promise<GetRateBasedRuleResult>;
/**
 * A collection of arguments for invoking getRateBasedRule.
 */
export interface GetRateBasedRuleArgs {
    /**
     * Name of the WAF rate based rule.
     */
    name: string;
}
/**
 * A collection of values returned by getRateBasedRule.
 */
export interface GetRateBasedRuleResult {
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly name: string;
}
/**
 * `aws.waf.RateBasedRule` Retrieves a WAF Rate Based Rule Resource Id.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.waf.getRateBasedRule({
 *     name: "tfWAFRateBasedRule",
 * });
 * ```
 */
export declare function getRateBasedRuleOutput(args: GetRateBasedRuleOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetRateBasedRuleResult>;
/**
 * A collection of arguments for invoking getRateBasedRule.
 */
export interface GetRateBasedRuleOutputArgs {
    /**
     * Name of the WAF rate based rule.
     */
    name: pulumi.Input<string>;
}
