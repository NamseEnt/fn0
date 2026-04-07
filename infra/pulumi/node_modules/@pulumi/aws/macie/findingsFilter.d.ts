import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a resource to manage an [Amazon Macie Findings Filter](https://docs.aws.amazon.com/macie/latest/APIReference/findingsfilters-id.html).
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.macie2.Account("example", {});
 * const test = new aws.macie.FindingsFilter("test", {
 *     name: "NAME OF THE FINDINGS FILTER",
 *     description: "DESCRIPTION",
 *     position: 1,
 *     action: "ARCHIVE",
 *     findingCriteria: {
 *         criterions: [{
 *             field: "region",
 *             eqs: [current.region],
 *         }],
 *     },
 * }, {
 *     dependsOn: [testAwsMacie2Account],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_macie2_findings_filter` using the id. For example:
 *
 * ```sh
 * $ pulumi import aws:macie/findingsFilter:FindingsFilter example abcd1
 * ```
 */
export declare class FindingsFilter extends pulumi.CustomResource {
    /**
     * Get an existing FindingsFilter resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: FindingsFilterState, opts?: pulumi.CustomResourceOptions): FindingsFilter;
    /**
     * Returns true if the given object is an instance of FindingsFilter.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is FindingsFilter;
    /**
     * The action to perform on findings that meet the filter criteria (`findingCriteria`). Valid values are: `ARCHIVE`, suppress (automatically archive) the findings; and, `NOOP`, don't perform any action on the findings.
     */
    readonly action: pulumi.Output<string>;
    /**
     * The Amazon Resource Name (ARN) of the Findings Filter.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * A custom description of the filter. The description can contain as many as 512 characters.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * The criteria to use to filter findings.
     */
    readonly findingCriteria: pulumi.Output<outputs.macie.FindingsFilterFindingCriteria>;
    /**
     * A custom name for the filter. The name must contain at least 3 characters and can contain as many as 64 characters. If omitted, the provider will assign a random, unique name. Conflicts with `namePrefix`.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Creates a unique name beginning with the specified prefix. Conflicts with `name`.
     */
    readonly namePrefix: pulumi.Output<string>;
    /**
     * The position of the filter in the list of saved filters on the Amazon Macie console. This value also determines the order in which the filter is applied to findings, relative to other filters that are also applied to the findings.
     */
    readonly position: pulumi.Output<number>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Create a FindingsFilter resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: FindingsFilterArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering FindingsFilter resources.
 */
export interface FindingsFilterState {
    /**
     * The action to perform on findings that meet the filter criteria (`findingCriteria`). Valid values are: `ARCHIVE`, suppress (automatically archive) the findings; and, `NOOP`, don't perform any action on the findings.
     */
    action?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) of the Findings Filter.
     */
    arn?: pulumi.Input<string>;
    /**
     * A custom description of the filter. The description can contain as many as 512 characters.
     */
    description?: pulumi.Input<string>;
    /**
     * The criteria to use to filter findings.
     */
    findingCriteria?: pulumi.Input<inputs.macie.FindingsFilterFindingCriteria>;
    /**
     * A custom name for the filter. The name must contain at least 3 characters and can contain as many as 64 characters. If omitted, the provider will assign a random, unique name. Conflicts with `namePrefix`.
     */
    name?: pulumi.Input<string>;
    /**
     * Creates a unique name beginning with the specified prefix. Conflicts with `name`.
     */
    namePrefix?: pulumi.Input<string>;
    /**
     * The position of the filter in the list of saved filters on the Amazon Macie console. This value also determines the order in which the filter is applied to findings, relative to other filters that are also applied to the findings.
     */
    position?: pulumi.Input<number>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
 * The set of arguments for constructing a FindingsFilter resource.
 */
export interface FindingsFilterArgs {
    /**
     * The action to perform on findings that meet the filter criteria (`findingCriteria`). Valid values are: `ARCHIVE`, suppress (automatically archive) the findings; and, `NOOP`, don't perform any action on the findings.
     */
    action: pulumi.Input<string>;
    /**
     * A custom description of the filter. The description can contain as many as 512 characters.
     */
    description?: pulumi.Input<string>;
    /**
     * The criteria to use to filter findings.
     */
    findingCriteria: pulumi.Input<inputs.macie.FindingsFilterFindingCriteria>;
    /**
     * A custom name for the filter. The name must contain at least 3 characters and can contain as many as 64 characters. If omitted, the provider will assign a random, unique name. Conflicts with `namePrefix`.
     */
    name?: pulumi.Input<string>;
    /**
     * Creates a unique name beginning with the specified prefix. Conflicts with `name`.
     */
    namePrefix?: pulumi.Input<string>;
    /**
     * The position of the filter in the list of saved filters on the Amazon Macie console. This value also determines the order in which the filter is applied to findings, relative to other filters that are also applied to the findings.
     */
    position?: pulumi.Input<number>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
