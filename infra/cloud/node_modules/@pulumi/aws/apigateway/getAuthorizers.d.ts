import * as pulumi from "@pulumi/pulumi";
/**
 * Provides details about multiple API Gateway Authorizers.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.apigateway.getAuthorizers({
 *     restApiId: exampleAwsApiGatewayRestApi.id,
 * });
 * ```
 */
export declare function getAuthorizers(args: GetAuthorizersArgs, opts?: pulumi.InvokeOptions): Promise<GetAuthorizersResult>;
/**
 * A collection of arguments for invoking getAuthorizers.
 */
export interface GetAuthorizersArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
    /**
     * ID of the associated REST API.
     */
    restApiId: string;
}
/**
 * A collection of values returned by getAuthorizers.
 */
export interface GetAuthorizersResult {
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    /**
     * List of Authorizer identifiers.
     */
    readonly ids: string[];
    readonly region: string;
    readonly restApiId: string;
}
/**
 * Provides details about multiple API Gateway Authorizers.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.apigateway.getAuthorizers({
 *     restApiId: exampleAwsApiGatewayRestApi.id,
 * });
 * ```
 */
export declare function getAuthorizersOutput(args: GetAuthorizersOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetAuthorizersResult>;
/**
 * A collection of arguments for invoking getAuthorizers.
 */
export interface GetAuthorizersOutputArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ID of the associated REST API.
     */
    restApiId: pulumi.Input<string>;
}
