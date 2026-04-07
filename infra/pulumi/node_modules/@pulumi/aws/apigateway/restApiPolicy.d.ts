import * as pulumi from "@pulumi/pulumi";
/**
 * Provides an API Gateway REST API Policy.
 *
 * > **Note:** Amazon API Gateway Version 1 resources are used for creating and deploying REST APIs. To create and deploy WebSocket and HTTP APIs, use Amazon API Gateway Version 2 resources.
 *
 * ## Example Usage
 *
 * ### Basic
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const testRestApi = new aws.apigateway.RestApi("test", {name: "example-rest-api"});
 * const test = aws.iam.getPolicyDocumentOutput({
 *     statements: [{
 *         effect: "Allow",
 *         principals: [{
 *             type: "AWS",
 *             identifiers: ["*"],
 *         }],
 *         actions: ["execute-api:Invoke"],
 *         resources: [pulumi.interpolate`${testRestApi.executionArn}/*`],
 *         conditions: [{
 *             test: "IpAddress",
 *             variable: "aws:SourceIp",
 *             values: ["123.123.123.123/32"],
 *         }],
 *     }],
 * });
 * const testRestApiPolicy = new aws.apigateway.RestApiPolicy("test", {
 *     restApiId: testRestApi.id,
 *     policy: test.apply(test => test.json),
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_api_gateway_rest_api_policy` using the REST API ID. For example:
 *
 * ```sh
 * $ pulumi import aws:apigateway/restApiPolicy:RestApiPolicy example 12345abcde
 * ```
 */
export declare class RestApiPolicy extends pulumi.CustomResource {
    /**
     * Get an existing RestApiPolicy resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: RestApiPolicyState, opts?: pulumi.CustomResourceOptions): RestApiPolicy;
    /**
     * Returns true if the given object is an instance of RestApiPolicy.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is RestApiPolicy;
    /**
     * JSON formatted policy document that controls access to the API Gateway.
     */
    readonly policy: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * ID of the REST API.
     */
    readonly restApiId: pulumi.Output<string>;
    /**
     * Create a RestApiPolicy resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: RestApiPolicyArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering RestApiPolicy resources.
 */
export interface RestApiPolicyState {
    /**
     * JSON formatted policy document that controls access to the API Gateway.
     */
    policy?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ID of the REST API.
     */
    restApiId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a RestApiPolicy resource.
 */
export interface RestApiPolicyArgs {
    /**
     * JSON formatted policy document that controls access to the API Gateway.
     */
    policy: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ID of the REST API.
     */
    restApiId: pulumi.Input<string>;
}
