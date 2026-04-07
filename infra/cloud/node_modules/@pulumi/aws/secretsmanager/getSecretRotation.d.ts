import * as pulumi from "@pulumi/pulumi";
import * as outputs from "../types/output";
/**
 * Retrieve information about a Secrets Manager secret rotation. To retrieve secret metadata, see the `aws.secretsmanager.Secret` data source. To retrieve a secret value, see the `aws.secretsmanager.SecretVersion` data source.
 *
 * ## Example Usage
 *
 * ### Retrieve Secret Rotation Configuration
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.secretsmanager.getSecretRotation({
 *     secretId: exampleAwsSecretsmanagerSecret.id,
 * });
 * ```
 */
export declare function getSecretRotation(args: GetSecretRotationArgs, opts?: pulumi.InvokeOptions): Promise<GetSecretRotationResult>;
/**
 * A collection of arguments for invoking getSecretRotation.
 */
export interface GetSecretRotationArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
    /**
     * Specifies the secret containing the version that you want to retrieve. You can specify either the ARN or the friendly name of the secret.
     */
    secretId: string;
}
/**
 * A collection of values returned by getSecretRotation.
 */
export interface GetSecretRotationResult {
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly region: string;
    /**
     * Specifies whether automatic rotation is enabled for this secret.
     */
    readonly rotationEnabled: boolean;
    /**
     * Amazon Resource Name (ARN) of the lambda function used for rotation.
     */
    readonly rotationLambdaArn: string;
    /**
     * Configuration block for rotation rules. See `rotationRules` below.
     */
    readonly rotationRules: outputs.secretsmanager.GetSecretRotationRotationRule[];
    readonly secretId: string;
}
/**
 * Retrieve information about a Secrets Manager secret rotation. To retrieve secret metadata, see the `aws.secretsmanager.Secret` data source. To retrieve a secret value, see the `aws.secretsmanager.SecretVersion` data source.
 *
 * ## Example Usage
 *
 * ### Retrieve Secret Rotation Configuration
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.secretsmanager.getSecretRotation({
 *     secretId: exampleAwsSecretsmanagerSecret.id,
 * });
 * ```
 */
export declare function getSecretRotationOutput(args: GetSecretRotationOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetSecretRotationResult>;
/**
 * A collection of arguments for invoking getSecretRotation.
 */
export interface GetSecretRotationOutputArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Specifies the secret containing the version that you want to retrieve. You can specify either the ARN or the friendly name of the secret.
     */
    secretId: pulumi.Input<string>;
}
