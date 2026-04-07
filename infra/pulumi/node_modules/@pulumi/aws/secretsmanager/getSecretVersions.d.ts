import * as pulumi from "@pulumi/pulumi";
import * as outputs from "../types/output";
export declare function getSecretVersions(args: GetSecretVersionsArgs, opts?: pulumi.InvokeOptions): Promise<GetSecretVersionsResult>;
/**
 * A collection of arguments for invoking getSecretVersions.
 */
export interface GetSecretVersionsArgs {
    /**
     * If true, all deprecated secret versions are included in the response.
     * If false, no deprecated secret versions are included in the response. If no value is specified, the default value is `false`.
     */
    includeDeprecated?: boolean;
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
 * A collection of values returned by getSecretVersions.
 */
export interface GetSecretVersionsResult {
    /**
     * ARN of the secret.
     */
    readonly arn: string;
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly includeDeprecated?: boolean;
    readonly name: string;
    readonly region: string;
    readonly secretId: string;
    /**
     * List of the versions of the secret. Attributes are specified below.
     */
    readonly versions: outputs.secretsmanager.GetSecretVersionsVersion[];
}
export declare function getSecretVersionsOutput(args: GetSecretVersionsOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetSecretVersionsResult>;
/**
 * A collection of arguments for invoking getSecretVersions.
 */
export interface GetSecretVersionsOutputArgs {
    /**
     * If true, all deprecated secret versions are included in the response.
     * If false, no deprecated secret versions are included in the response. If no value is specified, the default value is `false`.
     */
    includeDeprecated?: pulumi.Input<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Specifies the secret containing the version that you want to retrieve. You can specify either the ARN or the friendly name of the secret.
     */
    secretId: pulumi.Input<string>;
}
