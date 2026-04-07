import * as pulumi from "@pulumi/pulumi";
/**
 * Provides an API Gateway Usage Plan Key.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = new aws.apigateway.RestApi("test", {name: "MyDemoAPI"});
 * // ...
 * const myusageplan = new aws.apigateway.UsagePlan("myusageplan", {
 *     name: "my_usage_plan",
 *     apiStages: [{
 *         apiId: test.id,
 *         stage: foo.stageName,
 *     }],
 * });
 * const mykey = new aws.apigateway.ApiKey("mykey", {name: "my_key"});
 * const main = new aws.apigateway.UsagePlanKey("main", {
 *     keyId: mykey.id,
 *     keyType: "API_KEY",
 *     usagePlanId: myusageplan.id,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import AWS API Gateway Usage Plan Key using the `USAGE-PLAN-ID/USAGE-PLAN-KEY-ID`. For example:
 *
 * ```sh
 * $ pulumi import aws:apigateway/usagePlanKey:UsagePlanKey key 12345abcde/zzz
 * ```
 */
export declare class UsagePlanKey extends pulumi.CustomResource {
    /**
     * Get an existing UsagePlanKey resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: UsagePlanKeyState, opts?: pulumi.CustomResourceOptions): UsagePlanKey;
    /**
     * Returns true if the given object is an instance of UsagePlanKey.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is UsagePlanKey;
    /**
     * Identifier of the API key resource.
     */
    readonly keyId: pulumi.Output<string>;
    /**
     * Type of the API key resource. Currently, the valid key type is API_KEY.
     */
    readonly keyType: pulumi.Output<string>;
    /**
     * Name of a usage plan key.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Id of the usage plan resource representing to associate the key to.
     */
    readonly usagePlanId: pulumi.Output<string>;
    /**
     * Value of a usage plan key.
     */
    readonly value: pulumi.Output<string>;
    /**
     * Create a UsagePlanKey resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: UsagePlanKeyArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering UsagePlanKey resources.
 */
export interface UsagePlanKeyState {
    /**
     * Identifier of the API key resource.
     */
    keyId?: pulumi.Input<string>;
    /**
     * Type of the API key resource. Currently, the valid key type is API_KEY.
     */
    keyType?: pulumi.Input<string>;
    /**
     * Name of a usage plan key.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Id of the usage plan resource representing to associate the key to.
     */
    usagePlanId?: pulumi.Input<string>;
    /**
     * Value of a usage plan key.
     */
    value?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a UsagePlanKey resource.
 */
export interface UsagePlanKeyArgs {
    /**
     * Identifier of the API key resource.
     */
    keyId: pulumi.Input<string>;
    /**
     * Type of the API key resource. Currently, the valid key type is API_KEY.
     */
    keyType: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Id of the usage plan resource representing to associate the key to.
     */
    usagePlanId: pulumi.Input<string>;
}
