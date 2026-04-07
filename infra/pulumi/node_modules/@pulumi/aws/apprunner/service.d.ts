import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages an App Runner Service.
 *
 * ## Example Usage
 *
 * ### Service with a Code Repository Source
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.apprunner.Service("example", {
 *     serviceName: "example",
 *     sourceConfiguration: {
 *         authenticationConfiguration: {
 *             connectionArn: exampleAwsApprunnerConnection.arn,
 *         },
 *         codeRepository: {
 *             codeConfiguration: {
 *                 codeConfigurationValues: {
 *                     buildCommand: "python setup.py develop",
 *                     port: "8000",
 *                     runtime: "PYTHON_3",
 *                     startCommand: "python runapp.py",
 *                 },
 *                 configurationSource: "API",
 *             },
 *             repositoryUrl: "https://github.com/example/my-example-python-app",
 *             sourceCodeVersion: {
 *                 type: "BRANCH",
 *                 value: "main",
 *             },
 *         },
 *     },
 *     networkConfiguration: {
 *         egressConfiguration: {
 *             egressType: "VPC",
 *             vpcConnectorArn: connector.arn,
 *         },
 *     },
 *     tags: {
 *         Name: "example-apprunner-service",
 *     },
 * });
 * ```
 *
 * ### Service with an Image Repository Source
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.apprunner.Service("example", {
 *     serviceName: "example",
 *     sourceConfiguration: {
 *         imageRepository: {
 *             imageConfiguration: {
 *                 port: "8000",
 *             },
 *             imageIdentifier: "public.ecr.aws/aws-containers/hello-app-runner:latest",
 *             imageRepositoryType: "ECR_PUBLIC",
 *         },
 *         autoDeploymentsEnabled: false,
 *     },
 *     tags: {
 *         Name: "example-apprunner-service",
 *     },
 * });
 * ```
 *
 * ### Service with Observability Configuration
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const exampleObservabilityConfiguration = new aws.apprunner.ObservabilityConfiguration("example", {
 *     observabilityConfigurationName: "example",
 *     traceConfiguration: {
 *         vendor: "AWSXRAY",
 *     },
 * });
 * const example = new aws.apprunner.Service("example", {
 *     serviceName: "example",
 *     observabilityConfiguration: {
 *         observabilityConfigurationArn: exampleObservabilityConfiguration.arn,
 *         observabilityEnabled: true,
 *     },
 *     sourceConfiguration: {
 *         imageRepository: {
 *             imageConfiguration: {
 *                 port: "8000",
 *             },
 *             imageIdentifier: "public.ecr.aws/aws-containers/hello-app-runner:latest",
 *             imageRepositoryType: "ECR_PUBLIC",
 *         },
 *         autoDeploymentsEnabled: false,
 *     },
 *     tags: {
 *         Name: "example-apprunner-service",
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
 * - `arn` (String) Amazon Resource Name (ARN) of the App Runner service.
 *
 * Using `pulumi import`, import App Runner Services using the `arn`. For example:
 *
 * console
 *
 * % pulumi import aws_apprunner_service.example arn:aws:apprunner:us-east-1:1234567890:service/example/0a03292a89764e5882c41d8f991c82fe
 */
export declare class Service extends pulumi.CustomResource {
    /**
     * Get an existing Service resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ServiceState, opts?: pulumi.CustomResourceOptions): Service;
    /**
     * Returns true if the given object is an instance of Service.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Service;
    /**
     * ARN of the App Runner service.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * ARN of an App Runner automatic scaling configuration resource that you want to associate with your service. If not provided, App Runner associates the latest revision of a default auto scaling configuration.
     */
    readonly autoScalingConfigurationArn: pulumi.Output<string>;
    /**
     * An optional custom encryption key that App Runner uses to encrypt the copy of your source repository that it maintains and your service logs. By default, App Runner uses an AWS managed CMK. See Encryption Configuration below for more details.
     */
    readonly encryptionConfiguration: pulumi.Output<outputs.apprunner.ServiceEncryptionConfiguration | undefined>;
    /**
     * Settings of the health check that AWS App Runner performs to monitor the health of your service. See Health Check Configuration below for more details.
     */
    readonly healthCheckConfiguration: pulumi.Output<outputs.apprunner.ServiceHealthCheckConfiguration>;
    /**
     * The runtime configuration of instances (scaling units) of the App Runner service. See Instance Configuration below for more details.
     */
    readonly instanceConfiguration: pulumi.Output<outputs.apprunner.ServiceInstanceConfiguration>;
    /**
     * Configuration settings related to network traffic of the web application that the App Runner service runs. See Network Configuration below for more details.
     */
    readonly networkConfiguration: pulumi.Output<outputs.apprunner.ServiceNetworkConfiguration>;
    /**
     * The observability configuration of your service. See Observability Configuration below for more details.
     */
    readonly observabilityConfiguration: pulumi.Output<outputs.apprunner.ServiceObservabilityConfiguration | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * An alphanumeric ID that App Runner generated for this service. Unique within the AWS Region.
     */
    readonly serviceId: pulumi.Output<string>;
    /**
     * Name of the service.
     */
    readonly serviceName: pulumi.Output<string>;
    /**
     * Subdomain URL that App Runner generated for this service. You can use this URL to access your service web application.
     */
    readonly serviceUrl: pulumi.Output<string>;
    /**
     * The source to deploy to the App Runner service. Can be a code or an image repository. See Source Configuration below for more details.
     *
     * The following arguments are optional:
     */
    readonly sourceConfiguration: pulumi.Output<outputs.apprunner.ServiceSourceConfiguration>;
    /**
     * Current state of the App Runner service.
     */
    readonly status: pulumi.Output<string>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Create a Service resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ServiceArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Service resources.
 */
export interface ServiceState {
    /**
     * ARN of the App Runner service.
     */
    arn?: pulumi.Input<string>;
    /**
     * ARN of an App Runner automatic scaling configuration resource that you want to associate with your service. If not provided, App Runner associates the latest revision of a default auto scaling configuration.
     */
    autoScalingConfigurationArn?: pulumi.Input<string>;
    /**
     * An optional custom encryption key that App Runner uses to encrypt the copy of your source repository that it maintains and your service logs. By default, App Runner uses an AWS managed CMK. See Encryption Configuration below for more details.
     */
    encryptionConfiguration?: pulumi.Input<inputs.apprunner.ServiceEncryptionConfiguration>;
    /**
     * Settings of the health check that AWS App Runner performs to monitor the health of your service. See Health Check Configuration below for more details.
     */
    healthCheckConfiguration?: pulumi.Input<inputs.apprunner.ServiceHealthCheckConfiguration>;
    /**
     * The runtime configuration of instances (scaling units) of the App Runner service. See Instance Configuration below for more details.
     */
    instanceConfiguration?: pulumi.Input<inputs.apprunner.ServiceInstanceConfiguration>;
    /**
     * Configuration settings related to network traffic of the web application that the App Runner service runs. See Network Configuration below for more details.
     */
    networkConfiguration?: pulumi.Input<inputs.apprunner.ServiceNetworkConfiguration>;
    /**
     * The observability configuration of your service. See Observability Configuration below for more details.
     */
    observabilityConfiguration?: pulumi.Input<inputs.apprunner.ServiceObservabilityConfiguration>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * An alphanumeric ID that App Runner generated for this service. Unique within the AWS Region.
     */
    serviceId?: pulumi.Input<string>;
    /**
     * Name of the service.
     */
    serviceName?: pulumi.Input<string>;
    /**
     * Subdomain URL that App Runner generated for this service. You can use this URL to access your service web application.
     */
    serviceUrl?: pulumi.Input<string>;
    /**
     * The source to deploy to the App Runner service. Can be a code or an image repository. See Source Configuration below for more details.
     *
     * The following arguments are optional:
     */
    sourceConfiguration?: pulumi.Input<inputs.apprunner.ServiceSourceConfiguration>;
    /**
     * Current state of the App Runner service.
     */
    status?: pulumi.Input<string>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
 * The set of arguments for constructing a Service resource.
 */
export interface ServiceArgs {
    /**
     * ARN of an App Runner automatic scaling configuration resource that you want to associate with your service. If not provided, App Runner associates the latest revision of a default auto scaling configuration.
     */
    autoScalingConfigurationArn?: pulumi.Input<string>;
    /**
     * An optional custom encryption key that App Runner uses to encrypt the copy of your source repository that it maintains and your service logs. By default, App Runner uses an AWS managed CMK. See Encryption Configuration below for more details.
     */
    encryptionConfiguration?: pulumi.Input<inputs.apprunner.ServiceEncryptionConfiguration>;
    /**
     * Settings of the health check that AWS App Runner performs to monitor the health of your service. See Health Check Configuration below for more details.
     */
    healthCheckConfiguration?: pulumi.Input<inputs.apprunner.ServiceHealthCheckConfiguration>;
    /**
     * The runtime configuration of instances (scaling units) of the App Runner service. See Instance Configuration below for more details.
     */
    instanceConfiguration?: pulumi.Input<inputs.apprunner.ServiceInstanceConfiguration>;
    /**
     * Configuration settings related to network traffic of the web application that the App Runner service runs. See Network Configuration below for more details.
     */
    networkConfiguration?: pulumi.Input<inputs.apprunner.ServiceNetworkConfiguration>;
    /**
     * The observability configuration of your service. See Observability Configuration below for more details.
     */
    observabilityConfiguration?: pulumi.Input<inputs.apprunner.ServiceObservabilityConfiguration>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Name of the service.
     */
    serviceName: pulumi.Input<string>;
    /**
     * The source to deploy to the App Runner service. Can be a code or an image repository. See Source Configuration below for more details.
     *
     * The following arguments are optional:
     */
    sourceConfiguration: pulumi.Input<inputs.apprunner.ServiceSourceConfiguration>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
