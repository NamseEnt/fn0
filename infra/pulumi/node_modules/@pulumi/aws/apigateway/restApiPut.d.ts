import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.apigateway.RestApiPut("example", {
 *     body: JSON.stringify({
 *         swagger: "2.0",
 *         info: {
 *             title: "Example API",
 *             version: "v1",
 *         },
 *         schemes: ["https"],
 *         paths: {
 *             "/example": {
 *                 get: {
 *                     responses: {
 *                         "200": {
 *                             description: "OK",
 *                         },
 *                     },
 *                     "x-amazon-apigateway-integration": {
 *                         httpMethod: "GET",
 *                         type: "HTTP",
 *                         responses: {
 *                             "default": {
 *                                 statusCode: 200,
 *                             },
 *                         },
 *                         uri: "https://api.example.com/",
 *                     },
 *                 },
 *             },
 *         },
 *     }),
 *     failOnWarnings: true,
 *     restApiId: exampleAwsApiGatewayRestApi.id,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import API Gateway REST API Put using the `rest_api_id`. For example:
 *
 * ```sh
 * $ pulumi import aws:apigateway/restApiPut:RestApiPut example import-id-12345678
 * ```
 */
export declare class RestApiPut extends pulumi.CustomResource {
    /**
     * Get an existing RestApiPut resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: RestApiPutState, opts?: pulumi.CustomResourceOptions): RestApiPut;
    /**
     * Returns true if the given object is an instance of RestApiPut.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is RestApiPut;
    /**
     * PUT request body containing external API definitions. Currently, only OpenAPI definition JSON/YAML files are supported. The maximum size of the API definition file is 6MB.
     */
    readonly body: pulumi.Output<string>;
    /**
     * Whether to rollback the API update when a warning is encountered. The default value is `false`.
     */
    readonly failOnWarnings: pulumi.Output<boolean>;
    /**
     * Map of customizations for importing the specification in the `body` argument. For example, to exclude DocumentationParts from an imported API, use `ignore = "documentation"`. Additional documentation, including other parameters such as `basepath`, can be found in the [API Gateway Developer Guide](https://docs.aws.amazon.com/apigateway/latest/developerguide/api-gateway-import-api.html).
     */
    readonly parameters: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Identifier of the associated REST API.
     *
     * The following arguments are optional:
     */
    readonly restApiId: pulumi.Output<string>;
    readonly timeouts: pulumi.Output<outputs.apigateway.RestApiPutTimeouts | undefined>;
    /**
     * Map of arbitrary keys and values that, when changed, will trigger a redeployment. To force a redeployment without changing these keys/values, use the `-replace` option with `pulumi preview` or `pulumi up`.
     */
    readonly triggers: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * Create a RestApiPut resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: RestApiPutArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering RestApiPut resources.
 */
export interface RestApiPutState {
    /**
     * PUT request body containing external API definitions. Currently, only OpenAPI definition JSON/YAML files are supported. The maximum size of the API definition file is 6MB.
     */
    body?: pulumi.Input<string>;
    /**
     * Whether to rollback the API update when a warning is encountered. The default value is `false`.
     */
    failOnWarnings?: pulumi.Input<boolean>;
    /**
     * Map of customizations for importing the specification in the `body` argument. For example, to exclude DocumentationParts from an imported API, use `ignore = "documentation"`. Additional documentation, including other parameters such as `basepath`, can be found in the [API Gateway Developer Guide](https://docs.aws.amazon.com/apigateway/latest/developerguide/api-gateway-import-api.html).
     */
    parameters?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Identifier of the associated REST API.
     *
     * The following arguments are optional:
     */
    restApiId?: pulumi.Input<string>;
    timeouts?: pulumi.Input<inputs.apigateway.RestApiPutTimeouts>;
    /**
     * Map of arbitrary keys and values that, when changed, will trigger a redeployment. To force a redeployment without changing these keys/values, use the `-replace` option with `pulumi preview` or `pulumi up`.
     */
    triggers?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
/**
 * The set of arguments for constructing a RestApiPut resource.
 */
export interface RestApiPutArgs {
    /**
     * PUT request body containing external API definitions. Currently, only OpenAPI definition JSON/YAML files are supported. The maximum size of the API definition file is 6MB.
     */
    body: pulumi.Input<string>;
    /**
     * Whether to rollback the API update when a warning is encountered. The default value is `false`.
     */
    failOnWarnings?: pulumi.Input<boolean>;
    /**
     * Map of customizations for importing the specification in the `body` argument. For example, to exclude DocumentationParts from an imported API, use `ignore = "documentation"`. Additional documentation, including other parameters such as `basepath`, can be found in the [API Gateway Developer Guide](https://docs.aws.amazon.com/apigateway/latest/developerguide/api-gateway-import-api.html).
     */
    parameters?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Identifier of the associated REST API.
     *
     * The following arguments are optional:
     */
    restApiId: pulumi.Input<string>;
    timeouts?: pulumi.Input<inputs.apigateway.RestApiPutTimeouts>;
    /**
     * Map of arbitrary keys and values that, when changed, will trigger a redeployment. To force a redeployment without changing these keys/values, use the `-replace` option with `pulumi preview` or `pulumi up`.
     */
    triggers?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
