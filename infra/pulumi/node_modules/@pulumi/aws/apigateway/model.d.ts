import * as pulumi from "@pulumi/pulumi";
import { RestApi } from "./index";
/**
 * Provides a Model for a REST API Gateway.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const myDemoAPI = new aws.apigateway.RestApi("MyDemoAPI", {
 *     name: "MyDemoAPI",
 *     description: "This is my API for demonstration purposes",
 * });
 * const myDemoModel = new aws.apigateway.Model("MyDemoModel", {
 *     restApi: myDemoAPI.id,
 *     name: "user",
 *     description: "a JSON schema",
 *     contentType: "application/json",
 *     schema: JSON.stringify({
 *         type: "object",
 *     }),
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_api_gateway_model` using `REST-API-ID/NAME`. For example:
 *
 * ```sh
 * $ pulumi import aws:apigateway/model:Model example 12345abcde/example
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
     * Content type of the model
     */
    readonly contentType: pulumi.Output<string>;
    /**
     * Description of the model
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * Name of the model
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * ID of the associated REST API
     */
    readonly restApi: pulumi.Output<string>;
    /**
     * Schema of the model in a JSON form
     */
    readonly schema: pulumi.Output<string | undefined>;
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
     * Content type of the model
     */
    contentType?: pulumi.Input<string>;
    /**
     * Description of the model
     */
    description?: pulumi.Input<string>;
    /**
     * Name of the model
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ID of the associated REST API
     */
    restApi?: pulumi.Input<string | RestApi>;
    /**
     * Schema of the model in a JSON form
     */
    schema?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a Model resource.
 */
export interface ModelArgs {
    /**
     * Content type of the model
     */
    contentType: pulumi.Input<string>;
    /**
     * Description of the model
     */
    description?: pulumi.Input<string>;
    /**
     * Name of the model
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ID of the associated REST API
     */
    restApi: pulumi.Input<string | RestApi>;
    /**
     * Schema of the model in a JSON form
     */
    schema?: pulumi.Input<string>;
}
