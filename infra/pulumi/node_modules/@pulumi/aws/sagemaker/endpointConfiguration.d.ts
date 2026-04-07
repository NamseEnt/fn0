import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a SageMaker AI endpoint configuration resource.
 *
 * ## Example Usage
 *
 * Basic usage:
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const ec = new aws.sagemaker.EndpointConfiguration("ec", {
 *     name: "my-endpoint-config",
 *     productionVariants: [{
 *         variantName: "variant-1",
 *         modelName: m.name,
 *         initialInstanceCount: 1,
 *         instanceType: "ml.t2.medium",
 *     }],
 *     tags: {
 *         Name: "foo",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import endpoint configurations using the `name`. For example:
 *
 * ```sh
 * $ pulumi import aws:sagemaker/endpointConfiguration:EndpointConfiguration test_endpoint_config endpoint-config-foo
 * ```
 */
export declare class EndpointConfiguration extends pulumi.CustomResource {
    /**
     * Get an existing EndpointConfiguration resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: EndpointConfigurationState, opts?: pulumi.CustomResourceOptions): EndpointConfiguration;
    /**
     * Returns true if the given object is an instance of EndpointConfiguration.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is EndpointConfiguration;
    /**
     * ARN assigned by AWS to this endpoint configuration.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * How an endpoint performs asynchronous inference.
     */
    readonly asyncInferenceConfig: pulumi.Output<outputs.sagemaker.EndpointConfigurationAsyncInferenceConfig | undefined>;
    /**
     * Parameters to capture input/output of SageMaker AI models endpoints. Fields are documented below.
     */
    readonly dataCaptureConfig: pulumi.Output<outputs.sagemaker.EndpointConfigurationDataCaptureConfig | undefined>;
    /**
     * ARN of an IAM role that SageMaker AI can assume to perform actions on your behalf. Required when `modelName` is not specified in `productionVariants` to support Inference Components.
     */
    readonly executionRoleArn: pulumi.Output<string | undefined>;
    /**
     * ARN of a AWS KMS key that SageMaker AI uses to encrypt data on the storage volume attached to the ML compute instance that hosts the endpoint.
     */
    readonly kmsKeyArn: pulumi.Output<string | undefined>;
    /**
     * Name of the endpoint configuration. If omitted, the provider will assign a random, unique name. Conflicts with `namePrefix`.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Unique endpoint configuration name beginning with the specified prefix. Conflicts with `name`.
     */
    readonly namePrefix: pulumi.Output<string>;
    /**
     * List each model that you want to host at this endpoint. See below.
     */
    readonly productionVariants: pulumi.Output<outputs.sagemaker.EndpointConfigurationProductionVariant[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Models that you want to host at this endpoint in shadow mode with production traffic replicated from the model specified on `oroductionVariants`. If you use this field, you can only specify one variant for `productionVariants` and one variant for `shadowProductionVariants`. See below (same arguments as `productionVariants`).
     */
    readonly shadowProductionVariants: pulumi.Output<outputs.sagemaker.EndpointConfigurationShadowProductionVariant[] | undefined>;
    /**
     * Mapping of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
    /**
     * Create a EndpointConfiguration resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: EndpointConfigurationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering EndpointConfiguration resources.
 */
export interface EndpointConfigurationState {
    /**
     * ARN assigned by AWS to this endpoint configuration.
     */
    arn?: pulumi.Input<string>;
    /**
     * How an endpoint performs asynchronous inference.
     */
    asyncInferenceConfig?: pulumi.Input<inputs.sagemaker.EndpointConfigurationAsyncInferenceConfig>;
    /**
     * Parameters to capture input/output of SageMaker AI models endpoints. Fields are documented below.
     */
    dataCaptureConfig?: pulumi.Input<inputs.sagemaker.EndpointConfigurationDataCaptureConfig>;
    /**
     * ARN of an IAM role that SageMaker AI can assume to perform actions on your behalf. Required when `modelName` is not specified in `productionVariants` to support Inference Components.
     */
    executionRoleArn?: pulumi.Input<string>;
    /**
     * ARN of a AWS KMS key that SageMaker AI uses to encrypt data on the storage volume attached to the ML compute instance that hosts the endpoint.
     */
    kmsKeyArn?: pulumi.Input<string>;
    /**
     * Name of the endpoint configuration. If omitted, the provider will assign a random, unique name. Conflicts with `namePrefix`.
     */
    name?: pulumi.Input<string>;
    /**
     * Unique endpoint configuration name beginning with the specified prefix. Conflicts with `name`.
     */
    namePrefix?: pulumi.Input<string>;
    /**
     * List each model that you want to host at this endpoint. See below.
     */
    productionVariants?: pulumi.Input<pulumi.Input<inputs.sagemaker.EndpointConfigurationProductionVariant>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Models that you want to host at this endpoint in shadow mode with production traffic replicated from the model specified on `oroductionVariants`. If you use this field, you can only specify one variant for `productionVariants` and one variant for `shadowProductionVariants`. See below (same arguments as `productionVariants`).
     */
    shadowProductionVariants?: pulumi.Input<pulumi.Input<inputs.sagemaker.EndpointConfigurationShadowProductionVariant>[]>;
    /**
     * Mapping of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
}
/**
 * The set of arguments for constructing a EndpointConfiguration resource.
 */
export interface EndpointConfigurationArgs {
    /**
     * How an endpoint performs asynchronous inference.
     */
    asyncInferenceConfig?: pulumi.Input<inputs.sagemaker.EndpointConfigurationAsyncInferenceConfig>;
    /**
     * Parameters to capture input/output of SageMaker AI models endpoints. Fields are documented below.
     */
    dataCaptureConfig?: pulumi.Input<inputs.sagemaker.EndpointConfigurationDataCaptureConfig>;
    /**
     * ARN of an IAM role that SageMaker AI can assume to perform actions on your behalf. Required when `modelName` is not specified in `productionVariants` to support Inference Components.
     */
    executionRoleArn?: pulumi.Input<string>;
    /**
     * ARN of a AWS KMS key that SageMaker AI uses to encrypt data on the storage volume attached to the ML compute instance that hosts the endpoint.
     */
    kmsKeyArn?: pulumi.Input<string>;
    /**
     * Name of the endpoint configuration. If omitted, the provider will assign a random, unique name. Conflicts with `namePrefix`.
     */
    name?: pulumi.Input<string>;
    /**
     * Unique endpoint configuration name beginning with the specified prefix. Conflicts with `name`.
     */
    namePrefix?: pulumi.Input<string>;
    /**
     * List each model that you want to host at this endpoint. See below.
     */
    productionVariants: pulumi.Input<pulumi.Input<inputs.sagemaker.EndpointConfigurationProductionVariant>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Models that you want to host at this endpoint in shadow mode with production traffic replicated from the model specified on `oroductionVariants`. If you use this field, you can only specify one variant for `productionVariants` and one variant for `shadowProductionVariants`. See below (same arguments as `productionVariants`).
     */
    shadowProductionVariants?: pulumi.Input<pulumi.Input<inputs.sagemaker.EndpointConfigurationShadowProductionVariant>[]>;
    /**
     * Mapping of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
