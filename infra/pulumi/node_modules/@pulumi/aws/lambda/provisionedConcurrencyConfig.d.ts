import * as pulumi from "@pulumi/pulumi";
/**
 * Manages an AWS Lambda Provisioned Concurrency Configuration. Use this resource to configure provisioned concurrency for Lambda functions.
 *
 * > **Note:** Setting `skipDestroy` to `true` means that the AWS Provider will not destroy a provisioned concurrency configuration, even when running `pulumi destroy`. The configuration is thus an intentional dangling resource that is not managed by Pulumi and may incur extra expense in your AWS account.
 *
 * ## Example Usage
 *
 * ### Alias Name
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.lambda.ProvisionedConcurrencyConfig("example", {
 *     functionName: exampleAwsLambdaAlias.functionName,
 *     provisionedConcurrentExecutions: 1,
 *     qualifier: exampleAwsLambdaAlias.name,
 * });
 * ```
 *
 * ### Function Version
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.lambda.ProvisionedConcurrencyConfig("example", {
 *     functionName: exampleAwsLambdaFunction.functionName,
 *     provisionedConcurrentExecutions: 1,
 *     qualifier: exampleAwsLambdaFunction.version,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import a Lambda Provisioned Concurrency Configuration using the `function_name` and `qualifier` separated by a comma (`,`). For example:
 *
 * ```sh
 * $ pulumi import aws:lambda/provisionedConcurrencyConfig:ProvisionedConcurrencyConfig example example,production
 * ```
 */
export declare class ProvisionedConcurrencyConfig extends pulumi.CustomResource {
    /**
     * Get an existing ProvisionedConcurrencyConfig resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ProvisionedConcurrencyConfigState, opts?: pulumi.CustomResourceOptions): ProvisionedConcurrencyConfig;
    /**
     * Returns true if the given object is an instance of ProvisionedConcurrencyConfig.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ProvisionedConcurrencyConfig;
    /**
     * Name or Amazon Resource Name (ARN) of the Lambda Function.
     */
    readonly functionName: pulumi.Output<string>;
    /**
     * Amount of capacity to allocate. Must be greater than or equal to 1.
     */
    readonly provisionedConcurrentExecutions: pulumi.Output<number>;
    /**
     * Lambda Function version or Lambda Alias name.
     *
     * The following arguments are optional:
     */
    readonly qualifier: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Whether to retain the provisioned concurrency configuration upon destruction. Defaults to `false`. If set to `true`, the resource is simply removed from state instead.
     */
    readonly skipDestroy: pulumi.Output<boolean | undefined>;
    /**
     * Create a ProvisionedConcurrencyConfig resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ProvisionedConcurrencyConfigArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ProvisionedConcurrencyConfig resources.
 */
export interface ProvisionedConcurrencyConfigState {
    /**
     * Name or Amazon Resource Name (ARN) of the Lambda Function.
     */
    functionName?: pulumi.Input<string>;
    /**
     * Amount of capacity to allocate. Must be greater than or equal to 1.
     */
    provisionedConcurrentExecutions?: pulumi.Input<number>;
    /**
     * Lambda Function version or Lambda Alias name.
     *
     * The following arguments are optional:
     */
    qualifier?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Whether to retain the provisioned concurrency configuration upon destruction. Defaults to `false`. If set to `true`, the resource is simply removed from state instead.
     */
    skipDestroy?: pulumi.Input<boolean>;
}
/**
 * The set of arguments for constructing a ProvisionedConcurrencyConfig resource.
 */
export interface ProvisionedConcurrencyConfigArgs {
    /**
     * Name or Amazon Resource Name (ARN) of the Lambda Function.
     */
    functionName: pulumi.Input<string>;
    /**
     * Amount of capacity to allocate. Must be greater than or equal to 1.
     */
    provisionedConcurrentExecutions: pulumi.Input<number>;
    /**
     * Lambda Function version or Lambda Alias name.
     *
     * The following arguments are optional:
     */
    qualifier: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Whether to retain the provisioned concurrency configuration upon destruction. Defaults to `false`. If set to `true`, the resource is simply removed from state instead.
     */
    skipDestroy?: pulumi.Input<boolean>;
}
