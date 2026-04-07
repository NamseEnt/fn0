import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Data source for managing an AWS OpenSearch Serverless Security Config.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.opensearch.getServerlessSecurityConfig({
 *     id: "saml/12345678912/example",
 * });
 * ```
 */
export declare function getServerlessSecurityConfig(args: GetServerlessSecurityConfigArgs, opts?: pulumi.InvokeOptions): Promise<GetServerlessSecurityConfigResult>;
/**
 * A collection of arguments for invoking getServerlessSecurityConfig.
 */
export interface GetServerlessSecurityConfigArgs {
    /**
     * The unique identifier of the security configuration.
     */
    id: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
    /**
     * SAML options for the security configuration.
     */
    samlOptions?: inputs.opensearch.GetServerlessSecurityConfigSamlOption[];
}
/**
 * A collection of values returned by getServerlessSecurityConfig.
 */
export interface GetServerlessSecurityConfigResult {
    /**
     * The version of the security configuration.
     */
    readonly configVersion: string;
    /**
     * The date the configuration was created.
     */
    readonly createdDate: string;
    /**
     * The description of the security configuration.
     */
    readonly description: string;
    readonly id: string;
    /**
     * The date the configuration was last modified.
     */
    readonly lastModifiedDate: string;
    readonly region: string;
    /**
     * SAML options for the security configuration.
     */
    readonly samlOptions?: outputs.opensearch.GetServerlessSecurityConfigSamlOption[];
    /**
     * The type of security configuration.
     */
    readonly type: string;
}
/**
 * Data source for managing an AWS OpenSearch Serverless Security Config.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.opensearch.getServerlessSecurityConfig({
 *     id: "saml/12345678912/example",
 * });
 * ```
 */
export declare function getServerlessSecurityConfigOutput(args: GetServerlessSecurityConfigOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetServerlessSecurityConfigResult>;
/**
 * A collection of arguments for invoking getServerlessSecurityConfig.
 */
export interface GetServerlessSecurityConfigOutputArgs {
    /**
     * The unique identifier of the security configuration.
     */
    id: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * SAML options for the security configuration.
     */
    samlOptions?: pulumi.Input<pulumi.Input<inputs.opensearch.GetServerlessSecurityConfigSamlOptionArgs>[]>;
}
