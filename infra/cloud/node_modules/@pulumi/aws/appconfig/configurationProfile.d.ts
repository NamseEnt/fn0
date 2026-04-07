import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides an AppConfig Configuration Profile resource.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.appconfig.ConfigurationProfile("example", {
 *     applicationId: exampleAwsAppconfigApplication.id,
 *     description: "Example Configuration Profile",
 *     name: "example-configuration-profile-tf",
 *     locationUri: "hosted",
 *     validators: [{
 *         content: exampleAwsLambdaFunction.arn,
 *         type: "LAMBDA",
 *     }],
 *     tags: {
 *         Type: "AppConfig Configuration Profile",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import AppConfig Configuration Profiles using the configuration profile ID and application ID separated by a colon (`:`). For example:
 *
 * ```sh
 * $ pulumi import aws:appconfig/configurationProfile:ConfigurationProfile example 71abcde:11xxxxx
 * ```
 */
export declare class ConfigurationProfile extends pulumi.CustomResource {
    /**
     * Get an existing ConfigurationProfile resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ConfigurationProfileState, opts?: pulumi.CustomResourceOptions): ConfigurationProfile;
    /**
     * Returns true if the given object is an instance of ConfigurationProfile.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ConfigurationProfile;
    /**
     * Application ID. Must be between 4 and 7 characters in length.
     */
    readonly applicationId: pulumi.Output<string>;
    /**
     * ARN of the AppConfig Configuration Profile.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The configuration profile ID.
     */
    readonly configurationProfileId: pulumi.Output<string>;
    /**
     * Description of the configuration profile. Can be at most 1024 characters.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * The identifier for an Key Management Service key to encrypt new configuration data versions in the AppConfig hosted configuration store. This attribute is only used for hosted configuration types. The identifier can be an KMS key ID, alias, or the Amazon Resource Name (ARN) of the key ID or alias.
     */
    readonly kmsKeyIdentifier: pulumi.Output<string | undefined>;
    /**
     * URI to locate the configuration. You can specify the AWS AppConfig hosted configuration store, Systems Manager (SSM) document, an SSM Parameter Store parameter, or an Amazon S3 object. For the hosted configuration store, specify `hosted`. For an SSM document, specify either the document name in the format `ssm-document://<Document_name>` or the ARN. For a parameter, specify either the parameter name in the format `ssm-parameter://<Parameter_name>` or the ARN. For an Amazon S3 object, specify the URI in the following format: `s3://<bucket>/<objectKey>`.
     */
    readonly locationUri: pulumi.Output<string>;
    /**
     * Name for the configuration profile. Must be between 1 and 128 characters in length.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * ARN of an IAM role with permission to access the configuration at the specified `locationUri`. A retrieval role ARN is not required for configurations stored in the AWS AppConfig `hosted` configuration store. It is required for all other sources that store your configuration.
     */
    readonly retrievalRoleArn: pulumi.Output<string | undefined>;
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
     * Type of configurations contained in the profile. Valid values: `AWS.AppConfig.FeatureFlags` and `AWS.Freeform`.  Default: `AWS.Freeform`.
     */
    readonly type: pulumi.Output<string | undefined>;
    /**
     * Set of methods for validating the configuration. Maximum of 2. See Validator below for more details.
     */
    readonly validators: pulumi.Output<outputs.appconfig.ConfigurationProfileValidator[] | undefined>;
    /**
     * Create a ConfigurationProfile resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ConfigurationProfileArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ConfigurationProfile resources.
 */
export interface ConfigurationProfileState {
    /**
     * Application ID. Must be between 4 and 7 characters in length.
     */
    applicationId?: pulumi.Input<string>;
    /**
     * ARN of the AppConfig Configuration Profile.
     */
    arn?: pulumi.Input<string>;
    /**
     * The configuration profile ID.
     */
    configurationProfileId?: pulumi.Input<string>;
    /**
     * Description of the configuration profile. Can be at most 1024 characters.
     */
    description?: pulumi.Input<string>;
    /**
     * The identifier for an Key Management Service key to encrypt new configuration data versions in the AppConfig hosted configuration store. This attribute is only used for hosted configuration types. The identifier can be an KMS key ID, alias, or the Amazon Resource Name (ARN) of the key ID or alias.
     */
    kmsKeyIdentifier?: pulumi.Input<string>;
    /**
     * URI to locate the configuration. You can specify the AWS AppConfig hosted configuration store, Systems Manager (SSM) document, an SSM Parameter Store parameter, or an Amazon S3 object. For the hosted configuration store, specify `hosted`. For an SSM document, specify either the document name in the format `ssm-document://<Document_name>` or the ARN. For a parameter, specify either the parameter name in the format `ssm-parameter://<Parameter_name>` or the ARN. For an Amazon S3 object, specify the URI in the following format: `s3://<bucket>/<objectKey>`.
     */
    locationUri?: pulumi.Input<string>;
    /**
     * Name for the configuration profile. Must be between 1 and 128 characters in length.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ARN of an IAM role with permission to access the configuration at the specified `locationUri`. A retrieval role ARN is not required for configurations stored in the AWS AppConfig `hosted` configuration store. It is required for all other sources that store your configuration.
     */
    retrievalRoleArn?: pulumi.Input<string>;
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
    /**
     * Type of configurations contained in the profile. Valid values: `AWS.AppConfig.FeatureFlags` and `AWS.Freeform`.  Default: `AWS.Freeform`.
     */
    type?: pulumi.Input<string>;
    /**
     * Set of methods for validating the configuration. Maximum of 2. See Validator below for more details.
     */
    validators?: pulumi.Input<pulumi.Input<inputs.appconfig.ConfigurationProfileValidator>[]>;
}
/**
 * The set of arguments for constructing a ConfigurationProfile resource.
 */
export interface ConfigurationProfileArgs {
    /**
     * Application ID. Must be between 4 and 7 characters in length.
     */
    applicationId: pulumi.Input<string>;
    /**
     * Description of the configuration profile. Can be at most 1024 characters.
     */
    description?: pulumi.Input<string>;
    /**
     * The identifier for an Key Management Service key to encrypt new configuration data versions in the AppConfig hosted configuration store. This attribute is only used for hosted configuration types. The identifier can be an KMS key ID, alias, or the Amazon Resource Name (ARN) of the key ID or alias.
     */
    kmsKeyIdentifier?: pulumi.Input<string>;
    /**
     * URI to locate the configuration. You can specify the AWS AppConfig hosted configuration store, Systems Manager (SSM) document, an SSM Parameter Store parameter, or an Amazon S3 object. For the hosted configuration store, specify `hosted`. For an SSM document, specify either the document name in the format `ssm-document://<Document_name>` or the ARN. For a parameter, specify either the parameter name in the format `ssm-parameter://<Parameter_name>` or the ARN. For an Amazon S3 object, specify the URI in the following format: `s3://<bucket>/<objectKey>`.
     */
    locationUri: pulumi.Input<string>;
    /**
     * Name for the configuration profile. Must be between 1 and 128 characters in length.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ARN of an IAM role with permission to access the configuration at the specified `locationUri`. A retrieval role ARN is not required for configurations stored in the AWS AppConfig `hosted` configuration store. It is required for all other sources that store your configuration.
     */
    retrievalRoleArn?: pulumi.Input<string>;
    /**
     * Map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Type of configurations contained in the profile. Valid values: `AWS.AppConfig.FeatureFlags` and `AWS.Freeform`.  Default: `AWS.Freeform`.
     */
    type?: pulumi.Input<string>;
    /**
     * Set of methods for validating the configuration. Maximum of 2. See Validator below for more details.
     */
    validators?: pulumi.Input<pulumi.Input<inputs.appconfig.ConfigurationProfileValidator>[]>;
}
