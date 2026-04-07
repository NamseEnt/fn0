import * as pulumi from "@pulumi/pulumi";
import { RestApi } from "./index";
/**
 * Manages an API Gateway Request Validator.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.apigateway.RequestValidator("example", {
 *     name: "example",
 *     restApi: exampleAwsApiGatewayRestApi.id,
 *     validateRequestBody: true,
 *     validateRequestParameters: true,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_api_gateway_request_validator` using `REST-API-ID/REQUEST-VALIDATOR-ID`. For example:
 *
 * ```sh
 * $ pulumi import aws:apigateway/requestValidator:RequestValidator example 12345abcde/67890fghij
 * ```
 */
export declare class RequestValidator extends pulumi.CustomResource {
    /**
     * Get an existing RequestValidator resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: RequestValidatorState, opts?: pulumi.CustomResourceOptions): RequestValidator;
    /**
     * Returns true if the given object is an instance of RequestValidator.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is RequestValidator;
    /**
     * Name of the request validator
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * ID of the associated Rest API
     */
    readonly restApi: pulumi.Output<string>;
    /**
     * Boolean whether to validate request body. Defaults to `false`.
     */
    readonly validateRequestBody: pulumi.Output<boolean | undefined>;
    /**
     * Boolean whether to validate request parameters. Defaults to `false`.
     */
    readonly validateRequestParameters: pulumi.Output<boolean | undefined>;
    /**
     * Create a RequestValidator resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: RequestValidatorArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering RequestValidator resources.
 */
export interface RequestValidatorState {
    /**
     * Name of the request validator
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ID of the associated Rest API
     */
    restApi?: pulumi.Input<string | RestApi>;
    /**
     * Boolean whether to validate request body. Defaults to `false`.
     */
    validateRequestBody?: pulumi.Input<boolean>;
    /**
     * Boolean whether to validate request parameters. Defaults to `false`.
     */
    validateRequestParameters?: pulumi.Input<boolean>;
}
/**
 * The set of arguments for constructing a RequestValidator resource.
 */
export interface RequestValidatorArgs {
    /**
     * Name of the request validator
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ID of the associated Rest API
     */
    restApi: pulumi.Input<string | RestApi>;
    /**
     * Boolean whether to validate request body. Defaults to `false`.
     */
    validateRequestBody?: pulumi.Input<boolean>;
    /**
     * Boolean whether to validate request parameters. Defaults to `false`.
     */
    validateRequestParameters?: pulumi.Input<boolean>;
}
