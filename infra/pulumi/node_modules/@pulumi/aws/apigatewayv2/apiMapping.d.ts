import * as pulumi from "@pulumi/pulumi";
/**
 * Manages an Amazon API Gateway Version 2 API mapping.
 * More information can be found in the [Amazon API Gateway Developer Guide](https://docs.aws.amazon.com/apigateway/latest/developerguide/how-to-custom-domains.html).
 *
 * ## Example Usage
 *
 * ### Basic
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.apigatewayv2.ApiMapping("example", {
 *     apiId: exampleAwsApigatewayv2Api.id,
 *     domainName: exampleAwsApigatewayv2DomainName.id,
 *     stage: exampleAwsApigatewayv2Stage.id,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_apigatewayv2_api_mapping` using the API mapping identifier and domain name. For example:
 *
 * ```sh
 * $ pulumi import aws:apigatewayv2/apiMapping:ApiMapping example 1122334/ws-api.example.com
 * ```
 */
export declare class ApiMapping extends pulumi.CustomResource {
    /**
     * Get an existing ApiMapping resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ApiMappingState, opts?: pulumi.CustomResourceOptions): ApiMapping;
    /**
     * Returns true if the given object is an instance of ApiMapping.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ApiMapping;
    /**
     * API identifier.
     */
    readonly apiId: pulumi.Output<string>;
    /**
     * The API mapping key. Refer to [REST API](https://docs.aws.amazon.com/apigateway/latest/developerguide/rest-api-mappings.html), [HTTP API](https://docs.aws.amazon.com/apigateway/latest/developerguide/http-api-mappings.html) or [WebSocket API](https://docs.aws.amazon.com/apigateway/latest/developerguide/websocket-api-mappings.html).
     */
    readonly apiMappingKey: pulumi.Output<string | undefined>;
    /**
     * Domain name. Use the `aws.apigatewayv2.DomainName` resource to configure a domain name.
     */
    readonly domainName: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * API stage. Use the `aws.apigatewayv2.Stage` resource to configure an API stage.
     */
    readonly stage: pulumi.Output<string>;
    /**
     * Create a ApiMapping resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ApiMappingArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ApiMapping resources.
 */
export interface ApiMappingState {
    /**
     * API identifier.
     */
    apiId?: pulumi.Input<string>;
    /**
     * The API mapping key. Refer to [REST API](https://docs.aws.amazon.com/apigateway/latest/developerguide/rest-api-mappings.html), [HTTP API](https://docs.aws.amazon.com/apigateway/latest/developerguide/http-api-mappings.html) or [WebSocket API](https://docs.aws.amazon.com/apigateway/latest/developerguide/websocket-api-mappings.html).
     */
    apiMappingKey?: pulumi.Input<string>;
    /**
     * Domain name. Use the `aws.apigatewayv2.DomainName` resource to configure a domain name.
     */
    domainName?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * API stage. Use the `aws.apigatewayv2.Stage` resource to configure an API stage.
     */
    stage?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a ApiMapping resource.
 */
export interface ApiMappingArgs {
    /**
     * API identifier.
     */
    apiId: pulumi.Input<string>;
    /**
     * The API mapping key. Refer to [REST API](https://docs.aws.amazon.com/apigateway/latest/developerguide/rest-api-mappings.html), [HTTP API](https://docs.aws.amazon.com/apigateway/latest/developerguide/http-api-mappings.html) or [WebSocket API](https://docs.aws.amazon.com/apigateway/latest/developerguide/websocket-api-mappings.html).
     */
    apiMappingKey?: pulumi.Input<string>;
    /**
     * Domain name. Use the `aws.apigatewayv2.DomainName` resource to configure a domain name.
     */
    domainName: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * API stage. Use the `aws.apigatewayv2.Stage` resource to configure an API stage.
     */
    stage: pulumi.Input<string>;
}
