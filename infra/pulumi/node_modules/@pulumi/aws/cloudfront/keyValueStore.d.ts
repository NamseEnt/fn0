import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS CloudFront Key Value Store.
 *
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
 * ```
 *
 * ## Import
 *
 * ### Identity Schema
 *
 * #### Required
 *
 * * `name` (String) Name of the CloudFront Key Value Store.
 *
 * #### Optional
 *
 * * `account_id` (String) AWS Account where this resource is managed.
 *
 * Using `pulumi import`, import CloudFront Key Value Store using the `name`. For example:
 *
 * console
 *
 * % pulumi import aws_cloudfront_key_value_store.example example_store
 */
export declare class KeyValueStore extends pulumi.CustomResource {
    /**
     * Get an existing KeyValueStore resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: KeyValueStoreState, opts?: pulumi.CustomResourceOptions): KeyValueStore;
    /**
     * Returns true if the given object is an instance of KeyValueStore.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is KeyValueStore;
    /**
     * Amazon Resource Name (ARN) identifying your CloudFront KeyValueStore.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Comment.
     */
    readonly comment: pulumi.Output<string | undefined>;
    /**
     * ETag hash of the KeyValueStore.
     */
    readonly etag: pulumi.Output<string>;
    readonly lastModifiedTime: pulumi.Output<string>;
    /**
     * Unique name for your CloudFront KeyValueStore.
     *
     * The following arguments are optional:
     */
    readonly name: pulumi.Output<string>;
    readonly timeouts: pulumi.Output<outputs.cloudfront.KeyValueStoreTimeouts | undefined>;
    /**
     * Create a KeyValueStore resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: KeyValueStoreArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering KeyValueStore resources.
 */
export interface KeyValueStoreState {
    /**
     * Amazon Resource Name (ARN) identifying your CloudFront KeyValueStore.
     */
    arn?: pulumi.Input<string>;
    /**
     * Comment.
     */
    comment?: pulumi.Input<string>;
    /**
     * ETag hash of the KeyValueStore.
     */
    etag?: pulumi.Input<string>;
    lastModifiedTime?: pulumi.Input<string>;
    /**
     * Unique name for your CloudFront KeyValueStore.
     *
     * The following arguments are optional:
     */
    name?: pulumi.Input<string>;
    timeouts?: pulumi.Input<inputs.cloudfront.KeyValueStoreTimeouts>;
}
/**
 * The set of arguments for constructing a KeyValueStore resource.
 */
export interface KeyValueStoreArgs {
    /**
     * Comment.
     */
    comment?: pulumi.Input<string>;
    /**
     * Unique name for your CloudFront KeyValueStore.
     *
     * The following arguments are optional:
     */
    name?: pulumi.Input<string>;
    timeouts?: pulumi.Input<inputs.cloudfront.KeyValueStoreTimeouts>;
}
