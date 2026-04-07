import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a resource to manage AWS Secrets Manager secret version including its secret value. To manage secret metadata, see the `aws.secretsmanager.Secret` resource.
 *
 * > **NOTE:** If the `AWSCURRENT` staging label is present on this version during resource deletion, that label cannot be removed and will be skipped to prevent errors when fully deleting the secret. That label will leave this secret version active even after the resource is deleted from this provider unless the secret itself is deleted. Move the `AWSCURRENT` staging label before or after deleting this resource from this provider to fully trigger version deprecation if necessary.
 *
 * ## Example Usage
 *
 * ### Simple String Value
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.secretsmanager.SecretVersion("example", {
 *     secretId: exampleAwsSecretsmanagerSecret.id,
 *     secretString: "example-string-to-protect",
 * });
 * ```
 *
 * ### Key-Value Pairs
 *
 * Secrets Manager also accepts key-value pairs in JSON.
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const config = new pulumi.Config();
 * const example = config.getObject<Record<string, string>>("example") || {
 *     key1: "value1",
 *     key2: "value2",
 * };
 * const exampleSecretVersion = new aws.secretsmanager.SecretVersion("example", {
 *     secretId: exampleAwsSecretsmanagerSecret.id,
 *     secretString: JSON.stringify(example),
 * });
 * ```
 *
 * Reading key-value pairs from JSON back into a native map
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as std from "@pulumi/std";
 *
 * export const example = std.jsondecode({
 *     input: exampleAwsSecretsmanagerSecretVersion.secretString,
 * }).then(invoke => invoke.result?.key1);
 * ```
 *
 * ## Import
 *
 * ### Identity Schema
 *
 * #### Required
 *
 * * `secret_id` - (String) ID of the secret.
 *
 * * `version_id` - (String) ID of the secret version.
 *
 * #### Optional
 *
 * * `account_id` (String) AWS Account where this resource is managed.
 *
 * * `region` (String) Region where this resource is managed.
 *
 * Using `pulumi import`, import `aws_secretsmanager_secret_version` using the secret ID and version ID. For example:
 *
 * console
 *
 * % pulumi import aws_secretsmanager_secret_version.example 'arn:aws:secretsmanager:us-east-1:123456789012:secret:example-123456|xxxxx-xxxxxxx-xxxxxxx-xxxxx'
 */
export declare class SecretVersion extends pulumi.CustomResource {
    /**
     * Get an existing SecretVersion resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: SecretVersionState, opts?: pulumi.CustomResourceOptions): SecretVersion;
    /**
     * Returns true if the given object is an instance of SecretVersion.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is SecretVersion;
    /**
     * The ARN of the secret.
     */
    readonly arn: pulumi.Output<string>;
    readonly hasSecretStringWo: pulumi.Output<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Specifies binary data that you want to encrypt and store in this version of the secret. This is required if `secretString` or `secretStringWo` is not set. Needs to be encoded to base64.
     */
    readonly secretBinary: pulumi.Output<string | undefined>;
    /**
     * Specifies the secret to which you want to add a new version. You can specify either the Amazon Resource Name (ARN) or the friendly name of the secret. The secret must already exist.
     */
    readonly secretId: pulumi.Output<string>;
    /**
     * Specifies text data that you want to encrypt and store in this version of the secret. This is required if `secretBinary` or `secretStringWo` is not set.
     */
    readonly secretString: pulumi.Output<string | undefined>;
    /**
     * **NOTE:** This field is write-only and its value will not be updated in state as part of read operations.
     * Specifies text data that you want to encrypt and store in this version of the secret. This is required if `secretBinary` or `secretString` is not set.
     */
    readonly secretStringWo: pulumi.Output<string | undefined>;
    /**
     * Used together with `secretStringWo` to trigger an update. Increment this value when an update to `secretStringWo` is required.
     */
    readonly secretStringWoVersion: pulumi.Output<number | undefined>;
    /**
     * The unique identifier of the version of the secret.
     */
    readonly versionId: pulumi.Output<string>;
    /**
     * Specifies a list of staging labels that are attached to this version of the secret. A staging label must be unique to a single version of the secret. If you specify a staging label that's already associated with a different version of the same secret then that staging label is automatically removed from the other version and attached to this version. If you do not specify a value, then AWS Secrets Manager automatically moves the staging label `AWSCURRENT` to this new version on creation.
     *
     * > **NOTE:** If `versionStages` is configured, you must include the `AWSCURRENT` staging label if this secret version is the only version or if the label is currently present on this secret version, otherwise this provider will show a perpetual difference.
     */
    readonly versionStages: pulumi.Output<string[]>;
    /**
     * Create a SecretVersion resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: SecretVersionArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering SecretVersion resources.
 */
export interface SecretVersionState {
    /**
     * The ARN of the secret.
     */
    arn?: pulumi.Input<string>;
    hasSecretStringWo?: pulumi.Input<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Specifies binary data that you want to encrypt and store in this version of the secret. This is required if `secretString` or `secretStringWo` is not set. Needs to be encoded to base64.
     */
    secretBinary?: pulumi.Input<string>;
    /**
     * Specifies the secret to which you want to add a new version. You can specify either the Amazon Resource Name (ARN) or the friendly name of the secret. The secret must already exist.
     */
    secretId?: pulumi.Input<string>;
    /**
     * Specifies text data that you want to encrypt and store in this version of the secret. This is required if `secretBinary` or `secretStringWo` is not set.
     */
    secretString?: pulumi.Input<string>;
    /**
     * **NOTE:** This field is write-only and its value will not be updated in state as part of read operations.
     * Specifies text data that you want to encrypt and store in this version of the secret. This is required if `secretBinary` or `secretString` is not set.
     */
    secretStringWo?: pulumi.Input<string>;
    /**
     * Used together with `secretStringWo` to trigger an update. Increment this value when an update to `secretStringWo` is required.
     */
    secretStringWoVersion?: pulumi.Input<number>;
    /**
     * The unique identifier of the version of the secret.
     */
    versionId?: pulumi.Input<string>;
    /**
     * Specifies a list of staging labels that are attached to this version of the secret. A staging label must be unique to a single version of the secret. If you specify a staging label that's already associated with a different version of the same secret then that staging label is automatically removed from the other version and attached to this version. If you do not specify a value, then AWS Secrets Manager automatically moves the staging label `AWSCURRENT` to this new version on creation.
     *
     * > **NOTE:** If `versionStages` is configured, you must include the `AWSCURRENT` staging label if this secret version is the only version or if the label is currently present on this secret version, otherwise this provider will show a perpetual difference.
     */
    versionStages?: pulumi.Input<pulumi.Input<string>[]>;
}
/**
 * The set of arguments for constructing a SecretVersion resource.
 */
export interface SecretVersionArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Specifies binary data that you want to encrypt and store in this version of the secret. This is required if `secretString` or `secretStringWo` is not set. Needs to be encoded to base64.
     */
    secretBinary?: pulumi.Input<string>;
    /**
     * Specifies the secret to which you want to add a new version. You can specify either the Amazon Resource Name (ARN) or the friendly name of the secret. The secret must already exist.
     */
    secretId: pulumi.Input<string>;
    /**
     * Specifies text data that you want to encrypt and store in this version of the secret. This is required if `secretBinary` or `secretStringWo` is not set.
     */
    secretString?: pulumi.Input<string>;
    /**
     * **NOTE:** This field is write-only and its value will not be updated in state as part of read operations.
     * Specifies text data that you want to encrypt and store in this version of the secret. This is required if `secretBinary` or `secretString` is not set.
     */
    secretStringWo?: pulumi.Input<string>;
    /**
     * Used together with `secretStringWo` to trigger an update. Increment this value when an update to `secretStringWo` is required.
     */
    secretStringWoVersion?: pulumi.Input<number>;
    /**
     * Specifies a list of staging labels that are attached to this version of the secret. A staging label must be unique to a single version of the secret. If you specify a staging label that's already associated with a different version of the same secret then that staging label is automatically removed from the other version and attached to this version. If you do not specify a value, then AWS Secrets Manager automatically moves the staging label `AWSCURRENT` to this new version on creation.
     *
     * > **NOTE:** If `versionStages` is configured, you must include the `AWSCURRENT` staging label if this secret version is the only version or if the label is currently present on this secret version, otherwise this provider will show a perpetual difference.
     */
    versionStages?: pulumi.Input<pulumi.Input<string>[]>;
}
