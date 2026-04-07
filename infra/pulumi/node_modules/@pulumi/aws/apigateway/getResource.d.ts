import * as pulumi from "@pulumi/pulumi";
/**
 * Use this data source to get the id of a Resource in API Gateway.
 * To fetch the Resource, you must provide the REST API id as well as the full path.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const myRestApi = aws.apigateway.getRestApi({
 *     name: "my-rest-api",
 * });
 * const myResource = myRestApi.then(myRestApi => aws.apigateway.getResource({
 *     restApiId: myRestApi.id,
 *     path: "/endpoint/path",
 * }));
 * ```
 */
export declare function getResource(args: GetResourceArgs, opts?: pulumi.InvokeOptions): Promise<GetResourceResult>;
/**
 * A collection of arguments for invoking getResource.
 */
export interface GetResourceArgs {
    /**
     * Full path of the resource.  If no path is found, an error will be returned.
     */
    path: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
    /**
     * REST API id that owns the resource. If no REST API is found, an error will be returned.
     */
    restApiId: string;
}
/**
 * A collection of values returned by getResource.
 */
export interface GetResourceResult {
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    /**
     * Set to the ID of the parent Resource.
     */
    readonly parentId: string;
    readonly path: string;
    /**
     * Set to the path relative to the parent Resource.
     */
    readonly pathPart: string;
    readonly region: string;
    readonly restApiId: string;
}
/**
 * Use this data source to get the id of a Resource in API Gateway.
 * To fetch the Resource, you must provide the REST API id as well as the full path.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const myRestApi = aws.apigateway.getRestApi({
 *     name: "my-rest-api",
 * });
 * const myResource = myRestApi.then(myRestApi => aws.apigateway.getResource({
 *     restApiId: myRestApi.id,
 *     path: "/endpoint/path",
 * }));
 * ```
 */
export declare function getResourceOutput(args: GetResourceOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetResourceResult>;
/**
 * A collection of arguments for invoking getResource.
 */
export interface GetResourceOutputArgs {
    /**
     * Full path of the resource.  If no path is found, an error will be returned.
     */
    path: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * REST API id that owns the resource. If no REST API is found, an error will be returned.
     */
    restApiId: pulumi.Input<string>;
}
