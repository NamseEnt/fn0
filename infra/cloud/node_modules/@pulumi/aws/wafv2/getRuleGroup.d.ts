import * as pulumi from "@pulumi/pulumi";
/**
 * Retrieves the summary of a WAFv2 Rule Group.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.wafv2.getRuleGroup({
 *     name: "some-rule-group",
 *     scope: "REGIONAL",
 * });
 * ```
 */
export declare function getRuleGroup(args: GetRuleGroupArgs, opts?: pulumi.InvokeOptions): Promise<GetRuleGroupResult>;
/**
 * A collection of arguments for invoking getRuleGroup.
 */
export interface GetRuleGroupArgs {
    /**
     * Name of the WAFv2 Rule Group.
     */
    name: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
    /**
     * Specifies whether this is for an AWS CloudFront distribution or for a regional application. Valid values are `CLOUDFRONT` or `REGIONAL`. To work with CloudFront, you must also specify the region `us-east-1` (N. Virginia) on the AWS provider.
     */
    scope: string;
}
/**
 * A collection of values returned by getRuleGroup.
 */
export interface GetRuleGroupResult {
    /**
     * ARN of the entity.
     */
    readonly arn: string;
    /**
     * Description of the rule group that helps with identification.
     */
    readonly description: string;
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly name: string;
    readonly region: string;
    readonly scope: string;
}
/**
 * Retrieves the summary of a WAFv2 Rule Group.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.wafv2.getRuleGroup({
 *     name: "some-rule-group",
 *     scope: "REGIONAL",
 * });
 * ```
 */
export declare function getRuleGroupOutput(args: GetRuleGroupOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetRuleGroupResult>;
/**
 * A collection of arguments for invoking getRuleGroup.
 */
export interface GetRuleGroupOutputArgs {
    /**
     * Name of the WAFv2 Rule Group.
     */
    name: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Specifies whether this is for an AWS CloudFront distribution or for a regional application. Valid values are `CLOUDFRONT` or `REGIONAL`. To work with CloudFront, you must also specify the region `us-east-1` (N. Virginia) on the AWS provider.
     */
    scope: pulumi.Input<string>;
}
