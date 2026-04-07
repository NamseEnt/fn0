import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.bedrockfoundation.getModel({
 *     modelId: "amazon.titan-text-express-v1",
 * });
 * const exampleCustomModel = new aws.bedrock.CustomModel("example", {
 *     customModelName: "example-model",
 *     jobName: "example-job-1",
 *     baseModelIdentifier: example.then(example => example.modelArn),
 *     roleArn: exampleAwsIamRole.arn,
 *     hyperparameters: {
 *         epochCount: "1",
 *         batchSize: "1",
 *         learningRate: "0.005",
 *         learningRateWarmupSteps: "0",
 *     },
 *     outputDataConfig: {
 *         s3Uri: `s3://${output.id}/data/`,
 *     },
 *     trainingDataConfig: {
 *         s3Uri: `s3://${training.id}/data/train.jsonl`,
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * ### Identity Schema
 *
 * #### Required
 *
 * - `arn` (String) Amazon Resource Name (ARN) of the Bedrock custom model.
 *
 * Using `pulumi import`, import Bedrock custom model using the `job_arn`. For example:
 *
 * console
 *
 * % pulumi import aws_bedrock_custom_model.example arn:aws:bedrock:us-west-2:123456789012:model-customization-job/amazon.titan-text-express-v1:0:8k/1y5n57gh5y2e
 */
export declare class CustomModel extends pulumi.CustomResource {
    /**
     * Get an existing CustomModel resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: CustomModelState, opts?: pulumi.CustomResourceOptions): CustomModel;
    /**
     * Returns true if the given object is an instance of CustomModel.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is CustomModel;
    /**
     * The Amazon Resource Name (ARN) of the base model.
     */
    readonly baseModelIdentifier: pulumi.Output<string>;
    /**
     * The ARN of the output model.
     */
    readonly customModelArn: pulumi.Output<string>;
    /**
     * The custom model is encrypted at rest using this key. Specify the key ARN.
     */
    readonly customModelKmsKeyId: pulumi.Output<string | undefined>;
    /**
     * Name for the custom model.
     */
    readonly customModelName: pulumi.Output<string>;
    /**
     * The customization type. Valid values: `FINE_TUNING`, `CONTINUED_PRE_TRAINING`.
     */
    readonly customizationType: pulumi.Output<string>;
    /**
     * [Parameters](https://docs.aws.amazon.com/bedrock/latest/userguide/custom-models-hp.html) related to tuning the model.
     */
    readonly hyperparameters: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * The ARN of the customization job.
     */
    readonly jobArn: pulumi.Output<string>;
    /**
     * A name for the customization job.
     */
    readonly jobName: pulumi.Output<string>;
    /**
     * The status of the customization job. A successful job transitions from `InProgress` to `Completed` when the output model is ready to use.
     */
    readonly jobStatus: pulumi.Output<string>;
    /**
     * S3 location for the output data.
     */
    readonly outputDataConfig: pulumi.Output<outputs.bedrock.CustomModelOutputDataConfig | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The Amazon Resource Name (ARN) of an IAM role that Bedrock can assume to perform tasks on your behalf.
     */
    readonly roleArn: pulumi.Output<string>;
    /**
     * A map of tags to assign to the customization job and custom model. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * Map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    readonly timeouts: pulumi.Output<outputs.bedrock.CustomModelTimeouts | undefined>;
    /**
     * Information about the training dataset.
     */
    readonly trainingDataConfig: pulumi.Output<outputs.bedrock.CustomModelTrainingDataConfig | undefined>;
    /**
     * Metrics associated with the customization job.
     */
    readonly trainingMetrics: pulumi.Output<outputs.bedrock.CustomModelTrainingMetric[]>;
    /**
     * Information about the validation dataset.
     */
    readonly validationDataConfig: pulumi.Output<outputs.bedrock.CustomModelValidationDataConfig | undefined>;
    /**
     * The loss metric for each validator that you provided.
     */
    readonly validationMetrics: pulumi.Output<outputs.bedrock.CustomModelValidationMetric[]>;
    /**
     * Configuration parameters for the private Virtual Private Cloud (VPC) that contains the resources you are using for this job.
     */
    readonly vpcConfig: pulumi.Output<outputs.bedrock.CustomModelVpcConfig | undefined>;
    /**
     * Create a CustomModel resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: CustomModelArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering CustomModel resources.
 */
export interface CustomModelState {
    /**
     * The Amazon Resource Name (ARN) of the base model.
     */
    baseModelIdentifier?: pulumi.Input<string>;
    /**
     * The ARN of the output model.
     */
    customModelArn?: pulumi.Input<string>;
    /**
     * The custom model is encrypted at rest using this key. Specify the key ARN.
     */
    customModelKmsKeyId?: pulumi.Input<string>;
    /**
     * Name for the custom model.
     */
    customModelName?: pulumi.Input<string>;
    /**
     * The customization type. Valid values: `FINE_TUNING`, `CONTINUED_PRE_TRAINING`.
     */
    customizationType?: pulumi.Input<string>;
    /**
     * [Parameters](https://docs.aws.amazon.com/bedrock/latest/userguide/custom-models-hp.html) related to tuning the model.
     */
    hyperparameters?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The ARN of the customization job.
     */
    jobArn?: pulumi.Input<string>;
    /**
     * A name for the customization job.
     */
    jobName?: pulumi.Input<string>;
    /**
     * The status of the customization job. A successful job transitions from `InProgress` to `Completed` when the output model is ready to use.
     */
    jobStatus?: pulumi.Input<string>;
    /**
     * S3 location for the output data.
     */
    outputDataConfig?: pulumi.Input<inputs.bedrock.CustomModelOutputDataConfig>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) of an IAM role that Bedrock can assume to perform tasks on your behalf.
     */
    roleArn?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the customization job and custom model. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    timeouts?: pulumi.Input<inputs.bedrock.CustomModelTimeouts>;
    /**
     * Information about the training dataset.
     */
    trainingDataConfig?: pulumi.Input<inputs.bedrock.CustomModelTrainingDataConfig>;
    /**
     * Metrics associated with the customization job.
     */
    trainingMetrics?: pulumi.Input<pulumi.Input<inputs.bedrock.CustomModelTrainingMetric>[]>;
    /**
     * Information about the validation dataset.
     */
    validationDataConfig?: pulumi.Input<inputs.bedrock.CustomModelValidationDataConfig>;
    /**
     * The loss metric for each validator that you provided.
     */
    validationMetrics?: pulumi.Input<pulumi.Input<inputs.bedrock.CustomModelValidationMetric>[]>;
    /**
     * Configuration parameters for the private Virtual Private Cloud (VPC) that contains the resources you are using for this job.
     */
    vpcConfig?: pulumi.Input<inputs.bedrock.CustomModelVpcConfig>;
}
/**
 * The set of arguments for constructing a CustomModel resource.
 */
export interface CustomModelArgs {
    /**
     * The Amazon Resource Name (ARN) of the base model.
     */
    baseModelIdentifier: pulumi.Input<string>;
    /**
     * The custom model is encrypted at rest using this key. Specify the key ARN.
     */
    customModelKmsKeyId?: pulumi.Input<string>;
    /**
     * Name for the custom model.
     */
    customModelName: pulumi.Input<string>;
    /**
     * The customization type. Valid values: `FINE_TUNING`, `CONTINUED_PRE_TRAINING`.
     */
    customizationType?: pulumi.Input<string>;
    /**
     * [Parameters](https://docs.aws.amazon.com/bedrock/latest/userguide/custom-models-hp.html) related to tuning the model.
     */
    hyperparameters: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * A name for the customization job.
     */
    jobName: pulumi.Input<string>;
    /**
     * S3 location for the output data.
     */
    outputDataConfig?: pulumi.Input<inputs.bedrock.CustomModelOutputDataConfig>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) of an IAM role that Bedrock can assume to perform tasks on your behalf.
     */
    roleArn: pulumi.Input<string>;
    /**
     * A map of tags to assign to the customization job and custom model. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    timeouts?: pulumi.Input<inputs.bedrock.CustomModelTimeouts>;
    /**
     * Information about the training dataset.
     */
    trainingDataConfig?: pulumi.Input<inputs.bedrock.CustomModelTrainingDataConfig>;
    /**
     * Information about the validation dataset.
     */
    validationDataConfig?: pulumi.Input<inputs.bedrock.CustomModelValidationDataConfig>;
    /**
     * Configuration parameters for the private Virtual Private Cloud (VPC) that contains the resources you are using for this job.
     */
    vpcConfig?: pulumi.Input<inputs.bedrock.CustomModelVpcConfig>;
}
