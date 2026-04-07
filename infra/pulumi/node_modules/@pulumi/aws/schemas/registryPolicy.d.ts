import * as pulumi from "@pulumi/pulumi";
/**
 * Resource for managing an AWS EventBridge Schemas Registry Policy.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.iam.getPolicyDocument({
 *     statements: [{
 *         sid: "example",
 *         effect: "Allow",
 *         principals: [{
 *             type: "AWS",
 *             identifiers: ["109876543210"],
 *         }],
 *         actions: ["schemas:*"],
 *         resources: [
 *             "arn:aws:schemas:us-east-1:123456789012:registry/example",
 *             "arn:aws:schemas:us-east-1:123456789012:schema/example*",
 *         ],
 *     }],
 * });
 * const exampleRegistryPolicy = new aws.schemas.RegistryPolicy("example", {
 *     registryName: "example",
 *     policy: example.then(example => example.json),
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import EventBridge Schema Registry Policy using the `registry_name`. For example:
 *
 * ```sh
 * $ pulumi import aws:schemas/registryPolicy:RegistryPolicy example example
 * ```
 */
export declare class RegistryPolicy extends pulumi.CustomResource {
    /**
     * Get an existing RegistryPolicy resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: RegistryPolicyState, opts?: pulumi.CustomResourceOptions): RegistryPolicy;
    /**
     * Returns true if the given object is an instance of RegistryPolicy.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is RegistryPolicy;
    /**
     * Resource Policy for EventBridge Schema Registry
     */
    readonly policy: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Name of EventBridge Schema Registry
     */
    readonly registryName: pulumi.Output<string>;
    /**
     * Create a RegistryPolicy resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: RegistryPolicyArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering RegistryPolicy resources.
 */
export interface RegistryPolicyState {
    /**
     * Resource Policy for EventBridge Schema Registry
     */
    policy?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Name of EventBridge Schema Registry
     */
    registryName?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a RegistryPolicy resource.
 */
export interface RegistryPolicyArgs {
    /**
     * Resource Policy for EventBridge Schema Registry
     */
    policy: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Name of EventBridge Schema Registry
     */
    registryName: pulumi.Input<string>;
}
