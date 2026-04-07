import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a DataPipeline Pipeline Definition resource.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const _default = new aws.datapipeline.Pipeline("default", {name: "tf-pipeline-default"});
 * const example = new aws.datapipeline.PipelineDefinition("example", {
 *     pipelineId: _default.id,
 *     pipelineObjects: [
 *         {
 *             id: "Default",
 *             name: "Default",
 *             fields: [{
 *                 key: "workerGroup",
 *                 stringValue: "workerGroup",
 *             }],
 *         },
 *         {
 *             id: "Schedule",
 *             name: "Schedule",
 *             fields: [
 *                 {
 *                     key: "startDateTime",
 *                     stringValue: "2012-12-12T00:00:00",
 *                 },
 *                 {
 *                     key: "type",
 *                     stringValue: "Schedule",
 *                 },
 *                 {
 *                     key: "period",
 *                     stringValue: "1 hour",
 *                 },
 *                 {
 *                     key: "endDateTime",
 *                     stringValue: "2012-12-21T18:00:00",
 *                 },
 *             ],
 *         },
 *         {
 *             id: "SayHello",
 *             name: "SayHello",
 *             fields: [
 *                 {
 *                     key: "type",
 *                     stringValue: "ShellCommandActivity",
 *                 },
 *                 {
 *                     key: "command",
 *                     stringValue: "echo hello",
 *                 },
 *                 {
 *                     key: "parent",
 *                     stringValue: "Default",
 *                 },
 *                 {
 *                     key: "schedule",
 *                     stringValue: "Schedule",
 *                 },
 *             ],
 *         },
 *     ],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_datapipeline_pipeline_definition` using the id. For example:
 *
 * ```sh
 * $ pulumi import aws:datapipeline/pipelineDefinition:PipelineDefinition example df-1234567890
 * ```
 */
export declare class PipelineDefinition extends pulumi.CustomResource {
    /**
     * Get an existing PipelineDefinition resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: PipelineDefinitionState, opts?: pulumi.CustomResourceOptions): PipelineDefinition;
    /**
     * Returns true if the given object is an instance of PipelineDefinition.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is PipelineDefinition;
    /**
     * Configuration block for the parameter objects used in the pipeline definition. See below
     */
    readonly parameterObjects: pulumi.Output<outputs.datapipeline.PipelineDefinitionParameterObject[] | undefined>;
    /**
     * Configuration block for the parameter values used in the pipeline definition. See below
     */
    readonly parameterValues: pulumi.Output<outputs.datapipeline.PipelineDefinitionParameterValue[] | undefined>;
    /**
     * ID of the pipeline.
     */
    readonly pipelineId: pulumi.Output<string>;
    /**
     * Configuration block for the objects that define the pipeline. See below
     *
     * The following arguments are optional:
     */
    readonly pipelineObjects: pulumi.Output<outputs.datapipeline.PipelineDefinitionPipelineObject[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Create a PipelineDefinition resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: PipelineDefinitionArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering PipelineDefinition resources.
 */
export interface PipelineDefinitionState {
    /**
     * Configuration block for the parameter objects used in the pipeline definition. See below
     */
    parameterObjects?: pulumi.Input<pulumi.Input<inputs.datapipeline.PipelineDefinitionParameterObject>[]>;
    /**
     * Configuration block for the parameter values used in the pipeline definition. See below
     */
    parameterValues?: pulumi.Input<pulumi.Input<inputs.datapipeline.PipelineDefinitionParameterValue>[]>;
    /**
     * ID of the pipeline.
     */
    pipelineId?: pulumi.Input<string>;
    /**
     * Configuration block for the objects that define the pipeline. See below
     *
     * The following arguments are optional:
     */
    pipelineObjects?: pulumi.Input<pulumi.Input<inputs.datapipeline.PipelineDefinitionPipelineObject>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a PipelineDefinition resource.
 */
export interface PipelineDefinitionArgs {
    /**
     * Configuration block for the parameter objects used in the pipeline definition. See below
     */
    parameterObjects?: pulumi.Input<pulumi.Input<inputs.datapipeline.PipelineDefinitionParameterObject>[]>;
    /**
     * Configuration block for the parameter values used in the pipeline definition. See below
     */
    parameterValues?: pulumi.Input<pulumi.Input<inputs.datapipeline.PipelineDefinitionParameterValue>[]>;
    /**
     * ID of the pipeline.
     */
    pipelineId: pulumi.Input<string>;
    /**
     * Configuration block for the objects that define the pipeline. See below
     *
     * The following arguments are optional:
     */
    pipelineObjects: pulumi.Input<pulumi.Input<inputs.datapipeline.PipelineDefinitionPipelineObject>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
