import * as pulumi from "@pulumi/pulumi";
/**
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.cloudfront.KeyValueStore("example", {
 *     name: "ExampleKeyValueStore",
 *     comment: "This is an example key value store",
 * });
 * const exampleKeyvaluestoreKey = new aws.cloudfront.KeyvaluestoreKey("example", {
 *     keyValueStoreArn: example.arn,
 *     key: "Test Key",
 *     value: "Test Value",
 * });
 * ```
 *
 * ## Import
 *
 * ### Identity Schema
 *
 * #### Required
 *
 * * `key_value_store_arn` (String) ARN of the CloudFront Key Value Store.
 *
 * * `key` (String) Key name.
 *
 * #### Optional
 *
 * * `account_id` (String) AWS Account where this resource is managed.
 *
 * Using `pulumi import`, import CloudFront KeyValueStore Key using the `key_value_store_arn` and 'key' separated by `,`. For example:
 *
 * console
 *
 * % pulumi import aws_cloudfrontkeyvaluestore_key.example arn:aws:cloudfront::111111111111:key-value-store/8562g61f-caba-2845-9d99-b97diwae5d3c,someKey
 */
export declare class KeyvaluestoreKey extends pulumi.CustomResource {
    /**
     * Get an existing KeyvaluestoreKey resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: KeyvaluestoreKeyState, opts?: pulumi.CustomResourceOptions): KeyvaluestoreKey;
    /**
     * Returns true if the given object is an instance of KeyvaluestoreKey.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is KeyvaluestoreKey;
    /**
     * Key to put.
     */
    readonly key: pulumi.Output<string>;
    /**
     * Amazon Resource Name (ARN) of the Key Value Store.
     */
    readonly keyValueStoreArn: pulumi.Output<string>;
    /**
     * Total size of the Key Value Store in bytes.
     */
    readonly totalSizeInBytes: pulumi.Output<number>;
    /**
     * Value to put.
     */
    readonly value: pulumi.Output<string>;
    /**
     * Create a KeyvaluestoreKey resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: KeyvaluestoreKeyArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering KeyvaluestoreKey resources.
 */
export interface KeyvaluestoreKeyState {
    /**
     * Key to put.
     */
    key?: pulumi.Input<string>;
    /**
     * Amazon Resource Name (ARN) of the Key Value Store.
     */
    keyValueStoreArn?: pulumi.Input<string>;
    /**
     * Total size of the Key Value Store in bytes.
     */
    totalSizeInBytes?: pulumi.Input<number>;
    /**
     * Value to put.
     */
    value?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a KeyvaluestoreKey resource.
 */
export interface KeyvaluestoreKeyArgs {
    /**
     * Key to put.
     */
    key: pulumi.Input<string>;
    /**
     * Amazon Resource Name (ARN) of the Key Value Store.
     */
    keyValueStoreArn: pulumi.Input<string>;
    /**
     * Value to put.
     */
    value: pulumi.Input<string>;
}
