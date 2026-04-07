import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing a QuickSight Dashboard.
 *
 * ## Example Usage
 *
 * ### From Source Template
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.quicksight.Dashboard("example", {
 *     dashboardId: "example-id",
 *     name: "example-name",
 *     versionDescription: "version",
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
 * const example = new aws.quicksight.Dashboard("example", {
 *     dashboardId: "example-id",
 *     name: "example-name",
 *     versionDescription: "version",
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
 * Using `pulumi import`, import a QuickSight Dashboard using the AWS account ID and dashboard ID separated by a comma (`,`). For example:
 *
 * ```sh
 * $ pulumi import aws:quicksight/dashboard:Dashboard example 123456789012,example-id
 * ```
 */
export declare class Dashboard extends pulumi.CustomResource {
    /**
     * Get an existing Dashboard resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: DashboardState, opts?: pulumi.CustomResourceOptions): Dashboard;
    /**
     * Returns true if the given object is an instance of Dashboard.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Dashboard;
    /**
     * ARN of the dashboard.
     */
    readonly arn: pulumi.Output<string>;
    readonly awsAccountId: pulumi.Output<string>;
    /**
     * The time that the dashboard was created.
     */
    readonly createdTime: pulumi.Output<string>;
    /**
     * Identifier for the dashboard.
     */
    readonly dashboardId: pulumi.Output<string>;
    /**
     * Options for publishing the dashboard. See dashboard_publish_options.
     */
    readonly dashboardPublishOptions: pulumi.Output<outputs.quicksight.DashboardDashboardPublishOptions>;
    readonly lastPublishedTime: pulumi.Output<string>;
    /**
     * The time that the dashboard was last updated.
     */
    readonly lastUpdatedTime: pulumi.Output<string>;
    /**
     * Display name for the dashboard.
     */
    readonly name: pulumi.Output<string>;
    /**
     * The parameters for the creation of the dashboard, which you want to use to override the default settings. A dashboard can have any type of parameters, and some parameters might accept multiple values. See parameters.
     */
    readonly parameters: pulumi.Output<outputs.quicksight.DashboardParameters>;
    /**
     * A set of resource permissions on the dashboard. Maximum of 64 items. See permissions.
     */
    readonly permissions: pulumi.Output<outputs.quicksight.DashboardPermission[] | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The entity that you are using as a source when you create the dashboard (template). Only one of `definition` or `sourceEntity` should be configured. See source_entity.
     */
    readonly sourceEntity: pulumi.Output<outputs.quicksight.DashboardSourceEntity | undefined>;
    /**
     * Amazon Resource Name (ARN) of a template that was used to create this dashboard.
     */
    readonly sourceEntityArn: pulumi.Output<string>;
    /**
     * The dashboard creation status.
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
     * The Amazon Resource Name (ARN) of the theme that is being used for this dashboard. The theme ARN must exist in the same AWS account where you create the dashboard.
     */
    readonly themeArn: pulumi.Output<string | undefined>;
    /**
     * A description of the current dashboard version being created/updated.
     *
     * The following arguments are optional:
     */
    readonly versionDescription: pulumi.Output<string>;
    /**
     * The version number of the dashboard version.
     */
    readonly versionNumber: pulumi.Output<number>;
    /**
     * Create a Dashboard resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: DashboardArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Dashboard resources.
 */
export interface DashboardState {
    /**
     * ARN of the dashboard.
     */
    arn?: pulumi.Input<string>;
    awsAccountId?: pulumi.Input<string>;
    /**
     * The time that the dashboard was created.
     */
    createdTime?: pulumi.Input<string>;
    /**
     * Identifier for the dashboard.
     */
    dashboardId?: pulumi.Input<string>;
    /**
     * Options for publishing the dashboard. See dashboard_publish_options.
     */
    dashboardPublishOptions?: pulumi.Input<inputs.quicksight.DashboardDashboardPublishOptions>;
    lastPublishedTime?: pulumi.Input<string>;
    /**
     * The time that the dashboard was last updated.
     */
    lastUpdatedTime?: pulumi.Input<string>;
    /**
     * Display name for the dashboard.
     */
    name?: pulumi.Input<string>;
    /**
     * The parameters for the creation of the dashboard, which you want to use to override the default settings. A dashboard can have any type of parameters, and some parameters might accept multiple values. See parameters.
     */
    parameters?: pulumi.Input<inputs.quicksight.DashboardParameters>;
    /**
     * A set of resource permissions on the dashboard. Maximum of 64 items. See permissions.
     */
    permissions?: pulumi.Input<pulumi.Input<inputs.quicksight.DashboardPermission>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The entity that you are using as a source when you create the dashboard (template). Only one of `definition` or `sourceEntity` should be configured. See source_entity.
     */
    sourceEntity?: pulumi.Input<inputs.quicksight.DashboardSourceEntity>;
    /**
     * Amazon Resource Name (ARN) of a template that was used to create this dashboard.
     */
    sourceEntityArn?: pulumi.Input<string>;
    /**
     * The dashboard creation status.
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
     * The Amazon Resource Name (ARN) of the theme that is being used for this dashboard. The theme ARN must exist in the same AWS account where you create the dashboard.
     */
    themeArn?: pulumi.Input<string>;
    /**
     * A description of the current dashboard version being created/updated.
     *
     * The following arguments are optional:
     */
    versionDescription?: pulumi.Input<string>;
    /**
     * The version number of the dashboard version.
     */
    versionNumber?: pulumi.Input<number>;
}
/**
 * The set of arguments for constructing a Dashboard resource.
 */
export interface DashboardArgs {
    awsAccountId?: pulumi.Input<string>;
    /**
     * Identifier for the dashboard.
     */
    dashboardId: pulumi.Input<string>;
    /**
     * Options for publishing the dashboard. See dashboard_publish_options.
     */
    dashboardPublishOptions?: pulumi.Input<inputs.quicksight.DashboardDashboardPublishOptions>;
    /**
     * Display name for the dashboard.
     */
    name?: pulumi.Input<string>;
    /**
     * The parameters for the creation of the dashboard, which you want to use to override the default settings. A dashboard can have any type of parameters, and some parameters might accept multiple values. See parameters.
     */
    parameters?: pulumi.Input<inputs.quicksight.DashboardParameters>;
    /**
     * A set of resource permissions on the dashboard. Maximum of 64 items. See permissions.
     */
    permissions?: pulumi.Input<pulumi.Input<inputs.quicksight.DashboardPermission>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The entity that you are using as a source when you create the dashboard (template). Only one of `definition` or `sourceEntity` should be configured. See source_entity.
     */
    sourceEntity?: pulumi.Input<inputs.quicksight.DashboardSourceEntity>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The Amazon Resource Name (ARN) of the theme that is being used for this dashboard. The theme ARN must exist in the same AWS account where you create the dashboard.
     */
    themeArn?: pulumi.Input<string>;
    /**
     * A description of the current dashboard version being created/updated.
     *
     * The following arguments are optional:
     */
    versionDescription: pulumi.Input<string>;
}
