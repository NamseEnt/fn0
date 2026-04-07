import * as pulumi from "@pulumi/pulumi";
/**
 * Retrieve the active SES receipt rule set
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const main = aws.ses.getActiveReceiptRuleSet({});
 * ```
 */
export declare function getActiveReceiptRuleSet(args?: GetActiveReceiptRuleSetArgs, opts?: pulumi.InvokeOptions): Promise<GetActiveReceiptRuleSetResult>;
/**
 * A collection of arguments for invoking getActiveReceiptRuleSet.
 */
export interface GetActiveReceiptRuleSetArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
}
/**
 * A collection of values returned by getActiveReceiptRuleSet.
 */
export interface GetActiveReceiptRuleSetResult {
    /**
     * SES receipt rule set ARN.
     */
    readonly arn: string;
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly region: string;
    /**
     * Name of the rule set
     */
    readonly ruleSetName: string;
}
/**
 * Retrieve the active SES receipt rule set
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const main = aws.ses.getActiveReceiptRuleSet({});
 * ```
 */
export declare function getActiveReceiptRuleSetOutput(args?: GetActiveReceiptRuleSetOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetActiveReceiptRuleSetResult>;
/**
 * A collection of arguments for invoking getActiveReceiptRuleSet.
 */
export interface GetActiveReceiptRuleSetOutputArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
