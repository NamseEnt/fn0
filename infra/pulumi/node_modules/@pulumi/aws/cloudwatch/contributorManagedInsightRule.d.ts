import * as pulumi from "@pulumi/pulumi";
/**
 * Resource for managing an AWS CloudWatch Contributor Managed Insight Rule.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.cloudwatch.ContributorManagedInsightRule("example", {
 *     resourceArn: test.arn,
 *     templateName: "VpcEndpointService-BytesByEndpointId-v1",
 *     ruleState: "DISABLED",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import CloudWatch Contributor Managed Insight Rule using the `resource_arn`. For example:
 *
 * ```sh
 * $ pulumi import aws:cloudwatch/contributorManagedInsightRule:ContributorManagedInsightRule example contributor_managed_insight_rule-id-12345678
 * ```
 */
export declare class ContributorManagedInsightRule extends pulumi.CustomResource {
    /**
     * Get an existing ContributorManagedInsightRule resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ContributorManagedInsightRuleState, opts?: pulumi.CustomResourceOptions): ContributorManagedInsightRule;
    /**
     * Returns true if the given object is an instance of ContributorManagedInsightRule.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ContributorManagedInsightRule;
    /**
     * ARN of the Contributor Managed Insight Rule.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * ARN of an Amazon Web Services resource that has managed Contributor Insights rules.
     */
    readonly resourceArn: pulumi.Output<string>;
    readonly ruleName: pulumi.Output<string>;
    readonly state: pulumi.Output<string>;
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * Template name for the managed Contributor Insights rule, as returned by ListManagedInsightRules.
     *
     * The following arguments are optional:
     */
    readonly templateName: pulumi.Output<string>;
    /**
     * Create a ContributorManagedInsightRule resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ContributorManagedInsightRuleArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ContributorManagedInsightRule resources.
 */
export interface ContributorManagedInsightRuleState {
    /**
     * ARN of the Contributor Managed Insight Rule.
     */
    arn?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ARN of an Amazon Web Services resource that has managed Contributor Insights rules.
     */
    resourceArn?: pulumi.Input<string>;
    ruleName?: pulumi.Input<string>;
    state?: pulumi.Input<string>;
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Template name for the managed Contributor Insights rule, as returned by ListManagedInsightRules.
     *
     * The following arguments are optional:
     */
    templateName?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a ContributorManagedInsightRule resource.
 */
export interface ContributorManagedInsightRuleArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ARN of an Amazon Web Services resource that has managed Contributor Insights rules.
     */
    resourceArn: pulumi.Input<string>;
    state?: pulumi.Input<string>;
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Template name for the managed Contributor Insights rule, as returned by ListManagedInsightRules.
     *
     * The following arguments are optional:
     */
    templateName: pulumi.Input<string>;
}
