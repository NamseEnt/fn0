import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a resource to manage a GuardDuty filter.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const myFilter = new aws.guardduty.Filter("MyFilter", {
 *     name: "MyFilter",
 *     action: "ARCHIVE",
 *     detectorId: example.id,
 *     rank: 1,
 *     findingCriteria: {
 *         criterions: [
 *             {
 *                 field: "region",
 *                 equals: ["eu-west-1"],
 *             },
 *             {
 *                 field: "service.additionalInfo.threatListName",
 *                 notEquals: [
 *                     "some-threat",
 *                     "another-threat",
 *                 ],
 *             },
 *             {
 *                 field: "updatedAt",
 *                 greaterThan: "2020-01-01T00:00:00Z",
 *                 lessThan: "2020-02-01T00:00:00Z",
 *             },
 *             {
 *                 field: "severity",
 *                 greaterThanOrEqual: "4",
 *             },
 *         ],
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import GuardDuty filters using the detector ID and filter's name separated by a colon. For example:
 *
 * ```sh
 * $ pulumi import aws:guardduty/filter:Filter MyFilter 00b00fd5aecc0ab60a708659477e9617:MyFilter
 * ```
 */
export declare class Filter extends pulumi.CustomResource {
    /**
     * Get an existing Filter resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: FilterState, opts?: pulumi.CustomResourceOptions): Filter;
    /**
     * Returns true if the given object is an instance of Filter.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Filter;
    /**
     * Specifies the action that is to be applied to the findings that match the filter. Can be one of `ARCHIVE` or `NOOP`.
     */
    readonly action: pulumi.Output<string>;
    /**
     * The ARN of the GuardDuty filter.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Description of the filter.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * ID of a GuardDuty detector, attached to your account.
     */
    readonly detectorId: pulumi.Output<string>;
    /**
     * Represents the criteria to be used in the filter for querying findings. Contains one or more `criterion` blocks, documented below.
     */
    readonly findingCriteria: pulumi.Output<outputs.guardduty.FilterFindingCriteria>;
    /**
     * The name of your filter.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Specifies the position of the filter in the list of current filters. Also specifies the order in which this filter is applied to the findings.
     */
    readonly rank: pulumi.Output<number>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The tags that you want to add to the Filter resource. A tag consists of a key and a value. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Create a Filter resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: FilterArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Filter resources.
 */
export interface FilterState {
    /**
     * Specifies the action that is to be applied to the findings that match the filter. Can be one of `ARCHIVE` or `NOOP`.
     */
    action?: pulumi.Input<string>;
    /**
     * The ARN of the GuardDuty filter.
     */
    arn?: pulumi.Input<string>;
    /**
     * Description of the filter.
     */
    description?: pulumi.Input<string>;
    /**
     * ID of a GuardDuty detector, attached to your account.
     */
    detectorId?: pulumi.Input<string>;
    /**
     * Represents the criteria to be used in the filter for querying findings. Contains one or more `criterion` blocks, documented below.
     */
    findingCriteria?: pulumi.Input<inputs.guardduty.FilterFindingCriteria>;
    /**
     * The name of your filter.
     */
    name?: pulumi.Input<string>;
    /**
     * Specifies the position of the filter in the list of current filters. Also specifies the order in which this filter is applied to the findings.
     */
    rank?: pulumi.Input<number>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The tags that you want to add to the Filter resource. A tag consists of a key and a value. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
 * The set of arguments for constructing a Filter resource.
 */
export interface FilterArgs {
    /**
     * Specifies the action that is to be applied to the findings that match the filter. Can be one of `ARCHIVE` or `NOOP`.
     */
    action: pulumi.Input<string>;
    /**
     * Description of the filter.
     */
    description?: pulumi.Input<string>;
    /**
     * ID of a GuardDuty detector, attached to your account.
     */
    detectorId: pulumi.Input<string>;
    /**
     * Represents the criteria to be used in the filter for querying findings. Contains one or more `criterion` blocks, documented below.
     */
    findingCriteria: pulumi.Input<inputs.guardduty.FilterFindingCriteria>;
    /**
     * The name of your filter.
     */
    name?: pulumi.Input<string>;
    /**
     * Specifies the position of the filter in the list of current filters. Also specifies the order in which this filter is applied to the findings.
     */
    rank: pulumi.Input<number>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The tags that you want to add to the Filter resource. A tag consists of a key and a value. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
