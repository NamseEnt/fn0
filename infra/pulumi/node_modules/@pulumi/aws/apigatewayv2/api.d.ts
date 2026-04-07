import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages an Amazon API Gateway Version 2 API.
 *
 * > **Note:** Amazon API Gateway Version 2 resources are used for creating and deploying WebSocket and HTTP APIs. To create and deploy REST APIs, use Amazon API Gateway Version 1 resources.
 *
 * ## Example Usage
 *
 * ### Basic WebSocket API
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.apigatewayv2.Api("example", {
 *     name: "example-websocket-api",
 *     protocolType: "WEBSOCKET",
 *     routeSelectionExpression: "$request.body.action",
 * });
 * ```
 *
 * ### Basic HTTP API
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.apigatewayv2.Api("example", {
 *     name: "example-http-api",
 *     protocolType: "HTTP",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_apigatewayv2_api` using the API identifier. For example:
 *
 * ```sh
 * $ pulumi import aws:apigatewayv2/api:Api example aabbccddee
 * ```
 */
export declare class Api extends pulumi.CustomResource {
    /**
     * Get an existing Api resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ApiState, opts?: pulumi.CustomResourceOptions): Api;
    /**
     * Returns true if the given object is an instance of Api.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Api;
    /**
     * URI of the API, of the form `https://{api-id}.execute-api.{region}.amazonaws.com` for HTTP APIs and `wss://{api-id}.execute-api.{region}.amazonaws.com` for WebSocket APIs.
     */
    readonly apiEndpoint: pulumi.Output<string>;
    /**
     * An [API key selection expression](https://docs.aws.amazon.com/apigateway/latest/developerguide/apigateway-websocket-api-selection-expressions.html#apigateway-websocket-api-apikey-selection-expressions).
     * Valid values: `$context.authorizer.usageIdentifierKey`, `$request.header.x-api-key`. Defaults to `$request.header.x-api-key`.
     * Applicable for WebSocket APIs.
     */
    readonly apiKeySelectionExpression: pulumi.Output<string | undefined>;
    /**
     * ARN of the API.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * An OpenAPI specification that defines the set of routes and integrations to create as part of the HTTP APIs. Supported only for HTTP APIs.
     */
    readonly body: pulumi.Output<string | undefined>;
    /**
     * Cross-origin resource sharing (CORS) [configuration](https://docs.aws.amazon.com/apigateway/latest/developerguide/http-api-cors.html). Applicable for HTTP APIs.
     */
    readonly corsConfiguration: pulumi.Output<outputs.apigatewayv2.ApiCorsConfiguration | undefined>;
    /**
     * Part of _quick create_. Specifies any credentials required for the integration. Applicable for HTTP APIs.
     */
    readonly credentialsArn: pulumi.Output<string | undefined>;
    /**
     * Description of the API. Must be less than or equal to 1024 characters in length.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * Whether clients can invoke the API by using the default `execute-api` endpoint.
     * By default, clients can invoke the API with the default `{api_id}.execute-api.{region}.amazonaws.com endpoint`.
     * To require that clients use a custom domain name to invoke the API, disable the default endpoint.
     */
    readonly disableExecuteApiEndpoint: pulumi.Output<boolean | undefined>;
    /**
     * ARN prefix to be used in an `aws.lambda.Permission`'s `sourceArn` attribute
     * or in an `aws.iam.Policy` to authorize access to the [`@connections` API](https://docs.aws.amazon.com/apigateway/latest/developerguide/apigateway-how-to-call-websocket-api-connections.html).
     * See the [Amazon API Gateway Developer Guide](https://docs.aws.amazon.com/apigateway/latest/developerguide/apigateway-websocket-control-access-iam.html) for details.
     */
    readonly executionArn: pulumi.Output<string>;
    /**
     * Whether warnings should return an error while API Gateway is creating or updating the resource using an OpenAPI specification. Defaults to `false`. Applicable for HTTP APIs.
     */
    readonly failOnWarnings: pulumi.Output<boolean | undefined>;
    /**
     * The IP address types that can invoke the API. Valid values: `ipv4`, `dualstack`. Use `ipv4` to allow only IPv4 addresses to invoke your API, or use `dualstack` to allow both IPv4 and IPv6 addresses to invoke your API. Defaults to `ipv4`.
     */
    readonly ipAddressType: pulumi.Output<string>;
    /**
     * Name of the API. Must be less than or equal to 128 characters in length.
     */
    readonly name: pulumi.Output<string>;
    /**
     * API protocol. Valid values: `HTTP`, `WEBSOCKET`.
     */
    readonly protocolType: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Part of _quick create_. Specifies any [route key](https://docs.aws.amazon.com/apigateway/latest/developerguide/http-api-develop-routes.html). Applicable for HTTP APIs.
     */
    readonly routeKey: pulumi.Output<string | undefined>;
    /**
     * The [route selection expression](https://docs.aws.amazon.com/apigateway/latest/developerguide/apigateway-websocket-api-selection-expressions.html#apigateway-websocket-api-route-selection-expressions) for the API.
     * Defaults to `$request.method $request.path`.
     */
    readonly routeSelectionExpression: pulumi.Output<string | undefined>;
    /**
     * Map of tags to assign to the API. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Part of _quick create_. Quick create produces an API with an integration, a default catch-all route, and a default stage which is configured to automatically deploy changes.
     * For HTTP integrations, specify a fully qualified URL. For Lambda integrations, specify a function ARN.
     * The type of the integration will be `HTTP_PROXY` or `AWS_PROXY`, respectively. Applicable for HTTP APIs.
     */
    readonly target: pulumi.Output<string | undefined>;
    /**
     * Version identifier for the API. Must be between 1 and 64 characters in length.
     */
    readonly version: pulumi.Output<string | undefined>;
    /**
     * Create a Api resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ApiArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Api resources.
 */
export interface ApiState {
    /**
     * URI of the API, of the form `https://{api-id}.execute-api.{region}.amazonaws.com` for HTTP APIs and `wss://{api-id}.execute-api.{region}.amazonaws.com` for WebSocket APIs.
     */
    apiEndpoint?: pulumi.Input<string>;
    /**
     * An [API key selection expression](https://docs.aws.amazon.com/apigateway/latest/developerguide/apigateway-websocket-api-selection-expressions.html#apigateway-websocket-api-apikey-selection-expressions).
     * Valid values: `$context.authorizer.usageIdentifierKey`, `$request.header.x-api-key`. Defaults to `$request.header.x-api-key`.
     * Applicable for WebSocket APIs.
     */
    apiKeySelectionExpression?: pulumi.Input<string>;
    /**
     * ARN of the API.
     */
    arn?: pulumi.Input<string>;
    /**
     * An OpenAPI specification that defines the set of routes and integrations to create as part of the HTTP APIs. Supported only for HTTP APIs.
     */
    body?: pulumi.Input<string>;
    /**
     * Cross-origin resource sharing (CORS) [configuration](https://docs.aws.amazon.com/apigateway/latest/developerguide/http-api-cors.html). Applicable for HTTP APIs.
     */
    corsConfiguration?: pulumi.Input<inputs.apigatewayv2.ApiCorsConfiguration>;
    /**
     * Part of _quick create_. Specifies any credentials required for the integration. Applicable for HTTP APIs.
     */
    credentialsArn?: pulumi.Input<string>;
    /**
     * Description of the API. Must be less than or equal to 1024 characters in length.
     */
    description?: pulumi.Input<string>;
    /**
     * Whether clients can invoke the API by using the default `execute-api` endpoint.
     * By default, clients can invoke the API with the default `{api_id}.execute-api.{region}.amazonaws.com endpoint`.
     * To require that clients use a custom domain name to invoke the API, disable the default endpoint.
     */
    disableExecuteApiEndpoint?: pulumi.Input<boolean>;
    /**
     * ARN prefix to be used in an `aws.lambda.Permission`'s `sourceArn` attribute
     * or in an `aws.iam.Policy` to authorize access to the [`@connections` API](https://docs.aws.amazon.com/apigateway/latest/developerguide/apigateway-how-to-call-websocket-api-connections.html).
     * See the [Amazon API Gateway Developer Guide](https://docs.aws.amazon.com/apigateway/latest/developerguide/apigateway-websocket-control-access-iam.html) for details.
     */
    executionArn?: pulumi.Input<string>;
    /**
     * Whether warnings should return an error while API Gateway is creating or updating the resource using an OpenAPI specification. Defaults to `false`. Applicable for HTTP APIs.
     */
    failOnWarnings?: pulumi.Input<boolean>;
    /**
     * The IP address types that can invoke the API. Valid values: `ipv4`, `dualstack`. Use `ipv4` to allow only IPv4 addresses to invoke your API, or use `dualstack` to allow both IPv4 and IPv6 addresses to invoke your API. Defaults to `ipv4`.
     */
    ipAddressType?: pulumi.Input<string>;
    /**
     * Name of the API. Must be less than or equal to 128 characters in length.
     */
    name?: pulumi.Input<string>;
    /**
     * API protocol. Valid values: `HTTP`, `WEBSOCKET`.
     */
    protocolType?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Part of _quick create_. Specifies any [route key](https://docs.aws.amazon.com/apigateway/latest/developerguide/http-api-develop-routes.html). Applicable for HTTP APIs.
     */
    routeKey?: pulumi.Input<string>;
    /**
     * The [route selection expression](https://docs.aws.amazon.com/apigateway/latest/developerguide/apigateway-websocket-api-selection-expressions.html#apigateway-websocket-api-route-selection-expressions) for the API.
     * Defaults to `$request.method $request.path`.
     */
    routeSelectionExpression?: pulumi.Input<string>;
    /**
     * Map of tags to assign to the API. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Part of _quick create_. Quick create produces an API with an integration, a default catch-all route, and a default stage which is configured to automatically deploy changes.
     * For HTTP integrations, specify a fully qualified URL. For Lambda integrations, specify a function ARN.
     * The type of the integration will be `HTTP_PROXY` or `AWS_PROXY`, respectively. Applicable for HTTP APIs.
     */
    target?: pulumi.Input<string>;
    /**
     * Version identifier for the API. Must be between 1 and 64 characters in length.
     */
    version?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a Api resource.
 */
export interface ApiArgs {
    /**
     * An [API key selection expression](https://docs.aws.amazon.com/apigateway/latest/developerguide/apigateway-websocket-api-selection-expressions.html#apigateway-websocket-api-apikey-selection-expressions).
     * Valid values: `$context.authorizer.usageIdentifierKey`, `$request.header.x-api-key`. Defaults to `$request.header.x-api-key`.
     * Applicable for WebSocket APIs.
     */
    apiKeySelectionExpression?: pulumi.Input<string>;
    /**
     * An OpenAPI specification that defines the set of routes and integrations to create as part of the HTTP APIs. Supported only for HTTP APIs.
     */
    body?: pulumi.Input<string>;
    /**
     * Cross-origin resource sharing (CORS) [configuration](https://docs.aws.amazon.com/apigateway/latest/developerguide/http-api-cors.html). Applicable for HTTP APIs.
     */
    corsConfiguration?: pulumi.Input<inputs.apigatewayv2.ApiCorsConfiguration>;
    /**
     * Part of _quick create_. Specifies any credentials required for the integration. Applicable for HTTP APIs.
     */
    credentialsArn?: pulumi.Input<string>;
    /**
     * Description of the API. Must be less than or equal to 1024 characters in length.
     */
    description?: pulumi.Input<string>;
    /**
     * Whether clients can invoke the API by using the default `execute-api` endpoint.
     * By default, clients can invoke the API with the default `{api_id}.execute-api.{region}.amazonaws.com endpoint`.
     * To require that clients use a custom domain name to invoke the API, disable the default endpoint.
     */
    disableExecuteApiEndpoint?: pulumi.Input<boolean>;
    /**
     * Whether warnings should return an error while API Gateway is creating or updating the resource using an OpenAPI specification. Defaults to `false`. Applicable for HTTP APIs.
     */
    failOnWarnings?: pulumi.Input<boolean>;
    /**
     * The IP address types that can invoke the API. Valid values: `ipv4`, `dualstack`. Use `ipv4` to allow only IPv4 addresses to invoke your API, or use `dualstack` to allow both IPv4 and IPv6 addresses to invoke your API. Defaults to `ipv4`.
     */
    ipAddressType?: pulumi.Input<string>;
    /**
     * Name of the API. Must be less than or equal to 128 characters in length.
     */
    name?: pulumi.Input<string>;
    /**
     * API protocol. Valid values: `HTTP`, `WEBSOCKET`.
     */
    protocolType: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Part of _quick create_. Specifies any [route key](https://docs.aws.amazon.com/apigateway/latest/developerguide/http-api-develop-routes.html). Applicable for HTTP APIs.
     */
    routeKey?: pulumi.Input<string>;
    /**
     * The [route selection expression](https://docs.aws.amazon.com/apigateway/latest/developerguide/apigateway-websocket-api-selection-expressions.html#apigateway-websocket-api-route-selection-expressions) for the API.
     * Defaults to `$request.method $request.path`.
     */
    routeSelectionExpression?: pulumi.Input<string>;
    /**
     * Map of tags to assign to the API. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Part of _quick create_. Quick create produces an API with an integration, a default catch-all route, and a default stage which is configured to automatically deploy changes.
     * For HTTP integrations, specify a fully qualified URL. For Lambda integrations, specify a function ARN.
     * The type of the integration will be `HTTP_PROXY` or `AWS_PROXY`, respectively. Applicable for HTTP APIs.
     */
    target?: pulumi.Input<string>;
    /**
     * Version identifier for the API. Must be between 1 and 64 characters in length.
     */
    version?: pulumi.Input<string>;
}
