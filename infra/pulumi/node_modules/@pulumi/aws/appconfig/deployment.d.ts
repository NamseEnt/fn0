import * as pulumi from "@pulumi/pulumi";
/**
 * Provides an AppConfig Deployment resource for an `aws.appconfig.Application` resource.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.appconfig.Deployment("example", {
 *     applicationId: exampleAwsAppconfigApplication.id,
 *     configurationProfileId: exampleAwsAppconfigConfigurationProfile.configurationProfileId,
 *     configurationVersion: exampleAwsAppconfigHostedConfigurationVersion.versionNumber,
 *     deploymentStrategyId: exampleAwsAppconfigDeploymentStrategy.id,
 *     description: "My example deployment",
 *     environmentId: exampleAwsAppconfigEnvironment.environmentId,
 *     kmsKeyIdentifier: exampleAwsKmsKey.arn,
 *     tags: {
 *         Type: "AppConfig Deployment",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import AppConfig Deployments using the application ID, environment ID, and deployment number separated by a slash (`/`). For example:
 *
 * ```sh
 * $ pulumi import aws:appconfig/deployment:Deployment example 71abcde/11xxxxx/1
 * ```
 */
export declare class Deployment extends pulumi.CustomResource {
    /**
     * Get an existing Deployment resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: DeploymentState, opts?: pulumi.CustomResourceOptions): Deployment;
    /**
     * Returns true if the given object is an instance of Deployment.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Deployment;
    /**
     * Application ID. Must be between 4 and 7 characters in length.
     */
    readonly applicationId: pulumi.Output<string>;
    /**
     * ARN of the AppConfig Deployment.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Configuration profile ID. Must be between 4 and 7 characters in length.
     */
    readonly configurationProfileId: pulumi.Output<string>;
    /**
     * Configuration version to deploy. Can be at most 1024 characters.
     */
    readonly configurationVersion: pulumi.Output<string>;
    /**
     * Deployment number.
     */
    readonly deploymentNumber: pulumi.Output<number>;
    /**
     * Deployment strategy ID or name of a predefined deployment strategy. See [Predefined Deployment Strategies](https://docs.aws.amazon.com/appconfig/latest/userguide/appconfig-creating-deployment-strategy.html#appconfig-creating-deployment-strategy-predefined) for more details.
     */
    readonly deploymentStrategyId: pulumi.Output<string>;
    /**
     * Description of the deployment. Can be at most 1024 characters.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * Environment ID. Must be between 4 and 7 characters in length.
     */
    readonly environmentId: pulumi.Output<string>;
    /**
     * ARN of the KMS key used to encrypt configuration data.
     */
    readonly kmsKeyArn: pulumi.Output<string>;
    /**
     * The KMS key identifier (key ID, key alias, or key ARN). AppConfig uses this to encrypt the configuration data using a customer managed key.
     */
    readonly kmsKeyIdentifier: pulumi.Output<string | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * State of the deployment.
     */
    readonly state: pulumi.Output<string>;
    /**
     * Map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Create a Deployment resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: DeploymentArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Deployment resources.
 */
export interface DeploymentState {
    /**
     * Application ID. Must be between 4 and 7 characters in length.
     */
    applicationId?: pulumi.Input<string>;
    /**
     * ARN of the AppConfig Deployment.
     */
    arn?: pulumi.Input<string>;
    /**
     * Configuration profile ID. Must be between 4 and 7 characters in length.
     */
    configurationProfileId?: pulumi.Input<string>;
    /**
     * Configuration version to deploy. Can be at most 1024 characters.
     */
    configurationVersion?: pulumi.Input<string>;
    /**
     * Deployment number.
     */
    deploymentNumber?: pulumi.Input<number>;
    /**
     * Deployment strategy ID or name of a predefined deployment strategy. See [Predefined Deployment Strategies](https://docs.aws.amazon.com/appconfig/latest/userguide/appconfig-creating-deployment-strategy.html#appconfig-creating-deployment-strategy-predefined) for more details.
     */
    deploymentStrategyId?: pulumi.Input<string>;
    /**
     * Description of the deployment. Can be at most 1024 characters.
     */
    description?: pulumi.Input<string>;
    /**
     * Environment ID. Must be between 4 and 7 characters in length.
     */
    environmentId?: pulumi.Input<string>;
    /**
     * ARN of the KMS key used to encrypt configuration data.
     */
    kmsKeyArn?: pulumi.Input<string>;
    /**
     * The KMS key identifier (key ID, key alias, or key ARN). AppConfig uses this to encrypt the configuration data using a customer managed key.
     */
    kmsKeyIdentifier?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * State of the deployment.
     */
    state?: pulumi.Input<string>;
    /**
     * Map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
 * The set of arguments for constructing a Deployment resource.
 */
export interface DeploymentArgs {
    /**
     * Application ID. Must be between 4 and 7 characters in length.
     */
    applicationId: pulumi.Input<string>;
    /**
     * Configuration profile ID. Must be between 4 and 7 characters in length.
     */
    configurationProfileId: pulumi.Input<string>;
    /**
     * Configuration version to deploy. Can be at most 1024 characters.
     */
    configurationVersion: pulumi.Input<string>;
    /**
     * Deployment strategy ID or name of a predefined deployment strategy. See [Predefined Deployment Strategies](https://docs.aws.amazon.com/appconfig/latest/userguide/appconfig-creating-deployment-strategy.html#appconfig-creating-deployment-strategy-predefined) for more details.
     */
    deploymentStrategyId: pulumi.Input<string>;
    /**
     * Description of the deployment. Can be at most 1024 characters.
     */
    description?: pulumi.Input<string>;
    /**
     * Environment ID. Must be between 4 and 7 characters in length.
     */
    environmentId: pulumi.Input<string>;
    /**
     * The KMS key identifier (key ID, key alias, or key ARN). AppConfig uses this to encrypt the configuration data using a customer managed key.
     */
    kmsKeyIdentifier?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
