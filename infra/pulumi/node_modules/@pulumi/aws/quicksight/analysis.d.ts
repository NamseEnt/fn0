import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing a QuickSight Analysis.
 *
 * ## Example Usage
 *
 * ### From Source Template
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.quicksight.Analysis("example", {
 *     analysisId: "example-id",
 *     name: "example-name",
 *     sourceEntity: {
 *         sourceTemplate: {
 *             arn: source.arn,
 *             dataSetReferences: [{
 *                 dataSetArn: dataset.arn,
 *                 dataSetPlaceholder: "1",
 *             }],
 *         },
 *     },
 * });
 * ```
 *
 * ### With Definition
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.quicksight.Analysis("example", {
 *     analysisId: "example-id",
 *     name: "example-name",
 *     definition: {
 *         dataSetIdentifiersDeclarations: [{
 *             dataSetArn: dataset.arn,
 *             identifier: "1",
 *         }],
 *         sheets: [{
 *             title: "Example",
 *             sheetId: "Example1",
 *             visuals: [{
 *                 lineChartVisual: {
 *                     visualId: "LineChart",
 *                     title: {
 *                         formatText: {
 *                             plainText: "Line Chart Example",
 *                         },
 *                     },
 *                     chartConfiguration: {
 *                         fieldWells: {
 *                             lineChartAggregatedFieldWells: {
 *                                 categories: [{
 *                                     categoricalDimensionField: {
 *                                         fieldId: "1",
 *                                         column: {
 *                                             dataSetIdentifier: "1",
 *                                             columnName: "Column1",
 *                                         },
 *                                     },
 *                                 }],
 *                                 values: [{
 *                                     categoricalMeasureField: {
 *                                         fieldId: "2",
 *                                         column: {
 *                                             dataSetIdentifier: "1",
 *                                             columnName: "Column1",
 *                                         },
 *                                         aggregationFunction: "COUNT",
 *                                     },
 *                                 }],
 *                             },
 *                         },
 *                     },
 *                 },
 *             }],
 *         }],
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import a QuickSight Analysis using the AWS account ID and analysis ID separated by a comma (`,`). For example:
 *
 * ```sh
 * $ pulumi import aws:quicksight/analysis:Analysis example 123456789012,example-id
 * ```
 */
export declare class Analysis extends pulumi.CustomResource {
    /**
     * Get an existing Analysis resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: AnalysisState, opts?: pulumi.CustomResourceOptions): Analysis;
    /**
     * Returns true if the given object is an instance of Analysis.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Analysis;
    /**
     * Identifier for the analysis.
     */
    readonly analysisId: pulumi.Output<string>;
    /**
     * ARN of the analysis.
     */
    readonly arn: pulumi.Output<string>;
    readonly awsAccountId: pulumi.Output<string>;
    /**
     * The time that the analysis was created.
     */
    readonly createdTime: pulumi.Output<string>;
    readonly lastPublishedTime: pulumi.Output<string>;
    /**
     * The time that the analysis was last updated.
     */
    readonly lastUpdatedTime: pulumi.Output<string>;
    /**
     * Display name for the analysis.
     *
     * The following arguments are optional:
     */
    readonly name: pulumi.Output<string>;
    /**
     * The parameters for the creation of the analysis, which you want to use to override the default settings. An analysis can have any type of parameters, and some parameters might accept multiple values. See parameters.
     */
    readonly parameters: pulumi.Output<outputs.quicksight.AnalysisParameters>;
    /**
     * A set of resource permissions on the analysis. Maximum of 64 items. See permissions.
     */
    readonly permissions: pulumi.Output<outputs.quicksight.AnalysisPermission[] | undefined>;
    /**
     * A value that specifies the number of days that Amazon QuickSight waits before it deletes the analysis. Use `0` to force deletion without recovery. Minimum value of `7`. Maximum value of `30`. Default to `30`.
     */
    readonly recoveryWindowInDays: pulumi.Output<number | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The entity that you are using as a source when you create the analysis (template). Only one of `definition` or `sourceEntity` should be configured. See source_entity.
     */
    readonly sourceEntity: pulumi.Output<outputs.quicksight.AnalysisSourceEntity | undefined>;
    /**
     * The analysis creation status.
     */
    readonly status: pulumi.Output<string>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * The Amazon Resource Name (ARN) of the theme that is being used for this analysis. The theme ARN must exist in the same AWS account where you create the analysis.
     */
    readonly themeArn: pulumi.Output<string | undefined>;
    /**
     * Create a Analysis resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: AnalysisArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Analysis resources.
 */
export interface AnalysisState {
    /**
     * Identifier for the analysis.
     */
    analysisId?: pulumi.Input<string>;
    /**
     * ARN of the analysis.
     */
    arn?: pulumi.Input<string>;
    awsAccountId?: pulumi.Input<string>;
    /**
     * The time that the analysis was created.
     */
    createdTime?: pulumi.Input<string>;
    lastPublishedTime?: pulumi.Input<string>;
    /**
     * The time that the analysis was last updated.
     */
    lastUpdatedTime?: pulumi.Input<string>;
    /**
     * Display name for the analysis.
     *
     * The following arguments are optional:
     */
    name?: pulumi.Input<string>;
    /**
     * The parameters for the creation of the analysis, which you want to use to override the default settings. An analysis can have any type of parameters, and some parameters might accept multiple values. See parameters.
     */
    parameters?: pulumi.Input<inputs.quicksight.AnalysisParameters>;
    /**
     * A set of resource permissions on the analysis. Maximum of 64 items. See permissions.
     */
    permissions?: pulumi.Input<pulumi.Input<inputs.quicksight.AnalysisPermission>[]>;
    /**
     * A value that specifies the number of days that Amazon QuickSight waits before it deletes the analysis. Use `0` to force deletion without recovery. Minimum value of `7`. Maximum value of `30`. Default to `30`.
     */
    recoveryWindowInDays?: pulumi.Input<number>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The entity that you are using as a source when you create the analysis (template). Only one of `definition` or `sourceEntity` should be configured. See source_entity.
     */
    sourceEntity?: pulumi.Input<inputs.quicksight.AnalysisSourceEntity>;
    /**
     * The analysis creation status.
     */
    status?: pulumi.Input<string>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
    /**
     * The Amazon Resource Name (ARN) of the theme that is being used for this analysis. The theme ARN must exist in the same AWS account where you create the analysis.
     */
    themeArn?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a Analysis resource.
 */
export interface AnalysisArgs {
    /**
     * Identifier for the analysis.
     */
    analysisId: pulumi.Input<string>;
    awsAccountId?: pulumi.Input<string>;
    /**
     * Display name for the analysis.
     *
     * The following arguments are optional:
     */
    name?: pulumi.Input<string>;
    /**
     * The parameters for the creation of the analysis, which you want to use to override the default settings. An analysis can have any type of parameters, and some parameters might accept multiple values. See parameters.
     */
    parameters?: pulumi.Input<inputs.quicksight.AnalysisParameters>;
    /**
     * A set of resource permissions on the analysis. Maximum of 64 items. See permissions.
     */
    permissions?: pulumi.Input<pulumi.Input<inputs.quicksight.AnalysisPermission>[]>;
    /**
     * A value that specifies the number of days that Amazon QuickSight waits before it deletes the analysis. Use `0` to force deletion without recovery. Minimum value of `7`. Maximum value of `30`. Default to `30`.
     */
    recoveryWindowInDays?: pulumi.Input<number>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The entity that you are using as a source when you create the analysis (template). Only one of `definition` or `sourceEntity` should be configured. See source_entity.
     */
    sourceEntity?: pulumi.Input<inputs.quicksight.AnalysisSourceEntity>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The Amazon Resource Name (ARN) of the theme that is being used for this analysis. The theme ARN must exist in the same AWS account where you create the analysis.
     */
    themeArn?: pulumi.Input<string>;
}
