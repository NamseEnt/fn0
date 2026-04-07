import * as pulumi from "@pulumi/pulumi";
/**
 * Creates and manages an AWS XRay Encryption Config.
 *
 * > **NOTE:** Removing this resource from the provider has no effect to the encryption configuration within X-Ray.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.xray.EncryptionConfig("example", {type: "NONE"});
 * ```
 *
 * ### With KMS Key
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const current = aws.getCallerIdentity({});
 * const example = current.then(current => aws.iam.getPolicyDocument({
 *     statements: [{
 *         sid: "Enable IAM User Permissions",
 *         effect: "Allow",
 *         principals: [{
 *             type: "AWS",
 *             identifiers: [`arn:aws:iam::${current.accountId}:root`],
 *         }],
 *         actions: ["kms:*"],
 *         resources: ["*"],
 *     }],
 * }));
 * const exampleKey = new aws.kms.Key("example", {
 *     description: "Some Key",
 *     deletionWindowInDays: 7,
 *     policy: example.then(example => example.json),
 * });
 * const exampleEncryptionConfig = new aws.xray.EncryptionConfig("example", {
 *     type: "KMS",
 *     keyId: exampleKey.arn,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import XRay Encryption Config using the region name. For example:
 *
 * ```sh
 * $ pulumi import aws:xray/encryptionConfig:EncryptionConfig example us-west-2
 * ```
 */
export declare class EncryptionConfig extends pulumi.CustomResource {
    /**
     * Get an existing EncryptionConfig resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: EncryptionConfigState, opts?: pulumi.CustomResourceOptions): EncryptionConfig;
    /**
     * Returns true if the given object is an instance of EncryptionConfig.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is EncryptionConfig;
    /**
     * An AWS KMS customer master key (CMK) ARN.
     */
    readonly keyId: pulumi.Output<string | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The type of encryption. Set to `KMS` to use your own key for encryption. Set to `NONE` for default encryption.
     */
    readonly type: pulumi.Output<string>;
    /**
     * Create a EncryptionConfig resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: EncryptionConfigArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering EncryptionConfig resources.
 */
export interface EncryptionConfigState {
    /**
     * An AWS KMS customer master key (CMK) ARN.
     */
    keyId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The type of encryption. Set to `KMS` to use your own key for encryption. Set to `NONE` for default encryption.
     */
    type?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a EncryptionConfig resource.
 */
export interface EncryptionConfigArgs {
    /**
     * An AWS KMS customer master key (CMK) ARN.
     */
    keyId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The type of encryption. Set to `KMS` to use your own key for encryption. Set to `NONE` for default encryption.
     */
    type: pulumi.Input<string>;
}
