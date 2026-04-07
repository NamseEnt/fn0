import * as pulumi from "@pulumi/pulumi";
import * as outputs from "../types/output";
/**
 * Provides details about an AWS Lambda Function URL. Use this data source to retrieve information about an existing function URL configuration.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.lambda.getFunctionUrl({
 *     functionName: "my_lambda_function",
 * });
 * export const functionUrl = example.then(example => example.functionUrl);
 * ```
 *
 * ### With Qualifier
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 * import * as std from "@pulumi/std";
 *
 * const example = aws.lambda.getFunctionUrl({
 *     functionName: exampleAwsLambdaFunction.functionName,
 *     qualifier: "production",
 * });
 * // Use the URL in other resources
 * const lambdaAlias = new aws.route53.Record("lambda_alias", {
 *     zoneId: exampleAwsRoute53Zone.zoneId,
 *     name: "api.example.com",
 *     type: aws.route53.RecordType.CNAME,
 *     ttl: 300,
 *     records: [example.then(example => std.replace({
 *         text: example.functionUrl,
 *         search: "https://",
 *         replace: "",
 *     })).then(invoke => invoke.result)],
 * });
 * ```
 *
 * ### Retrieve CORS Configuration
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.lambda.getFunctionUrl({
 *     functionName: "api_function",
 * });
 * const corsConfig = pulumi.all([example.then(example => example.cors).length, example]).apply(([length, example]) => length > 0 ? example.cors?.[0] : null);
 * const allowedOrigins = corsConfig != null ? corsConfig?.allowOrigins : [];
 * export const corsAllowedOrigins = allowedOrigins;
 * ```
 */
export declare function getFunctionUrl(args: GetFunctionUrlArgs, opts?: pulumi.InvokeOptions): Promise<GetFunctionUrlResult>;
/**
 * A collection of arguments for invoking getFunctionUrl.
 */
export interface GetFunctionUrlArgs {
    /**
     * Name or ARN of the Lambda function.
     *
     * The following arguments are optional:
     */
    functionName: string;
    /**
     * Alias name or `$LATEST`.
     */
    qualifier?: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
}
/**
 * A collection of values returned by getFunctionUrl.
 */
export interface GetFunctionUrlResult {
    /**
     * Type of authentication that the function URL uses.
     */
    readonly authorizationType: string;
    /**
     * Cross-origin resource sharing (CORS) settings for the function URL. See below.
     */
    readonly cors: outputs.lambda.GetFunctionUrlCor[];
    /**
     * When the function URL was created, in [ISO-8601 format](https://www.w3.org/TR/NOTE-datetime).
     */
    readonly creationTime: string;
    /**
     * ARN of the function.
     */
    readonly functionArn: string;
    readonly functionName: string;
    /**
     * HTTP URL endpoint for the function in the format `https://<url_id>.lambda-url.<region>.on.aws/`.
     */
    readonly functionUrl: string;
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    /**
     * Whether the Lambda function responds in `BUFFERED` or `RESPONSE_STREAM` mode.
     */
    readonly invokeMode: string;
    /**
     * When the function URL configuration was last updated, in [ISO-8601 format](https://www.w3.org/TR/NOTE-datetime).
     */
    readonly lastModifiedTime: string;
    readonly qualifier?: string;
    readonly region: string;
    /**
     * Generated ID for the endpoint.
     */
    readonly urlId: string;
}
/**
 * Provides details about an AWS Lambda Function URL. Use this data source to retrieve information about an existing function URL configuration.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.lambda.getFunctionUrl({
 *     functionName: "my_lambda_function",
 * });
 * export const functionUrl = example.then(example => example.functionUrl);
 * ```
 *
 * ### With Qualifier
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 * import * as std from "@pulumi/std";
 *
 * const example = aws.lambda.getFunctionUrl({
 *     functionName: exampleAwsLambdaFunction.functionName,
 *     qualifier: "production",
 * });
 * // Use the URL in other resources
 * const lambdaAlias = new aws.route53.Record("lambda_alias", {
 *     zoneId: exampleAwsRoute53Zone.zoneId,
 *     name: "api.example.com",
 *     type: aws.route53.RecordType.CNAME,
 *     ttl: 300,
 *     records: [example.then(example => std.replace({
 *         text: example.functionUrl,
 *         search: "https://",
 *         replace: "",
 *     })).then(invoke => invoke.result)],
 * });
 * ```
 *
 * ### Retrieve CORS Configuration
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.lambda.getFunctionUrl({
 *     functionName: "api_function",
 * });
 * const corsConfig = pulumi.all([example.then(example => example.cors).length, example]).apply(([length, example]) => length > 0 ? example.cors?.[0] : null);
 * const allowedOrigins = corsConfig != null ? corsConfig?.allowOrigins : [];
 * export const corsAllowedOrigins = allowedOrigins;
 * ```
 */
export declare function getFunctionUrlOutput(args: GetFunctionUrlOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetFunctionUrlResult>;
/**
 * A collection of arguments for invoking getFunctionUrl.
 */
export interface GetFunctionUrlOutputArgs {
    /**
     * Name or ARN of the Lambda function.
     *
     * The following arguments are optional:
     */
    functionName: pulumi.Input<string>;
    /**
     * Alias name or `$LATEST`.
     */
    qualifier?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
