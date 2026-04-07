import * as pulumi from "@pulumi/pulumi";
/**
 * Resource for managing an AWS CloudWatch Contributor Insight Rule.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = new aws.cloudwatch.ContributorInsightRule("test", {
 *     ruleName: "testing",
 *     ruleState: "ENABLED",
 *     ruleDefinition: "{\"Schema\":{\"Name\":\"CloudWatchLogRule\",\"Version\":1},\"AggregateOn\":\"Count\",\"Contribution\":{\"Filters\":[{\"In\":[\"some-keyword\"],\"Match\":\"$.message\"}],\"Keys\":[\"$.country\"]},\"LogFormat\":\"JSON\",\"LogGroupNames\":[\"/aws/lambda/api-prod\"]}",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import CloudWatch Contributor Insight Rule using the `rule_name`. For example:
 *
 * ```sh
 * $ pulumi import aws:cloudwatch/contributorInsightRule:ContributorInsightRule example contributor_insight_rule-name
 * ```
 */
export declare class ContributorInsightRule extends pulumi.CustomResource {
    /**
     * Get an existing ContributorInsightRule resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ContributorInsightRuleState, opts?: pulumi.CustomResourceOptions): ContributorInsightRule;
    /**
     * Returns true if the given object is an instance of ContributorInsightRule.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ContributorInsightRule;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * ARN of the Contributor Insight Rule.
     */
    readonly resourceArn: pulumi.Output<string>;
    /**
     * Definition of the rule, as a JSON object. For details on the valid syntax, see [Contributor Insights Rule Syntax](https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/ContributorInsights-RuleSyntax.html).
     */
    readonly ruleDefinition: pulumi.Output<string>;
    /**
     * Unique name of the rule.
     *
     * The following arguments are optional:
     */
    readonly ruleName: pulumi.Output<string>;
    /**
     * State of the rule. Valid values are `ENABLED` and `DISABLED`.
     */
    readonly ruleState: pulumi.Output<string | undefined>;
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * Create a ContributorInsightRule resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ContributorInsightRuleArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ContributorInsightRule resources.
 */
export interface ContributorInsightRuleState {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ARN of the Contributor Insight Rule.
     */
    resourceArn?: pulumi.Input<string>;
    /**
     * Definition of the rule, as a JSON object. For details on the valid syntax, see [Contributor Insights Rule Syntax](https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/ContributorInsights-RuleSyntax.html).
     */
    ruleDefinition?: pulumi.Input<string>;
    /**
     * Unique name of the rule.
     *
     * The following arguments are optional:
     */
    ruleName?: pulumi.Input<string>;
    /**
     * State of the rule. Valid values are `ENABLED` and `DISABLED`.
     */
    ruleState?: pulumi.Input<string>;
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
/**
 * The set of arguments for constructing a ContributorInsightRule resource.
 */
export interface ContributorInsightRuleArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Definition of the rule, as a JSON object. For details on the valid syntax, see [Contributor Insights Rule Syntax](https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/ContributorInsights-RuleSyntax.html).
     */
    ruleDefinition: pulumi.Input<string>;
    /**
     * Unique name of the rule.
     *
     * The following arguments are optional:
     */
    ruleName: pulumi.Input<string>;
    /**
     * State of the rule. Valid values are `ENABLED` and `DISABLED`.
     */
    ruleState?: pulumi.Input<string>;
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
