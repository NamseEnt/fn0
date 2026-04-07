import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS AccessAnalyzer Archive Rule.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.accessanalyzer.ArchiveRule("example", {
 *     analyzerName: "example-analyzer",
 *     ruleName: "example-rule",
 *     filters: [
 *         {
 *             criteria: "condition.aws:UserId",
 *             eqs: ["userid"],
 *         },
 *         {
 *             criteria: "error",
 *             exists: "true",
 *         },
 *         {
 *             criteria: "isPublic",
 *             eqs: ["false"],
 *         },
 *     ],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import AccessAnalyzer ArchiveRule using the `analyzer_name/rule_name`. For example:
 *
 * ```sh
 * $ pulumi import aws:accessanalyzer/archiveRule:ArchiveRule example example-analyzer/example-rule
 * ```
 */
export declare class ArchiveRule extends pulumi.CustomResource {
    /**
     * Get an existing ArchiveRule resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ArchiveRuleState, opts?: pulumi.CustomResourceOptions): ArchiveRule;
    /**
     * Returns true if the given object is an instance of ArchiveRule.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ArchiveRule;
    /**
     * Analyzer name.
     */
    readonly analyzerName: pulumi.Output<string>;
    /**
     * Filter criteria for the archive rule. See Filter for more details.
     */
    readonly filters: pulumi.Output<outputs.accessanalyzer.ArchiveRuleFilter[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Rule name.
     */
    readonly ruleName: pulumi.Output<string>;
    /**
     * Create a ArchiveRule resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ArchiveRuleArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ArchiveRule resources.
 */
export interface ArchiveRuleState {
    /**
     * Analyzer name.
     */
    analyzerName?: pulumi.Input<string>;
    /**
     * Filter criteria for the archive rule. See Filter for more details.
     */
    filters?: pulumi.Input<pulumi.Input<inputs.accessanalyzer.ArchiveRuleFilter>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Rule name.
     */
    ruleName?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a ArchiveRule resource.
 */
export interface ArchiveRuleArgs {
    /**
     * Analyzer name.
     */
    analyzerName: pulumi.Input<string>;
    /**
     * Filter criteria for the archive rule. See Filter for more details.
     */
    filters: pulumi.Input<pulumi.Input<inputs.accessanalyzer.ArchiveRuleFilter>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Rule name.
     */
    ruleName: pulumi.Input<string>;
}
