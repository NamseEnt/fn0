import * as pulumi from "@pulumi/pulumi";
import * as outputs from "../types/output";
/**
 * Data source for managing an AWS CloudWatch Contributor Managed Insight Rules.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.cloudwatch.getContributorManagedInsightRules({
 *     resourceArn: "arn:aws:ec2:us-west-2:123456789012:resource-name/resourceid",
 * });
 * ```
 */
export declare function getContributorManagedInsightRules(args: GetContributorManagedInsightRulesArgs, opts?: pulumi.InvokeOptions): Promise<GetContributorManagedInsightRulesResult>;
/**
 * A collection of arguments for invoking getContributorManagedInsightRules.
 */
export interface GetContributorManagedInsightRulesArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
    /**
     * ARN of an Amazon Web Services resource that has managed Contributor Insights rules.
     */
    resourceArn: string;
}
/**
 * A collection of values returned by getContributorManagedInsightRules.
 */
export interface GetContributorManagedInsightRulesResult {
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    /**
     * Managed rules that are available for the specified Amazon Web Services resource. See `managedRules reference` below for details.
     */
    readonly managedRules: outputs.cloudwatch.GetContributorManagedInsightRulesManagedRule[];
    readonly region: string;
    /**
     * If a managed rule is enabled, this is the ARN for the related Amazon Web Services resource.
     */
    readonly resourceArn: string;
}
/**
 * Data source for managing an AWS CloudWatch Contributor Managed Insight Rules.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.cloudwatch.getContributorManagedInsightRules({
 *     resourceArn: "arn:aws:ec2:us-west-2:123456789012:resource-name/resourceid",
 * });
 * ```
 */
export declare function getContributorManagedInsightRulesOutput(args: GetContributorManagedInsightRulesOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetContributorManagedInsightRulesResult>;
/**
 * A collection of arguments for invoking getContributorManagedInsightRules.
 */
export interface GetContributorManagedInsightRulesOutputArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ARN of an Amazon Web Services resource that has managed Contributor Insights rules.
     */
    resourceArn: pulumi.Input<string>;
}
