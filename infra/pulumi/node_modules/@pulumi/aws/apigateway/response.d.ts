import * as pulumi from "@pulumi/pulumi";
/**
 * Provides an API Gateway Gateway Response for a REST API Gateway.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const main = new aws.apigateway.RestApi("main", {name: "MyDemoAPI"});
 * const test = new aws.apigateway.Response("test", {
 *     restApiId: main.id,
 *     statusCode: "401",
 *     responseType: "UNAUTHORIZED",
 *     responseTemplates: {
 *         "application/json": "{\"message\":$context.error.messageString}",
 *     },
 *     responseParameters: {
 *         "gatewayresponse.header.Authorization": "'Basic'",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_api_gateway_gateway_response` using `REST-API-ID/RESPONSE-TYPE`. For example:
 *
 * ```sh
 * $ pulumi import aws:apigateway/response:Response example 12345abcde/UNAUTHORIZED
 * ```
 */
export declare class Response extends pulumi.CustomResource {
    /**
     * Get an existing Response resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ResponseState, opts?: pulumi.CustomResourceOptions): Response;
    /**
     * Returns true if the given object is an instance of Response.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Response;
    /**
     * Region where this resource will be managed. See the [AWS Documentation](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints) for supported values. Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Map of parameters (paths, query strings and headers) of the Gateway Response.
     */
    readonly responseParameters: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * Map of templates used to transform the response body.
     */
    readonly responseTemplates: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * Response type of the associated GatewayResponse. See the [AWS Documentation](https://docs.aws.amazon.com/apigateway/latest/developerguide/supported-gateway-response-types.html) for supported values.
     */
    readonly responseType: pulumi.Output<string>;
    /**
     * String identifier of the associated REST API.
     */
    readonly restApiId: pulumi.Output<string>;
    /**
     * HTTP status code of the Gateway Response.
     */
    readonly statusCode: pulumi.Output<string | undefined>;
    /**
     * Create a Response resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ResponseArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Response resources.
 */
export interface ResponseState {
    /**
     * Region where this resource will be managed. See the [AWS Documentation](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints) for supported values. Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Map of parameters (paths, query strings and headers) of the Gateway Response.
     */
    responseParameters?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Map of templates used to transform the response body.
     */
    responseTemplates?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Response type of the associated GatewayResponse. See the [AWS Documentation](https://docs.aws.amazon.com/apigateway/latest/developerguide/supported-gateway-response-types.html) for supported values.
     */
    responseType?: pulumi.Input<string>;
    /**
     * String identifier of the associated REST API.
     */
    restApiId?: pulumi.Input<string>;
    /**
     * HTTP status code of the Gateway Response.
     */
    statusCode?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a Response resource.
 */
export interface ResponseArgs {
    /**
     * Region where this resource will be managed. See the [AWS Documentation](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints) for supported values. Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Map of parameters (paths, query strings and headers) of the Gateway Response.
     */
    responseParameters?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Map of templates used to transform the response body.
     */
    responseTemplates?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Response type of the associated GatewayResponse. See the [AWS Documentation](https://docs.aws.amazon.com/apigateway/latest/developerguide/supported-gateway-response-types.html) for supported values.
     */
    responseType: pulumi.Input<string>;
    /**
     * String identifier of the associated REST API.
     */
    restApiId: pulumi.Input<string>;
    /**
     * HTTP status code of the Gateway Response.
     */
    statusCode?: pulumi.Input<string>;
}
