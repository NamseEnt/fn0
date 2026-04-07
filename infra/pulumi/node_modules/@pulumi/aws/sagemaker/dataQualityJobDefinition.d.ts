import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a SageMaker AI data quality job definition resource.
 *
 * ## Example Usage
 *
 * Basic usage:
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = new aws.sagemaker.DataQualityJobDefinition("test", {
 *     name: "my-data-quality-job-definition",
 *     dataQualityAppSpecification: {
 *         imageUri: monitor.registryPath,
 *     },
 *     dataQualityJobInput: {
 *         endpointInput: {
 *             endpointName: myEndpoint.name,
 *         },
 *     },
 *     dataQualityJobOutputConfig: {
 *         monitoringOutputs: {
 *             s3Output: {
 *                 s3Uri: `https://${myBucket.bucketRegionalDomainName}/output`,
 *             },
 *         },
 *     },
 *     jobResources: {
 *         clusterConfig: {
 *             instanceCount: 1,
 *             instanceType: "ml.t3.medium",
 *             volumeSizeInGb: 20,
 *         },
 *     },
 *     roleArn: myRole.arn,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import data quality job definitions using the `name`. For example:
 *
 * ```sh
 * $ pulumi import aws:sagemaker/dataQualityJobDefinition:DataQualityJobDefinition test_data_quality_job_definition data-quality-job-definition-foo
 * ```
 */
export declare class DataQualityJobDefinition extends pulumi.CustomResource {
    /**
     * Get an existing DataQualityJobDefinition resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: DataQualityJobDefinitionState, opts?: pulumi.CustomResourceOptions): DataQualityJobDefinition;
    /**
     * Returns true if the given object is an instance of DataQualityJobDefinition.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is DataQualityJobDefinition;
    /**
     * The Amazon Resource Name (ARN) assigned by AWS to this data quality job definition.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Specifies the container that runs the monitoring job. Fields are documented below.
     */
    readonly dataQualityAppSpecification: pulumi.Output<outputs.sagemaker.DataQualityJobDefinitionDataQualityAppSpecification>;
    /**
     * Configures the constraints and baselines for the monitoring job. Fields are documented below.
     */
    readonly dataQualityBaselineConfig: pulumi.Output<outputs.sagemaker.DataQualityJobDefinitionDataQualityBaselineConfig | undefined>;
    /**
     * A list of inputs for the monitoring job. Fields are documented below.
     */
    readonly dataQualityJobInput: pulumi.Output<outputs.sagemaker.DataQualityJobDefinitionDataQualityJobInput>;
    /**
     * The output configuration for monitoring jobs. Fields are documented below.
     */
    readonly dataQualityJobOutputConfig: pulumi.Output<outputs.sagemaker.DataQualityJobDefinitionDataQualityJobOutputConfig>;
    /**
     * Identifies the resources to deploy for a monitoring job. Fields are documented below.
     */
    readonly jobResources: pulumi.Output<outputs.sagemaker.DataQualityJobDefinitionJobResources>;
    /**
     * The name of the data quality job definition. If omitted, the provider will assign a random, unique name.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Specifies networking configuration for the monitoring job. Fields are documented below.
     */
    readonly networkConfig: pulumi.Output<outputs.sagemaker.DataQualityJobDefinitionNetworkConfig | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The Amazon Resource Name (ARN) of an IAM role that Amazon SageMaker AI can assume to perform tasks on your behalf.
     */
    readonly roleArn: pulumi.Output<string>;
    /**
     * A time limit for how long the monitoring job is allowed to run before stopping. Fields are documented below.
     */
    readonly stoppingCondition: pulumi.Output<outputs.sagemaker.DataQualityJobDefinitionStoppingCondition>;
    /**
     * A mapping of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Create a DataQualityJobDefinition resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: DataQualityJobDefinitionArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering DataQualityJobDefinition resources.
 */
export interface DataQualityJobDefinitionState {
    /**
     * The Amazon Resource Name (ARN) assigned by AWS to this data quality job definition.
     */
    arn?: pulumi.Input<string>;
    /**
     * Specifies the container that runs the monitoring job. Fields are documented below.
     */
    dataQualityAppSpecification?: pulumi.Input<inputs.sagemaker.DataQualityJobDefinitionDataQualityAppSpecification>;
    /**
     * Configures the constraints and baselines for the monitoring job. Fields are documented below.
     */
    dataQualityBaselineConfig?: pulumi.Input<inputs.sagemaker.DataQualityJobDefinitionDataQualityBaselineConfig>;
    /**
     * A list of inputs for the monitoring job. Fields are documented below.
     */
    dataQualityJobInput?: pulumi.Input<inputs.sagemaker.DataQualityJobDefinitionDataQualityJobInput>;
    /**
     * The output configuration for monitoring jobs. Fields are documented below.
     */
    dataQualityJobOutputConfig?: pulumi.Input<inputs.sagemaker.DataQualityJobDefinitionDataQualityJobOutputConfig>;
    /**
     * Identifies the resources to deploy for a monitoring job. Fields are documented below.
     */
    jobResources?: pulumi.Input<inputs.sagemaker.DataQualityJobDefinitionJobResources>;
    /**
     * The name of the data quality job definition. If omitted, the provider will assign a random, unique name.
     */
    name?: pulumi.Input<string>;
    /**
     * Specifies networking configuration for the monitoring job. Fields are documented below.
     */
    networkConfig?: pulumi.Input<inputs.sagemaker.DataQualityJobDefinitionNetworkConfig>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) of an IAM role that Amazon SageMaker AI can assume to perform tasks on your behalf.
     */
    roleArn?: pulumi.Input<string>;
    /**
     * A time limit for how long the monitoring job is allowed to run before stopping. Fields are documented below.
     */
    stoppingCondition?: pulumi.Input<inputs.sagemaker.DataQualityJobDefinitionStoppingCondition>;
    /**
     * A mapping of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
 * The set of arguments for constructing a DataQualityJobDefinition resource.
 */
export interface DataQualityJobDefinitionArgs {
    /**
     * Specifies the container that runs the monitoring job. Fields are documented below.
     */
    dataQualityAppSpecification: pulumi.Input<inputs.sagemaker.DataQualityJobDefinitionDataQualityAppSpecification>;
    /**
     * Configures the constraints and baselines for the monitoring job. Fields are documented below.
     */
    dataQualityBaselineConfig?: pulumi.Input<inputs.sagemaker.DataQualityJobDefinitionDataQualityBaselineConfig>;
    /**
     * A list of inputs for the monitoring job. Fields are documented below.
     */
    dataQualityJobInput: pulumi.Input<inputs.sagemaker.DataQualityJobDefinitionDataQualityJobInput>;
    /**
     * The output configuration for monitoring jobs. Fields are documented below.
     */
    dataQualityJobOutputConfig: pulumi.Input<inputs.sagemaker.DataQualityJobDefinitionDataQualityJobOutputConfig>;
    /**
     * Identifies the resources to deploy for a monitoring job. Fields are documented below.
     */
    jobResources: pulumi.Input<inputs.sagemaker.DataQualityJobDefinitionJobResources>;
    /**
     * The name of the data quality job definition. If omitted, the provider will assign a random, unique name.
     */
    name?: pulumi.Input<string>;
    /**
     * Specifies networking configuration for the monitoring job. Fields are documented below.
     */
    networkConfig?: pulumi.Input<inputs.sagemaker.DataQualityJobDefinitionNetworkConfig>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) of an IAM role that Amazon SageMaker AI can assume to perform tasks on your behalf.
     */
    roleArn: pulumi.Input<string>;
    /**
     * A time limit for how long the monitoring job is allowed to run before stopping. Fields are documented below.
     */
    stoppingCondition?: pulumi.Input<inputs.sagemaker.DataQualityJobDefinitionStoppingCondition>;
    /**
     * A mapping of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
