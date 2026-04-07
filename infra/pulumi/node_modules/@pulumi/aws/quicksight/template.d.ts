import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing a QuickSight Template.
 *
 * ## Example Usage
 *
 * ### From Source Template
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.quicksight.Template("example", {
 *     templateId: "example-id",
 *     name: "example-name",
 *     versionDescription: "version",
 *     sourceEntity: {
 *         sourceTemplate: {
 *             arn: source.arn,
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
 * const example = new aws.quicksight.Template("example", {
 *     templateId: "example-id",
 *     name: "example-name",
 *     versionDescription: "version",
 *     definition: {
 *         dataSetConfigurations: [{
 *             dataSetSchema: {
 *                 columnSchemaLists: [
 *                     {
 *                         name: "Column1",
 *                         dataType: "STRING",
 *                     },
 *                     {
 *                         name: "Column2",
 *                         dataType: "INTEGER",
 *                     },
 *                 ],
 *             },
 *             placeholder: "1",
 *         }],
 *         sheets: [{
 *             title: "Test",
 *             sheetId: "Test1",
 *             visuals: [{
 *                 barChartVisual: {
 *                     visualId: "BarChart",
 *                     chartConfiguration: {
 *                         fieldWells: {
 *                             barChartAggregatedFieldWells: {
 *                                 categories: [{
 *                                     categoricalDimensionField: {
 *                                         fieldId: "1",
 *                                         column: {
 *                                             columnName: "Column1",
 *                                             dataSetIdentifier: "1",
 *                                         },
 *                                     },
 *                                 }],
 *                                 values: [{
 *                                     numericalMeasureField: {
 *                                         fieldId: "2",
 *                                         column: {
 *                                             columnName: "Column2",
 *                                             dataSetIdentifier: "1",
 *                                         },
 *                                         aggregationFunction: {
 *                                             simpleNumericalAggregation: "SUM",
 *                                         },
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
 * Using `pulumi import`, import a QuickSight Template using the AWS account ID and template ID separated by a comma (`,`). For example:
 *
 * ```sh
 * $ pulumi import aws:quicksight/template:Template example 123456789012,example-id
 * ```
 */
export declare class Template extends pulumi.CustomResource {
    /**
     * Get an existing Template resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: TemplateState, opts?: pulumi.CustomResourceOptions): Template;
    /**
     * Returns true if the given object is an instance of Template.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Template;
    /**
     * ARN of the template.
     */
    readonly arn: pulumi.Output<string>;
    readonly awsAccountId: pulumi.Output<string>;
    /**
     * The time that the template was created.
     */
    readonly createdTime: pulumi.Output<string>;
    /**
     * The time that the template was last updated.
     */
    readonly lastUpdatedTime: pulumi.Output<string>;
    /**
     * Display name for the template.
     */
    readonly name: pulumi.Output<string>;
    /**
     * A set of resource permissions on the template. Maximum of 64 items. See permissions.
     */
    readonly permissions: pulumi.Output<outputs.quicksight.TemplatePermission[] | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The entity that you are using as a source when you create the template (analysis or template). Only one of `definition` or `sourceEntity` should be configured. See source_entity.
     */
    readonly sourceEntity: pulumi.Output<outputs.quicksight.TemplateSourceEntity | undefined>;
    /**
     * Amazon Resource Name (ARN) of an analysis or template that was used to create this template.
     */
    readonly sourceEntityArn: pulumi.Output<string>;
    /**
     * The template creation status.
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
     * Identifier for the template.
     */
    readonly templateId: pulumi.Output<string>;
    /**
     * A description of the current template version being created/updated.
     *
     * The following arguments are optional:
     */
    readonly versionDescription: pulumi.Output<string>;
    /**
     * The version number of the template version.
     */
    readonly versionNumber: pulumi.Output<number>;
    /**
     * Create a Template resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: TemplateArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Template resources.
 */
export interface TemplateState {
    /**
     * ARN of the template.
     */
    arn?: pulumi.Input<string>;
    awsAccountId?: pulumi.Input<string>;
    /**
     * The time that the template was created.
     */
    createdTime?: pulumi.Input<string>;
    /**
     * The time that the template was last updated.
     */
    lastUpdatedTime?: pulumi.Input<string>;
    /**
     * Display name for the template.
     */
    name?: pulumi.Input<string>;
    /**
     * A set of resource permissions on the template. Maximum of 64 items. See permissions.
     */
    permissions?: pulumi.Input<pulumi.Input<inputs.quicksight.TemplatePermission>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The entity that you are using as a source when you create the template (analysis or template). Only one of `definition` or `sourceEntity` should be configured. See source_entity.
     */
    sourceEntity?: pulumi.Input<inputs.quicksight.TemplateSourceEntity>;
    /**
     * Amazon Resource Name (ARN) of an analysis or template that was used to create this template.
     */
    sourceEntityArn?: pulumi.Input<string>;
    /**
     * The template creation status.
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
     * Identifier for the template.
     */
    templateId?: pulumi.Input<string>;
    /**
     * A description of the current template version being created/updated.
     *
     * The following arguments are optional:
     */
    versionDescription?: pulumi.Input<string>;
    /**
     * The version number of the template version.
     */
    versionNumber?: pulumi.Input<number>;
}
/**
 * The set of arguments for constructing a Template resource.
 */
export interface TemplateArgs {
    awsAccountId?: pulumi.Input<string>;
    /**
     * Display name for the template.
     */
    name?: pulumi.Input<string>;
    /**
     * A set of resource permissions on the template. Maximum of 64 items. See permissions.
     */
    permissions?: pulumi.Input<pulumi.Input<inputs.quicksight.TemplatePermission>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The entity that you are using as a source when you create the template (analysis or template). Only one of `definition` or `sourceEntity` should be configured. See source_entity.
     */
    sourceEntity?: pulumi.Input<inputs.quicksight.TemplateSourceEntity>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Identifier for the template.
     */
    templateId: pulumi.Input<string>;
    /**
     * A description of the current template version being created/updated.
     *
     * The following arguments are optional:
     */
    versionDescription: pulumi.Input<string>;
}
