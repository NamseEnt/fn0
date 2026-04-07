import * as pulumi from "@pulumi/pulumi";
/**
 * Manages an AWS Lambda Runtime Management Config. Use this resource to control how Lambda updates the runtime for your function.
 *
 * Refer to the [AWS Lambda documentation](https://docs.aws.amazon.com/lambda/latest/dg/lambda-runtimes.html) for supported runtimes.
 *
 * > **Note:** Deletion of this resource returns the runtime update mode to `Auto` (the default behavior). To leave the configured runtime management options in-place, use a `removed` block with the destroy lifecycle set to `false`.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.lambda.RuntimeManagementConfig("example", {
 *     functionName: exampleAwsLambdaFunction.functionName,
 *     updateRuntimeOn: "FunctionUpdate",
 * });
 * ```
 *
 * ### Manual Update
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.lambda.RuntimeManagementConfig("example", {
 *     functionName: exampleAwsLambdaFunction.functionName,
 *     updateRuntimeOn: "Manual",
 *     runtimeVersionArn: "arn:aws:lambda:us-east-1::runtime:abcd1234",
 * });
 * ```
 *
 * > **Note:** Once the runtime update mode is set to `Manual`, the `aws.lambda.Function` `runtime` cannot be updated. To upgrade a runtime, the `updateRuntimeOn` argument must be set to `Auto` or `FunctionUpdate` prior to changing the function's `runtime` argument.
 *
 * ## Import
 *
 * Using `pulumi import`, import Lambda Runtime Management Config using a comma-delimited string combining `function_name` and `qualifier`. For example:
 *
 * ```sh
 * $ pulumi import aws:lambda/runtimeManagementConfig:RuntimeManagementConfig example example,$LATEST
 * ```
 */
export declare class RuntimeManagementConfig extends pulumi.CustomResource {
    /**
     * Get an existing RuntimeManagementConfig resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: RuntimeManagementConfigState, opts?: pulumi.CustomResourceOptions): RuntimeManagementConfig;
    /**
     * Returns true if the given object is an instance of RuntimeManagementConfig.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is RuntimeManagementConfig;
    /**
     * ARN of the function.
     */
    readonly functionArn: pulumi.Output<string>;
    /**
     * Name or ARN of the Lambda function.
     *
     * The following arguments are optional:
     */
    readonly functionName: pulumi.Output<string>;
    /**
     * Version of the function. This can be `$LATEST` or a published version number. If omitted, this resource will manage the runtime configuration for `$LATEST`.
     */
    readonly qualifier: pulumi.Output<string | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * ARN of the runtime version. Only required when `updateRuntimeOn` is `Manual`.
     */
    readonly runtimeVersionArn: pulumi.Output<string | undefined>;
    /**
     * Runtime update mode. Valid values are `Auto`, `FunctionUpdate`, and `Manual`. When a function is created, the default mode is `Auto`.
     */
    readonly updateRuntimeOn: pulumi.Output<string | undefined>;
    /**
     * Create a RuntimeManagementConfig resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: RuntimeManagementConfigArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering RuntimeManagementConfig resources.
 */
export interface RuntimeManagementConfigState {
    /**
     * ARN of the function.
     */
    functionArn?: pulumi.Input<string>;
    /**
     * Name or ARN of the Lambda function.
     *
     * The following arguments are optional:
     */
    functionName?: pulumi.Input<string>;
    /**
     * Version of the function. This can be `$LATEST` or a published version number. If omitted, this resource will manage the runtime configuration for `$LATEST`.
     */
    qualifier?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ARN of the runtime version. Only required when `updateRuntimeOn` is `Manual`.
     */
    runtimeVersionArn?: pulumi.Input<string>;
    /**
     * Runtime update mode. Valid values are `Auto`, `FunctionUpdate`, and `Manual`. When a function is created, the default mode is `Auto`.
     */
    updateRuntimeOn?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a RuntimeManagementConfig resource.
 */
export interface RuntimeManagementConfigArgs {
    /**
     * Name or ARN of the Lambda function.
     *
     * The following arguments are optional:
     */
    functionName: pulumi.Input<string>;
    /**
     * Version of the function. This can be `$LATEST` or a published version number. If omitted, this resource will manage the runtime configuration for `$LATEST`.
     */
    qualifier?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ARN of the runtime version. Only required when `updateRuntimeOn` is `Manual`.
     */
    runtimeVersionArn?: pulumi.Input<string>;
    /**
     * Runtime update mode. Valid values are `Auto`, `FunctionUpdate`, and `Manual`. When a function is created, the default mode is `Auto`.
     */
    updateRuntimeOn?: pulumi.Input<string>;
}
