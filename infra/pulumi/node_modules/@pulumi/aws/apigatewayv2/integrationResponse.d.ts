import * as pulumi from "@pulumi/pulumi";
/**
 * Manages an Amazon API Gateway Version 2 integration response.
 * More information can be found in the [Amazon API Gateway Developer Guide](https://docs.aws.amazon.com/apigateway/latest/developerguide/apigateway-websocket-api.html).
 *
 * ## Example Usage
 *
 * ### Basic
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.apigatewayv2.IntegrationResponse("example", {
 *     apiId: exampleAwsApigatewayv2Api.id,
 *     integrationId: exampleAwsApigatewayv2Integration.id,
 *     integrationResponseKey: "/200/",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_apigatewayv2_integration_response` using the API identifier, integration identifier and integration response identifier. For example:
 *
 * ```sh
 * $ pulumi import aws:apigatewayv2/integrationResponse:IntegrationResponse example aabbccddee/1122334/998877
 * ```
 */
export declare class IntegrationResponse extends pulumi.CustomResource {
    /**
     * Get an existing IntegrationResponse resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: IntegrationResponseState, opts?: pulumi.CustomResourceOptions): IntegrationResponse;
    /**
     * Returns true if the given object is an instance of IntegrationResponse.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is IntegrationResponse;
    /**
     * API identifier.
     */
    readonly apiId: pulumi.Output<string>;
    /**
     * How to handle response payload content type conversions. Valid values: `CONVERT_TO_BINARY`, `CONVERT_TO_TEXT`.
     */
    readonly contentHandlingStrategy: pulumi.Output<string | undefined>;
    /**
     * Identifier of the `aws.apigatewayv2.Integration`.
     */
    readonly integrationId: pulumi.Output<string>;
    /**
     * Integration response key.
     */
    readonly integrationResponseKey: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Map of Velocity templates that are applied on the request payload based on the value of the Content-Type header sent by the client.
     */
    readonly responseTemplates: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * The [template selection expression](https://docs.aws.amazon.com/apigateway/latest/developerguide/apigateway-websocket-api-selection-expressions.html#apigateway-websocket-api-template-selection-expressions) for the integration response.
     */
    readonly templateSelectionExpression: pulumi.Output<string | undefined>;
    /**
     * Create a IntegrationResponse resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: IntegrationResponseArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering IntegrationResponse resources.
 */
export interface IntegrationResponseState {
    /**
     * API identifier.
     */
    apiId?: pulumi.Input<string>;
    /**
     * How to handle response payload content type conversions. Valid values: `CONVERT_TO_BINARY`, `CONVERT_TO_TEXT`.
     */
    contentHandlingStrategy?: pulumi.Input<string>;
    /**
     * Identifier of the `aws.apigatewayv2.Integration`.
     */
    integrationId?: pulumi.Input<string>;
    /**
     * Integration response key.
     */
    integrationResponseKey?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Map of Velocity templates that are applied on the request payload based on the value of the Content-Type header sent by the client.
     */
    responseTemplates?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The [template selection expression](https://docs.aws.amazon.com/apigateway/latest/developerguide/apigateway-websocket-api-selection-expressions.html#apigateway-websocket-api-template-selection-expressions) for the integration response.
     */
    templateSelectionExpression?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a IntegrationResponse resource.
 */
export interface IntegrationResponseArgs {
    /**
     * API identifier.
     */
    apiId: pulumi.Input<string>;
    /**
     * How to handle response payload content type conversions. Valid values: `CONVERT_TO_BINARY`, `CONVERT_TO_TEXT`.
     */
    contentHandlingStrategy?: pulumi.Input<string>;
    /**
     * Identifier of the `aws.apigatewayv2.Integration`.
     */
    integrationId: pulumi.Input<string>;
    /**
     * Integration response key.
     */
    integrationResponseKey: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Map of Velocity templates that are applied on the request payload based on the value of the Content-Type header sent by the client.
     */
    responseTemplates?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The [template selection expression](https://docs.aws.amazon.com/apigateway/latest/developerguide/apigateway-websocket-api-selection-expressions.html#apigateway-websocket-api-template-selection-expressions) for the integration response.
     */
    templateSelectionExpression?: pulumi.Input<string>;
}
