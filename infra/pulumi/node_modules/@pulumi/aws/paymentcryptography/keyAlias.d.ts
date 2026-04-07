import * as pulumi from "@pulumi/pulumi";
/**
 * Resource for managing an AWS Payment Cryptography Control Plane Key Alias.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = new aws.paymentcryptography.Key("test", {
 *     exportable: true,
 *     keyAttributes: [{
 *         keyAlgorithm: "TDES_3KEY",
 *         keyClass: "SYMMETRIC_KEY",
 *         keyUsage: "TR31_P0_PIN_ENCRYPTION_KEY",
 *         keyModesOfUses: [{
 *             decrypt: true,
 *             encrypt: true,
 *             wrap: true,
 *             unwrap: true,
 *         }],
 *     }],
 * });
 * const testKeyAlias = new aws.paymentcryptography.KeyAlias("test", {
 *     aliasName: "alias/test-alias",
 *     keyArn: test.arn,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Payment Cryptography Control Plane Key Alias using the `alias/4681482429376900170`. For example:
 *
 * ```sh
 * $ pulumi import aws:paymentcryptography/keyAlias:KeyAlias example alias/4681482429376900170
 * ```
 */
export declare class KeyAlias extends pulumi.CustomResource {
    /**
     * Get an existing KeyAlias resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: KeyAliasState, opts?: pulumi.CustomResourceOptions): KeyAlias;
    /**
     * Returns true if the given object is an instance of KeyAlias.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is KeyAlias;
    /**
     * Name of the Key Alias.
     *
     * The following arguments are optional:
     */
    readonly aliasName: pulumi.Output<string>;
    /**
     * ARN of the key.
     */
    readonly keyArn: pulumi.Output<string | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Create a KeyAlias resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: KeyAliasArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering KeyAlias resources.
 */
export interface KeyAliasState {
    /**
     * Name of the Key Alias.
     *
     * The following arguments are optional:
     */
    aliasName?: pulumi.Input<string>;
    /**
     * ARN of the key.
     */
    keyArn?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a KeyAlias resource.
 */
export interface KeyAliasArgs {
    /**
     * Name of the Key Alias.
     *
     * The following arguments are optional:
     */
    aliasName: pulumi.Input<string>;
    /**
     * ARN of the key.
     */
    keyArn?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
