import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a CE Cost Category.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = new aws.costexplorer.CostCategory("test", {
 *     name: "NAME",
 *     ruleVersion: "CostCategoryExpression.v1",
 *     rules: [
 *         {
 *             value: "production",
 *             rule: {
 *                 dimension: {
 *                     key: "LINKED_ACCOUNT_NAME",
 *                     values: ["-prod"],
 *                     matchOptions: ["ENDS_WITH"],
 *                 },
 *             },
 *         },
 *         {
 *             value: "staging",
 *             rule: {
 *                 dimension: {
 *                     key: "LINKED_ACCOUNT_NAME",
 *                     values: ["-stg"],
 *                     matchOptions: ["ENDS_WITH"],
 *                 },
 *             },
 *         },
 *         {
 *             value: "testing",
 *             rule: {
 *                 dimension: {
 *                     key: "LINKED_ACCOUNT_NAME",
 *                     values: ["-dev"],
 *                     matchOptions: ["ENDS_WITH"],
 *                 },
 *             },
 *         },
 *     ],
 * });
 * ```
 *
 * ## Import
 *
 * ### Identity Schema
 *
 * #### Required
 *
 * - `arn` (String) Amazon Resource Name (ARN) of the Cost Explorer cost category.
 *
 * Using `pulumi import`, import `aws_ce_cost_category` using the id. For example:
 *
 * console
 *
 * % pulumi import aws_ce_cost_category.example costCategoryARN
 */
export declare class CostCategory extends pulumi.CustomResource {
    /**
     * Get an existing CostCategory resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: CostCategoryState, opts?: pulumi.CustomResourceOptions): CostCategory;
    /**
     * Returns true if the given object is an instance of CostCategory.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is CostCategory;
    /**
     * ARN of the cost category.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Default value for the cost category.
     */
    readonly defaultValue: pulumi.Output<string | undefined>;
    /**
     * Effective end data of your Cost Category.
     */
    readonly effectiveEnd: pulumi.Output<string>;
    /**
     * The Cost Category's effective start date. It can only be a billing start date (first day of the month). If the date isn't provided, it's the first day of the current month. Dates can't be before the previous twelve months, or in the future. For example `2022-11-01T00:00:00Z`.
     */
    readonly effectiveStart: pulumi.Output<string>;
    /**
     * Unique name for the Cost Category.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Rule schema version in this particular Cost Category.
     *
     * The following arguments are optional:
     */
    readonly ruleVersion: pulumi.Output<string>;
    /**
     * Configuration block for the Cost Category rules used to categorize costs. See below.
     */
    readonly rules: pulumi.Output<outputs.costexplorer.CostCategoryRule[]>;
    /**
     * Configuration block for the split charge rules used to allocate your charges between your Cost Category values. See below.
     */
    readonly splitChargeRules: pulumi.Output<outputs.costexplorer.CostCategorySplitChargeRule[] | undefined>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * A map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * Create a CostCategory resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: CostCategoryArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering CostCategory resources.
 */
export interface CostCategoryState {
    /**
     * ARN of the cost category.
     */
    arn?: pulumi.Input<string>;
    /**
     * Default value for the cost category.
     */
    defaultValue?: pulumi.Input<string>;
    /**
     * Effective end data of your Cost Category.
     */
    effectiveEnd?: pulumi.Input<string>;
    /**
     * The Cost Category's effective start date. It can only be a billing start date (first day of the month). If the date isn't provided, it's the first day of the current month. Dates can't be before the previous twelve months, or in the future. For example `2022-11-01T00:00:00Z`.
     */
    effectiveStart?: pulumi.Input<string>;
    /**
     * Unique name for the Cost Category.
     */
    name?: pulumi.Input<string>;
    /**
     * Rule schema version in this particular Cost Category.
     *
     * The following arguments are optional:
     */
    ruleVersion?: pulumi.Input<string>;
    /**
     * Configuration block for the Cost Category rules used to categorize costs. See below.
     */
    rules?: pulumi.Input<pulumi.Input<inputs.costexplorer.CostCategoryRule>[]>;
    /**
     * Configuration block for the split charge rules used to allocate your charges between your Cost Category values. See below.
     */
    splitChargeRules?: pulumi.Input<pulumi.Input<inputs.costexplorer.CostCategorySplitChargeRule>[]>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * A map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
/**
 * The set of arguments for constructing a CostCategory resource.
 */
export interface CostCategoryArgs {
    /**
     * Default value for the cost category.
     */
    defaultValue?: pulumi.Input<string>;
    /**
     * The Cost Category's effective start date. It can only be a billing start date (first day of the month). If the date isn't provided, it's the first day of the current month. Dates can't be before the previous twelve months, or in the future. For example `2022-11-01T00:00:00Z`.
     */
    effectiveStart?: pulumi.Input<string>;
    /**
     * Unique name for the Cost Category.
     */
    name?: pulumi.Input<string>;
    /**
     * Rule schema version in this particular Cost Category.
     *
     * The following arguments are optional:
     */
    ruleVersion: pulumi.Input<string>;
    /**
     * Configuration block for the Cost Category rules used to categorize costs. See below.
     */
    rules: pulumi.Input<pulumi.Input<inputs.costexplorer.CostCategoryRule>[]>;
    /**
     * Configuration block for the split charge rules used to allocate your charges between your Cost Category values. See below.
     */
    splitChargeRules?: pulumi.Input<pulumi.Input<inputs.costexplorer.CostCategorySplitChargeRule>[]>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
