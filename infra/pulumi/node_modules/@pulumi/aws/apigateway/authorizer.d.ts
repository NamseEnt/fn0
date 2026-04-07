import * as pulumi from "@pulumi/pulumi";
import { RestApi } from "./index";
/**
 * Provides an API Gateway Authorizer.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 * import * as std from "@pulumi/std";
 *
 * const demoRestApi = new aws.apigateway.RestApi("demo", {name: "auth-demo"});
 * const invocationAssumeRole = aws.iam.getPolicyDocument({
 *     statements: [{
 *         effect: "Allow",
 *         principals: [{
 *             type: "Service",
 *             identifiers: ["apigateway.amazonaws.com"],
 *         }],
 *         actions: ["sts:AssumeRole"],
 *     }],
 * });
 * const invocationRole = new aws.iam.Role("invocation_role", {
 *     name: "api_gateway_auth_invocation",
 *     path: "/",
 *     assumeRolePolicy: invocationAssumeRole.then(invocationAssumeRole => invocationAssumeRole.json),
 * });
 * const lambdaAssumeRole = aws.iam.getPolicyDocument({
 *     statements: [{
 *         effect: "Allow",
 *         actions: ["sts:AssumeRole"],
 *         principals: [{
 *             type: "Service",
 *             identifiers: ["lambda.amazonaws.com"],
 *         }],
 *     }],
 * });
 * const lambda = new aws.iam.Role("lambda", {
 *     name: "demo-lambda",
 *     assumeRolePolicy: lambdaAssumeRole.then(lambdaAssumeRole => lambdaAssumeRole.json),
 * });
 * const authorizer = new aws.lambda.Function("authorizer", {
 *     code: new pulumi.asset.FileArchive("lambda-function.zip"),
 *     name: "api_gateway_authorizer",
 *     role: lambda.arn,
 *     handler: "exports.example",
 *     sourceCodeHash: std.filebase64sha256({
 *         input: "lambda-function.zip",
 *     }).then(invoke => invoke.result),
 * });
 * const demo = new aws.apigateway.Authorizer("demo", {
 *     name: "demo",
 *     restApi: demoRestApi.id,
 *     authorizerUri: authorizer.invokeArn,
 *     authorizerCredentials: invocationRole.arn,
 * });
 * const invocationPolicy = aws.iam.getPolicyDocumentOutput({
 *     statements: [{
 *         effect: "Allow",
 *         actions: ["lambda:InvokeFunction"],
 *         resources: [authorizer.arn],
 *     }],
 * });
 * const invocationPolicyRolePolicy = new aws.iam.RolePolicy("invocation_policy", {
 *     name: "default",
 *     role: invocationRole.id,
 *     policy: invocationPolicy.apply(invocationPolicy => invocationPolicy.json),
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import AWS API Gateway Authorizer using the `REST-API-ID/AUTHORIZER-ID`. For example:
 *
 * ```sh
 * $ pulumi import aws:apigateway/authorizer:Authorizer authorizer 12345abcde/example
 * ```
 */
export declare class Authorizer extends pulumi.CustomResource {
    /**
     * Get an existing Authorizer resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: AuthorizerState, opts?: pulumi.CustomResourceOptions): Authorizer;
    /**
     * Returns true if the given object is an instance of Authorizer.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Authorizer;
    /**
     * ARN of the API Gateway Authorizer
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Credentials required for the authorizer. To specify an IAM Role for API Gateway to assume, use the IAM Role ARN.
     */
    readonly authorizerCredentials: pulumi.Output<string | undefined>;
    /**
     * TTL of cached authorizer results in seconds. Defaults to `300`.
     */
    readonly authorizerResultTtlInSeconds: pulumi.Output<number | undefined>;
    /**
     * Authorizer's Uniform Resource Identifier (URI). This must be a well-formed Lambda function URI in the form of `arn:aws:apigateway:{region}:lambda:path/{service_api}`,
     * e.g., `arn:aws:apigateway:us-west-2:lambda:path/2015-03-31/functions/arn:aws:lambda:us-west-2:012345678912:function:my-function/invocations`
     */
    readonly authorizerUri: pulumi.Output<string | undefined>;
    /**
     * Source of the identity in an incoming request. Defaults to `method.request.header.Authorization`. For `REQUEST` type, this may be a comma-separated list of values, including headers, query string parameters and stage variables - e.g., `"method.request.header.SomeHeaderName,method.request.querystring.SomeQueryStringName,stageVariables.SomeStageVariableName"`
     */
    readonly identitySource: pulumi.Output<string | undefined>;
    /**
     * Validation expression for the incoming identity. For `TOKEN` type, this value should be a regular expression. The incoming token from the client is matched against this expression, and will proceed if the token matches. If the token doesn't match, the client receives a 401 Unauthorized response.
     */
    readonly identityValidationExpression: pulumi.Output<string | undefined>;
    /**
     * Name of the authorizer
     */
    readonly name: pulumi.Output<string>;
    /**
     * List of the Amazon Cognito user pool ARNs. Each element is of this format: `arn:aws:cognito-idp:{region}:{account_id}:userpool/{user_pool_id}`.
     */
    readonly providerArns: pulumi.Output<string[] | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * ID of the associated REST API
     */
    readonly restApi: pulumi.Output<string>;
    /**
     * Type of the authorizer. Possible values are `TOKEN` for a Lambda function using a single authorization token submitted in a custom header, `REQUEST` for a Lambda function using incoming request parameters, or `COGNITO_USER_POOLS` for using an Amazon Cognito user pool. Defaults to `TOKEN`.
     */
    readonly type: pulumi.Output<string | undefined>;
    /**
     * Create a Authorizer resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: AuthorizerArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Authorizer resources.
 */
export interface AuthorizerState {
    /**
     * ARN of the API Gateway Authorizer
     */
    arn?: pulumi.Input<string>;
    /**
     * Credentials required for the authorizer. To specify an IAM Role for API Gateway to assume, use the IAM Role ARN.
     */
    authorizerCredentials?: pulumi.Input<string>;
    /**
     * TTL of cached authorizer results in seconds. Defaults to `300`.
     */
    authorizerResultTtlInSeconds?: pulumi.Input<number>;
    /**
     * Authorizer's Uniform Resource Identifier (URI). This must be a well-formed Lambda function URI in the form of `arn:aws:apigateway:{region}:lambda:path/{service_api}`,
     * e.g., `arn:aws:apigateway:us-west-2:lambda:path/2015-03-31/functions/arn:aws:lambda:us-west-2:012345678912:function:my-function/invocations`
     */
    authorizerUri?: pulumi.Input<string>;
    /**
     * Source of the identity in an incoming request. Defaults to `method.request.header.Authorization`. For `REQUEST` type, this may be a comma-separated list of values, including headers, query string parameters and stage variables - e.g., `"method.request.header.SomeHeaderName,method.request.querystring.SomeQueryStringName,stageVariables.SomeStageVariableName"`
     */
    identitySource?: pulumi.Input<string>;
    /**
     * Validation expression for the incoming identity. For `TOKEN` type, this value should be a regular expression. The incoming token from the client is matched against this expression, and will proceed if the token matches. If the token doesn't match, the client receives a 401 Unauthorized response.
     */
    identityValidationExpression?: pulumi.Input<string>;
    /**
     * Name of the authorizer
     */
    name?: pulumi.Input<string>;
    /**
     * List of the Amazon Cognito user pool ARNs. Each element is of this format: `arn:aws:cognito-idp:{region}:{account_id}:userpool/{user_pool_id}`.
     */
    providerArns?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ID of the associated REST API
     */
    restApi?: pulumi.Input<string | RestApi>;
    /**
     * Type of the authorizer. Possible values are `TOKEN` for a Lambda function using a single authorization token submitted in a custom header, `REQUEST` for a Lambda function using incoming request parameters, or `COGNITO_USER_POOLS` for using an Amazon Cognito user pool. Defaults to `TOKEN`.
     */
    type?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a Authorizer resource.
 */
export interface AuthorizerArgs {
    /**
     * Credentials required for the authorizer. To specify an IAM Role for API Gateway to assume, use the IAM Role ARN.
     */
    authorizerCredentials?: pulumi.Input<string>;
    /**
     * TTL of cached authorizer results in seconds. Defaults to `300`.
     */
    authorizerResultTtlInSeconds?: pulumi.Input<number>;
    /**
     * Authorizer's Uniform Resource Identifier (URI). This must be a well-formed Lambda function URI in the form of `arn:aws:apigateway:{region}:lambda:path/{service_api}`,
     * e.g., `arn:aws:apigateway:us-west-2:lambda:path/2015-03-31/functions/arn:aws:lambda:us-west-2:012345678912:function:my-function/invocations`
     */
    authorizerUri?: pulumi.Input<string>;
    /**
     * Source of the identity in an incoming request. Defaults to `method.request.header.Authorization`. For `REQUEST` type, this may be a comma-separated list of values, including headers, query string parameters and stage variables - e.g., `"method.request.header.SomeHeaderName,method.request.querystring.SomeQueryStringName,stageVariables.SomeStageVariableName"`
     */
    identitySource?: pulumi.Input<string>;
    /**
     * Validation expression for the incoming identity. For `TOKEN` type, this value should be a regular expression. The incoming token from the client is matched against this expression, and will proceed if the token matches. If the token doesn't match, the client receives a 401 Unauthorized response.
     */
    identityValidationExpression?: pulumi.Input<string>;
    /**
     * Name of the authorizer
     */
    name?: pulumi.Input<string>;
    /**
     * List of the Amazon Cognito user pool ARNs. Each element is of this format: `arn:aws:cognito-idp:{region}:{account_id}:userpool/{user_pool_id}`.
     */
    providerArns?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ID of the associated REST API
     */
    restApi: pulumi.Input<string | RestApi>;
    /**
     * Type of the authorizer. Possible values are `TOKEN` for a Lambda function using a single authorization token submitted in a custom header, `REQUEST` for a Lambda function using incoming request parameters, or `COGNITO_USER_POOLS` for using an Amazon Cognito user pool. Defaults to `TOKEN`.
     */
    type?: pulumi.Input<string>;
}
