import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * ## Import
 *
 * Using `pulumi import`, import AWS CloudFront KeyValueStore Key Value Pairs using the `key_value_store_arn`. For example:
 *
 * ```sh
 * $ pulumi import aws:cloudfront/keyvaluestoreKeysExclusive:KeyvaluestoreKeysExclusive example arn:aws:cloudfront::111111111111:key-value-store/8562g61f-caba-2845-9d99-b97diwae5d3c
 * ```
 */
export declare class KeyvaluestoreKeysExclusive extends pulumi.CustomResource {
    /**
     * Get an existing KeyvaluestoreKeysExclusive resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: KeyvaluestoreKeysExclusiveState, opts?: pulumi.CustomResourceOptions): KeyvaluestoreKeysExclusive;
    /**
     * Returns true if the given object is an instance of KeyvaluestoreKeysExclusive.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is KeyvaluestoreKeysExclusive;
    /**
     * Amazon Resource Name (ARN) of the Key Value Store.
     *
     * The following arguments are optional:
     */
    readonly keyValueStoreArn: pulumi.Output<string>;
    /**
     * Maximum resource key values pairs that will update in a single API request. AWS has a default quota of 50 keys or a 3 MB payload, whichever is reached first. Defaults to `50`.
     */
    readonly maxBatchSize: pulumi.Output<number>;
    /**
     * A list of all resource key value pairs associated with the KeyValueStore.
     * See `resourceKeyValuePair` below.
     */
    readonly resourceKeyValuePairs: pulumi.Output<outputs.cloudfront.KeyvaluestoreKeysExclusiveResourceKeyValuePair[] | undefined>;
    /**
     * Total size of the Key Value Store in bytes.
     */
    readonly totalSizeInBytes: pulumi.Output<number>;
    /**
     * Create a KeyvaluestoreKeysExclusive resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: KeyvaluestoreKeysExclusiveArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering KeyvaluestoreKeysExclusive resources.
 */
export interface KeyvaluestoreKeysExclusiveState {
    /**
     * Amazon Resource Name (ARN) of the Key Value Store.
     *
     * The following arguments are optional:
     */
    keyValueStoreArn?: pulumi.Input<string>;
    /**
     * Maximum resource key values pairs that will update in a single API request. AWS has a default quota of 50 keys or a 3 MB payload, whichever is reached first. Defaults to `50`.
     */
    maxBatchSize?: pulumi.Input<number>;
    /**
     * A list of all resource key value pairs associated with the KeyValueStore.
     * See `resourceKeyValuePair` below.
     */
    resourceKeyValuePairs?: pulumi.Input<pulumi.Input<inputs.cloudfront.KeyvaluestoreKeysExclusiveResourceKeyValuePair>[]>;
    /**
     * Total size of the Key Value Store in bytes.
     */
    totalSizeInBytes?: pulumi.Input<number>;
}
/**
 * The set of arguments for constructing a KeyvaluestoreKeysExclusive resource.
 */
export interface KeyvaluestoreKeysExclusiveArgs {
    /**
     * Amazon Resource Name (ARN) of the Key Value Store.
     *
     * The following arguments are optional:
     */
    keyValueStoreArn: pulumi.Input<string>;
    /**
     * Maximum resource key values pairs that will update in a single API request. AWS has a default quota of 50 keys or a 3 MB payload, whichever is reached first. Defaults to `50`.
     */
    maxBatchSize?: pulumi.Input<number>;
    /**
     * A list of all resource key value pairs associated with the KeyValueStore.
     * See `resourceKeyValuePair` below.
     */
    resourceKeyValuePairs?: pulumi.Input<pulumi.Input<inputs.cloudfront.KeyvaluestoreKeysExclusiveResourceKeyValuePair>[]>;
}
