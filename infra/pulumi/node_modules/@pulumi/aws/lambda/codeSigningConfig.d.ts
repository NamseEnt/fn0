import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages an AWS Lambda Code Signing Config. Use this resource to define allowed signing profiles and code-signing validation policies for Lambda functions to ensure code integrity and authenticity.
 *
 * For information about Lambda code signing configurations and how to use them, see [configuring code signing for Lambda functions](https://docs.aws.amazon.com/lambda/latest/dg/configuration-codesigning.html).
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * // Create signing profiles for different environments
 * const prod = new aws.signer.SigningProfile("prod", {
 *     platformId: "AWSLambda-SHA384-ECDSA",
 *     namePrefix: "prod_lambda_",
 *     tags: {
 *         Environment: "production",
 *     },
 * });
 * const dev = new aws.signer.SigningProfile("dev", {
 *     platformId: "AWSLambda-SHA384-ECDSA",
 *     namePrefix: "dev_lambda_",
 *     tags: {
 *         Environment: "development",
 *     },
 * });
 * // Code signing configuration with enforcement
 * const example = new aws.lambda.CodeSigningConfig("example", {
 *     description: "Code signing configuration for Lambda functions",
 *     allowedPublishers: {
 *         signingProfileVersionArns: [
 *             prod.versionArn,
 *             dev.versionArn,
 *         ],
 *     },
 *     policies: {
 *         untrustedArtifactOnDeployment: "Enforce",
 *     },
 *     tags: {
 *         Environment: "production",
 *         Purpose: "code-signing",
 *     },
 * });
 * ```
 *
 * ### Warning Only Configuration
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.lambda.CodeSigningConfig("example", {
 *     description: "Development code signing configuration",
 *     allowedPublishers: {
 *         signingProfileVersionArns: [dev.versionArn],
 *     },
 *     policies: {
 *         untrustedArtifactOnDeployment: "Warn",
 *     },
 *     tags: {
 *         Environment: "development",
 *         Purpose: "code-signing",
 *     },
 * });
 * ```
 *
 * ### Multiple Environment Configuration
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * // Production signing configuration
 * const prod = new aws.lambda.CodeSigningConfig("prod", {
 *     description: "Production code signing configuration with strict enforcement",
 *     allowedPublishers: {
 *         signingProfileVersionArns: [prodAwsSignerSigningProfile.versionArn],
 *     },
 *     policies: {
 *         untrustedArtifactOnDeployment: "Enforce",
 *     },
 *     tags: {
 *         Environment: "production",
 *         Security: "strict",
 *     },
 * });
 * // Development signing configuration
 * const dev = new aws.lambda.CodeSigningConfig("dev", {
 *     description: "Development code signing configuration with warnings",
 *     allowedPublishers: {
 *         signingProfileVersionArns: [
 *             devAwsSignerSigningProfile.versionArn,
 *             test.versionArn,
 *         ],
 *     },
 *     policies: {
 *         untrustedArtifactOnDeployment: "Warn",
 *     },
 *     tags: {
 *         Environment: "development",
 *         Security: "flexible",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * For backwards compatibility, the following legacy `pulumi import` command is also supported:
 *
 * ```sh
 * $ pulumi import aws:lambda/codeSigningConfig:CodeSigningConfig example arn:aws:lambda:us-west-2:123456789012:code-signing-config:csc-0f6c334abcdea4d8b
 * ```
 */
export declare class CodeSigningConfig extends pulumi.CustomResource {
    /**
     * Get an existing CodeSigningConfig resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: CodeSigningConfigState, opts?: pulumi.CustomResourceOptions): CodeSigningConfig;
    /**
     * Returns true if the given object is an instance of CodeSigningConfig.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is CodeSigningConfig;
    /**
     * Configuration block of allowed publishers as signing profiles for this code signing configuration. See below.
     *
     * The following arguments are optional:
     */
    readonly allowedPublishers: pulumi.Output<outputs.lambda.CodeSigningConfigAllowedPublishers>;
    /**
     * ARN of the code signing configuration.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Unique identifier for the code signing configuration.
     */
    readonly configId: pulumi.Output<string>;
    /**
     * Descriptive name for this code signing configuration.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * Date and time that the code signing configuration was last modified.
     */
    readonly lastModified: pulumi.Output<string>;
    /**
     * Configuration block of code signing policies that define the actions to take if the validation checks fail. See below.
     */
    readonly policies: pulumi.Output<outputs.lambda.CodeSigningConfigPolicies>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Map of tags to assign to the object. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Create a CodeSigningConfig resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: CodeSigningConfigArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering CodeSigningConfig resources.
 */
export interface CodeSigningConfigState {
    /**
     * Configuration block of allowed publishers as signing profiles for this code signing configuration. See below.
     *
     * The following arguments are optional:
     */
    allowedPublishers?: pulumi.Input<inputs.lambda.CodeSigningConfigAllowedPublishers>;
    /**
     * ARN of the code signing configuration.
     */
    arn?: pulumi.Input<string>;
    /**
     * Unique identifier for the code signing configuration.
     */
    configId?: pulumi.Input<string>;
    /**
     * Descriptive name for this code signing configuration.
     */
    description?: pulumi.Input<string>;
    /**
     * Date and time that the code signing configuration was last modified.
     */
    lastModified?: pulumi.Input<string>;
    /**
     * Configuration block of code signing policies that define the actions to take if the validation checks fail. See below.
     */
    policies?: pulumi.Input<inputs.lambda.CodeSigningConfigPolicies>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Map of tags to assign to the object. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
 * The set of arguments for constructing a CodeSigningConfig resource.
 */
export interface CodeSigningConfigArgs {
    /**
     * Configuration block of allowed publishers as signing profiles for this code signing configuration. See below.
     *
     * The following arguments are optional:
     */
    allowedPublishers: pulumi.Input<inputs.lambda.CodeSigningConfigAllowedPublishers>;
    /**
     * Descriptive name for this code signing configuration.
     */
    description?: pulumi.Input<string>;
    /**
     * Configuration block of code signing policies that define the actions to take if the validation checks fail. See below.
     */
    policies?: pulumi.Input<inputs.lambda.CodeSigningConfigPolicies>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Map of tags to assign to the object. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
