import * as pulumi from "@pulumi/pulumi";
/**
 * Provides an AppSync API Key.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.appsync.GraphQLApi("example", {
 *     authenticationType: "API_KEY",
 *     name: "example",
 * });
 * const exampleApiKey = new aws.appsync.ApiKey("example", {
 *     apiId: example.id,
 *     expires: "2018-05-03T04:00:00Z",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_appsync_api_key` using the AppSync API ID and key separated by `:`. For example:
 *
 * ```sh
 * $ pulumi import aws:appsync/apiKey:ApiKey example xxxxx:yyyyy
 * ```
 */
export declare class ApiKey extends pulumi.CustomResource {
    /**
     * Get an existing ApiKey resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ApiKeyState, opts?: pulumi.CustomResourceOptions): ApiKey;
    /**
     * Returns true if the given object is an instance of ApiKey.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ApiKey;
    /**
     * ID of the associated AppSync API
     */
    readonly apiId: pulumi.Output<string>;
    readonly apiKeyId: pulumi.Output<string>;
    /**
     * API key description. Defaults to "Managed by Pulumi".
     */
    readonly description: pulumi.Output<string>;
    /**
     * RFC3339 string representation of the expiry date. Rounded down to nearest hour. By default, it is 7 days from the date of creation.
     */
    readonly expires: pulumi.Output<string | undefined>;
    /**
     * API key
     */
    readonly key: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Create a ApiKey resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ApiKeyArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ApiKey resources.
 */
export interface ApiKeyState {
    /**
     * ID of the associated AppSync API
     */
    apiId?: pulumi.Input<string>;
    apiKeyId?: pulumi.Input<string>;
    /**
     * API key description. Defaults to "Managed by Pulumi".
     */
    description?: pulumi.Input<string>;
    /**
     * RFC3339 string representation of the expiry date. Rounded down to nearest hour. By default, it is 7 days from the date of creation.
     */
    expires?: pulumi.Input<string>;
    /**
     * API key
     */
    key?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a ApiKey resource.
 */
export interface ApiKeyArgs {
    /**
     * ID of the associated AppSync API
     */
    apiId: pulumi.Input<string>;
    /**
     * API key description. Defaults to "Managed by Pulumi".
     */
    description?: pulumi.Input<string>;
    /**
     * RFC3339 string representation of the expiry date. Rounded down to nearest hour. By default, it is 7 days from the date of creation.
     */
    expires?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
