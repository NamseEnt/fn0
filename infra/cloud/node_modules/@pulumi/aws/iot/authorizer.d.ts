import * as pulumi from "@pulumi/pulumi";
/**
 * Creates and manages an AWS IoT Authorizer.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 * import * as std from "@pulumi/std";
 *
 * const example = new aws.iot.Authorizer("example", {
 *     name: "example",
 *     authorizerFunctionArn: exampleAwsLambdaFunction.arn,
 *     signingDisabled: false,
 *     status: "ACTIVE",
 *     tokenKeyName: "Token-Header",
 *     tokenSigningPublicKeys: {
 *         Key1: std.file({
 *             input: "test-fixtures/iot-authorizer-signing-key.pem",
 *         }).then(invoke => invoke.result),
 *     },
 *     tags: {
 *         Name: "example",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import IOT Authorizers using the name. For example:
 *
 * ```sh
 * $ pulumi import aws:iot/authorizer:Authorizer example example
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
     * The ARN of the authorizer.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The ARN of the authorizer's Lambda function.
     */
    readonly authorizerFunctionArn: pulumi.Output<string>;
    /**
     * Specifies whether the HTTP caching is enabled or not. Default: `false`.
     */
    readonly enableCachingForHttp: pulumi.Output<boolean | undefined>;
    /**
     * The name of the authorizer.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Specifies whether AWS IoT validates the token signature in an authorization request. Default: `false`.
     */
    readonly signingDisabled: pulumi.Output<boolean | undefined>;
    /**
     * The status of Authorizer request at creation. Valid values: `ACTIVE`, `INACTIVE`. Default: `ACTIVE`.
     */
    readonly status: pulumi.Output<string | undefined>;
    /**
     * Map of tags to assign to this resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * A map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * The name of the token key used to extract the token from the HTTP headers. This value is required if signing is enabled in your authorizer.
     */
    readonly tokenKeyName: pulumi.Output<string | undefined>;
    /**
     * The public keys used to verify the digital signature returned by your custom authentication service. This value is required if signing is enabled in your authorizer.
     */
    readonly tokenSigningPublicKeys: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
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
     * The ARN of the authorizer.
     */
    arn?: pulumi.Input<string>;
    /**
     * The ARN of the authorizer's Lambda function.
     */
    authorizerFunctionArn?: pulumi.Input<string>;
    /**
     * Specifies whether the HTTP caching is enabled or not. Default: `false`.
     */
    enableCachingForHttp?: pulumi.Input<boolean>;
    /**
     * The name of the authorizer.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Specifies whether AWS IoT validates the token signature in an authorization request. Default: `false`.
     */
    signingDisabled?: pulumi.Input<boolean>;
    /**
     * The status of Authorizer request at creation. Valid values: `ACTIVE`, `INACTIVE`. Default: `ACTIVE`.
     */
    status?: pulumi.Input<string>;
    /**
     * Map of tags to assign to this resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * A map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The name of the token key used to extract the token from the HTTP headers. This value is required if signing is enabled in your authorizer.
     */
    tokenKeyName?: pulumi.Input<string>;
    /**
     * The public keys used to verify the digital signature returned by your custom authentication service. This value is required if signing is enabled in your authorizer.
     */
    tokenSigningPublicKeys?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
/**
 * The set of arguments for constructing a Authorizer resource.
 */
export interface AuthorizerArgs {
    /**
     * The ARN of the authorizer's Lambda function.
     */
    authorizerFunctionArn: pulumi.Input<string>;
    /**
     * Specifies whether the HTTP caching is enabled or not. Default: `false`.
     */
    enableCachingForHttp?: pulumi.Input<boolean>;
    /**
     * The name of the authorizer.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Specifies whether AWS IoT validates the token signature in an authorization request. Default: `false`.
     */
    signingDisabled?: pulumi.Input<boolean>;
    /**
     * The status of Authorizer request at creation. Valid values: `ACTIVE`, `INACTIVE`. Default: `ACTIVE`.
     */
    status?: pulumi.Input<string>;
    /**
     * Map of tags to assign to this resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The name of the token key used to extract the token from the HTTP headers. This value is required if signing is enabled in your authorizer.
     */
    tokenKeyName?: pulumi.Input<string>;
    /**
     * The public keys used to verify the digital signature returned by your custom authentication service. This value is required if signing is enabled in your authorizer.
     */
    tokenSigningPublicKeys?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
