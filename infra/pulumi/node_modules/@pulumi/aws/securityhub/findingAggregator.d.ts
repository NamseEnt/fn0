import * as pulumi from "@pulumi/pulumi";
/**
 * Manages a Security Hub finding aggregator. Security Hub needs to be enabled in a region in order for the aggregator to pull through findings.
 *
 * ## Example Usage
 *
 * ### All Regions Usage
 *
 * The following example will enable the aggregator for every region.
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.securityhub.Account("example", {});
 * const exampleFindingAggregator = new aws.securityhub.FindingAggregator("example", {linkingMode: "ALL_REGIONS"}, {
 *     dependsOn: [example],
 * });
 * ```
 *
 * ### All Regions Except Specified Regions Usage
 *
 * The following example will enable the aggregator for every region except those specified in `specifiedRegions`.
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.securityhub.Account("example", {});
 * const exampleFindingAggregator = new aws.securityhub.FindingAggregator("example", {
 *     linkingMode: "ALL_REGIONS_EXCEPT_SPECIFIED",
 *     specifiedRegions: [
 *         "eu-west-1",
 *         "eu-west-2",
 *     ],
 * }, {
 *     dependsOn: [example],
 * });
 * ```
 *
 * ### Specified Regions Usage
 *
 * The following example will enable the aggregator for every region specified in `specifiedRegions`.
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.securityhub.Account("example", {});
 * const exampleFindingAggregator = new aws.securityhub.FindingAggregator("example", {
 *     linkingMode: "SPECIFIED_REGIONS",
 *     specifiedRegions: [
 *         "eu-west-1",
 *         "eu-west-2",
 *     ],
 * }, {
 *     dependsOn: [example],
 * });
 * ```
 *
 * ### No Regions Usage
 *
 * The following example will enable the aggregator but not link any AWS Regions to the home Region.
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.securityhub.Account("example", {});
 * const exampleFindingAggregator = new aws.securityhub.FindingAggregator("example", {linkingMode: "NO_REGIONS"}, {
 *     dependsOn: [example],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import an existing Security Hub finding aggregator using the `arn`. For example:
 *
 * ```sh
 * $ pulumi import aws:securityhub/findingAggregator:FindingAggregator example arn:aws:securityhub:eu-west-1:123456789098:finding-aggregator/abcd1234-abcd-1234-1234-abcdef123456
 * ```
 */
export declare class FindingAggregator extends pulumi.CustomResource {
    /**
     * Get an existing FindingAggregator resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: FindingAggregatorState, opts?: pulumi.CustomResourceOptions): FindingAggregator;
    /**
     * Returns true if the given object is an instance of FindingAggregator.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is FindingAggregator;
    /**
     * Indicates whether to aggregate findings from all of the available Regions or from a specified list. The options are `ALL_REGIONS`, `ALL_REGIONS_EXCEPT_SPECIFIED`, `SPECIFIED_REGIONS` or `NO_REGIONS`. When `ALL_REGIONS` or `ALL_REGIONS_EXCEPT_SPECIFIED` are used, Security Hub will automatically aggregate findings from new Regions as Security Hub supports them and you opt into them.
     */
    readonly linkingMode: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * List of regions to include or exclude (required if `linkingMode` is set to `ALL_REGIONS_EXCEPT_SPECIFIED` or `SPECIFIED_REGIONS`)
     */
    readonly specifiedRegions: pulumi.Output<string[] | undefined>;
    /**
     * Create a FindingAggregator resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: FindingAggregatorArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering FindingAggregator resources.
 */
export interface FindingAggregatorState {
    /**
     * Indicates whether to aggregate findings from all of the available Regions or from a specified list. The options are `ALL_REGIONS`, `ALL_REGIONS_EXCEPT_SPECIFIED`, `SPECIFIED_REGIONS` or `NO_REGIONS`. When `ALL_REGIONS` or `ALL_REGIONS_EXCEPT_SPECIFIED` are used, Security Hub will automatically aggregate findings from new Regions as Security Hub supports them and you opt into them.
     */
    linkingMode?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * List of regions to include or exclude (required if `linkingMode` is set to `ALL_REGIONS_EXCEPT_SPECIFIED` or `SPECIFIED_REGIONS`)
     */
    specifiedRegions?: pulumi.Input<pulumi.Input<string>[]>;
}
/**
 * The set of arguments for constructing a FindingAggregator resource.
 */
export interface FindingAggregatorArgs {
    /**
     * Indicates whether to aggregate findings from all of the available Regions or from a specified list. The options are `ALL_REGIONS`, `ALL_REGIONS_EXCEPT_SPECIFIED`, `SPECIFIED_REGIONS` or `NO_REGIONS`. When `ALL_REGIONS` or `ALL_REGIONS_EXCEPT_SPECIFIED` are used, Security Hub will automatically aggregate findings from new Regions as Security Hub supports them and you opt into them.
     */
    linkingMode: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * List of regions to include or exclude (required if `linkingMode` is set to `ALL_REGIONS_EXCEPT_SPECIFIED` or `SPECIFIED_REGIONS`)
     */
    specifiedRegions?: pulumi.Input<pulumi.Input<string>[]>;
}
