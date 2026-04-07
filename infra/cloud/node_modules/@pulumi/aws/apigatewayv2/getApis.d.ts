import * as pulumi from "@pulumi/pulumi";
/**
 * Provides details about multiple Amazon API Gateway Version 2 APIs.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.apigatewayv2.getApis({
 *     protocolType: "HTTP",
 * });
 * ```
 */
export declare function getApis(args?: GetApisArgs, opts?: pulumi.InvokeOptions): Promise<GetApisResult>;
/**
 * A collection of arguments for invoking getApis.
 */
export interface GetApisArgs {
    /**
     * API name.
     */
    name?: string;
    /**
     * API protocol.
     */
    protocolType?: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
    /**
     * Map of tags, each pair of which must exactly match
     * a pair on the desired APIs.
     */
    tags?: {
        [key: string]: string;
    };
}
/**
 * A collection of values returned by getApis.
 */
export interface GetApisResult {
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    /**
     * Set of API identifiers.
     */
    readonly ids: string[];
    readonly name?: string;
    readonly protocolType?: string;
    readonly region: string;
    readonly tags?: {
        [key: string]: string;
    };
}
/**
 * Provides details about multiple Amazon API Gateway Version 2 APIs.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.apigatewayv2.getApis({
 *     protocolType: "HTTP",
 * });
 * ```
 */
export declare function getApisOutput(args?: GetApisOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetApisResult>;
/**
 * A collection of arguments for invoking getApis.
 */
export interface GetApisOutputArgs {
    /**
     * API name.
     */
    name?: pulumi.Input<string>;
    /**
     * API protocol.
     */
    protocolType?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Map of tags, each pair of which must exactly match
     * a pair on the desired APIs.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
