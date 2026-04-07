import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a SageMaker AI Pipeline resource.
 *
 * ## Example Usage
 *
 * ### Basic usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.sagemaker.Pipeline("example", {
 *     pipelineName: "example",
 *     pipelineDisplayName: "example",
 *     roleArn: exampleAwsIamRole.arn,
 *     pipelineDefinition: JSON.stringify({
 *         Version: "2020-12-01",
 *         Steps: [{
 *             Name: "Test",
 *             Type: "Fail",
 *             Arguments: {
 *                 ErrorMessage: "test",
 *             },
 *         }],
 *     }),
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import pipelines using the `pipeline_name`. For example:
 *
 * ```sh
 * $ pulumi import aws:sagemaker/pipeline:Pipeline test_pipeline pipeline
 * ```
 */
export declare class Pipeline extends pulumi.CustomResource {
    /**
     * Get an existing Pipeline resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: PipelineState, opts?: pulumi.CustomResourceOptions): Pipeline;
    /**
     * Returns true if the given object is an instance of Pipeline.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Pipeline;
    /**
     * The Amazon Resource Name (ARN) assigned by AWS to this Pipeline.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * This is the configuration that controls the parallelism of the pipeline. If specified, it applies to all runs of this pipeline by default. see Parallelism Configuration details below.
     */
    readonly parallelismConfiguration: pulumi.Output<outputs.sagemaker.PipelineParallelismConfiguration | undefined>;
    /**
     * The [JSON pipeline definition](https://aws-sagemaker-mlops.github.io/sagemaker-model-building-pipeline-definition-JSON-schema/) of the pipeline.
     */
    readonly pipelineDefinition: pulumi.Output<string | undefined>;
    /**
     * The location of the pipeline definition stored in Amazon S3. If specified, SageMaker AI will retrieve the pipeline definition from this location. see Pipeline Definition S3 Location details below.
     */
    readonly pipelineDefinitionS3Location: pulumi.Output<outputs.sagemaker.PipelinePipelineDefinitionS3Location | undefined>;
    /**
     * A description of the pipeline.
     */
    readonly pipelineDescription: pulumi.Output<string | undefined>;
    /**
     * The display name of the pipeline.
     */
    readonly pipelineDisplayName: pulumi.Output<string>;
    /**
     * The name of the pipeline.
     */
    readonly pipelineName: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The ARN of the IAM role the pipeline will execute as.
     */
    readonly roleArn: pulumi.Output<string | undefined>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Create a Pipeline resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: PipelineArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Pipeline resources.
 */
export interface PipelineState {
    /**
     * The Amazon Resource Name (ARN) assigned by AWS to this Pipeline.
     */
    arn?: pulumi.Input<string>;
    /**
     * This is the configuration that controls the parallelism of the pipeline. If specified, it applies to all runs of this pipeline by default. see Parallelism Configuration details below.
     */
    parallelismConfiguration?: pulumi.Input<inputs.sagemaker.PipelineParallelismConfiguration>;
    /**
     * The [JSON pipeline definition](https://aws-sagemaker-mlops.github.io/sagemaker-model-building-pipeline-definition-JSON-schema/) of the pipeline.
     */
    pipelineDefinition?: pulumi.Input<string>;
    /**
     * The location of the pipeline definition stored in Amazon S3. If specified, SageMaker AI will retrieve the pipeline definition from this location. see Pipeline Definition S3 Location details below.
     */
    pipelineDefinitionS3Location?: pulumi.Input<inputs.sagemaker.PipelinePipelineDefinitionS3Location>;
    /**
     * A description of the pipeline.
     */
    pipelineDescription?: pulumi.Input<string>;
    /**
     * The display name of the pipeline.
     */
    pipelineDisplayName?: pulumi.Input<string>;
    /**
     * The name of the pipeline.
     */
    pipelineName?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The ARN of the IAM role the pipeline will execute as.
     */
    roleArn?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
 * The set of arguments for constructing a Pipeline resource.
 */
export interface PipelineArgs {
    /**
     * This is the configuration that controls the parallelism of the pipeline. If specified, it applies to all runs of this pipeline by default. see Parallelism Configuration details below.
     */
    parallelismConfiguration?: pulumi.Input<inputs.sagemaker.PipelineParallelismConfiguration>;
    /**
     * The [JSON pipeline definition](https://aws-sagemaker-mlops.github.io/sagemaker-model-building-pipeline-definition-JSON-schema/) of the pipeline.
     */
    pipelineDefinition?: pulumi.Input<string>;
    /**
     * The location of the pipeline definition stored in Amazon S3. If specified, SageMaker AI will retrieve the pipeline definition from this location. see Pipeline Definition S3 Location details below.
     */
    pipelineDefinitionS3Location?: pulumi.Input<inputs.sagemaker.PipelinePipelineDefinitionS3Location>;
    /**
     * A description of the pipeline.
     */
    pipelineDescription?: pulumi.Input<string>;
    /**
     * The display name of the pipeline.
     */
    pipelineDisplayName: pulumi.Input<string>;
    /**
     * The name of the pipeline.
     */
    pipelineName: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The ARN of the IAM role the pipeline will execute as.
     */
    roleArn?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
