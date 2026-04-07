import * as pulumi from "@pulumi/pulumi";
/**
 * `aws.wafregional.getSubscribedRuleGroup` retrieves information about a Managed WAF Rule Group from AWS Marketplace for use in WAF Regional (needs to be subscribed to first).
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const byName = aws.wafregional.getSubscribedRuleGroup({
 *     name: "F5 Bot Detection Signatures For AWS WAF",
 * });
 * const byMetricName = aws.wafregional.getSubscribedRuleGroup({
 *     metricName: "F5BotDetectionSignatures",
 * });
 * const acl = new aws.wafregional.WebAcl("acl", {rules: [
 *     {
 *         priority: 1,
 *         ruleId: byName.then(byName => byName.id),
 *         type: "GROUP",
 *     },
 *     {
 *         priority: 2,
 *         ruleId: byMetricName.then(byMetricName => byMetricName.id),
 *         type: "GROUP",
 *     },
 * ]});
 * ```
 */
export declare function getSubscribedRuleGroup(args?: GetSubscribedRuleGroupArgs, opts?: pulumi.InvokeOptions): Promise<GetSubscribedRuleGroupResult>;
/**
 * A collection of arguments for invoking getSubscribedRuleGroup.
 */
export interface GetSubscribedRuleGroupArgs {
    /**
     * Name of the WAF rule group.
     *
     * At least one of `name` or `metricName` must be configured.
     */
    metricName?: string;
    /**
     * Name of the WAF rule group.
     */
    name?: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
}
/**
 * A collection of values returned by getSubscribedRuleGroup.
 */
export interface GetSubscribedRuleGroupResult {
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly metricName?: string;
    readonly name?: string;
    readonly region: string;
}
/**
 * `aws.wafregional.getSubscribedRuleGroup` retrieves information about a Managed WAF Rule Group from AWS Marketplace for use in WAF Regional (needs to be subscribed to first).
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const byName = aws.wafregional.getSubscribedRuleGroup({
 *     name: "F5 Bot Detection Signatures For AWS WAF",
 * });
 * const byMetricName = aws.wafregional.getSubscribedRuleGroup({
 *     metricName: "F5BotDetectionSignatures",
 * });
 * const acl = new aws.wafregional.WebAcl("acl", {rules: [
 *     {
 *         priority: 1,
 *         ruleId: byName.then(byName => byName.id),
 *         type: "GROUP",
 *     },
 *     {
 *         priority: 2,
 *         ruleId: byMetricName.then(byMetricName => byMetricName.id),
 *         type: "GROUP",
 *     },
 * ]});
 * ```
 */
export declare function getSubscribedRuleGroupOutput(args?: GetSubscribedRuleGroupOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetSubscribedRuleGroupResult>;
/**
 * A collection of arguments for invoking getSubscribedRuleGroup.
 */
export interface GetSubscribedRuleGroupOutputArgs {
    /**
     * Name of the WAF rule group.
     *
     * At least one of `name` or `metricName` must be configured.
     */
    metricName?: pulumi.Input<string>;
    /**
     * Name of the WAF rule group.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
