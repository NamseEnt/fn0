import * as pulumi from "@pulumi/pulumi";
/**
 * Manages an Amazon API Gateway Version 2 [model](https://docs.aws.amazon.com/apigateway/latest/developerguide/models-mappings.html#models-mappings-models).
 *
 * ## Example Usage
 *
 * ### Basic
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.apigatewayv2.Model("example", {
 *     apiId: exampleAwsApigatewayv2Api.id,
 *     contentType: "application/json",
 *     name: "example",
 *     schema: JSON.stringify({
 *         $schema: "http://json-schema.org/draft-04/schema#",
 *         title: "ExampleModel",
 *         type: "object",
 *         properties: {
 *             id: {
 *                 type: "string",
 *             },
 *         },
 *     }),
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_apigatewayv2_model` using the API identifier and model identifier. For example:
 *
 * ```sh
 * $ pulumi import aws:apigatewayv2/model:Model example aabbccddee/1122334
 * ```
 */
export declare class Model extends pulumi.CustomResource {
    /**
     * Get an existing Model resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ModelState, opts?: pulumi.CustomResourceOptions): Model;
    /**
     * Returns true if the given object is an instance of Model.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Model;
    /**
     * API identifier.
     */
    readonly apiId: pulumi.Output<string>;
    /**
     * The content-type for the model, for example, `application/json`. Must be between 1 and 256 characters in length.
     */
    readonly contentType: pulumi.Output<string>;
    /**
     * Description of the model. Must be between 1 and 128 characters in length.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * Name of the model. Must be alphanumeric. Must be between 1 and 128 characters in length.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Schema for the model. This should be a [JSON schema draft 4](https://tools.ietf.org/html/draft-zyp-json-schema-04) model. Must be less than or equal to 32768 characters in length.
     */
    readonly schema: pulumi.Output<string>;
    /**
     * Create a Model resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ModelArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Model resources.
 */
export interface ModelState {
    /**
     * API identifier.
     */
    apiId?: pulumi.Input<string>;
    /**
     * The content-type for the model, for example, `application/json`. Must be between 1 and 256 characters in length.
     */
    contentType?: pulumi.Input<string>;
    /**
     * Description of the model. Must be between 1 and 128 characters in length.
     */
    description?: pulumi.Input<string>;
    /**
     * Name of the model. Must be alphanumeric. Must be between 1 and 128 characters in length.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Schema for the model. This should be a [JSON schema draft 4](https://tools.ietf.org/html/draft-zyp-json-schema-04) model. Must be less than or equal to 32768 characters in length.
     */
    schema?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a Model resource.
 */
export interface ModelArgs {
    /**
     * API identifier.
     */
    apiId: pulumi.Input<string>;
    /**
     * The content-type for the model, for example, `application/json`. Must be between 1 and 256 characters in length.
     */
    contentType: pulumi.Input<string>;
    /**
     * Description of the model. Must be between 1 and 128 characters in length.
     */
    description?: pulumi.Input<string>;
    /**
     * Name of the model. Must be alphanumeric. Must be between 1 and 128 characters in length.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Schema for the model. This should be a [JSON schema draft 4](https://tools.ietf.org/html/draft-zyp-json-schema-04) model. Must be less than or equal to 32768 characters in length.
     */
    schema: pulumi.Input<string>;
}
