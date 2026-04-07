import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a CloudFront Function resource. With CloudFront Functions in Amazon CloudFront, you can write lightweight functions in JavaScript for high-scale, latency-sensitive CDN customizations.
 *
 * See [CloudFront Functions](https://docs.aws.amazon.com/AmazonCloudFront/latest/DeveloperGuide/cloudfront-functions.html)
 *
 * > **NOTE:** You cannot delete a function if it’s associated with a cache behavior. First, update your distributions to remove the function association from all cache behaviors, then delete the function.
 *
 * ## Example Usage
 *
 * ## Import
 *
 * Using `pulumi import`, import CloudFront Functions using the `name`. For example:
 *
 * ```sh
 * $ pulumi import aws:cloudfront/function:Function test my_test_function
 * ```
 */
export declare class Function extends pulumi.CustomResource {
    /**
     * Get an existing Function resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: FunctionState, opts?: pulumi.CustomResourceOptions): Function;
    /**
     * Returns true if the given object is an instance of Function.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Function;
    /**
     * Amazon Resource Name (ARN) identifying your CloudFront Function.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Source code of the function
     */
    readonly code: pulumi.Output<string>;
    /**
     * Comment.
     */
    readonly comment: pulumi.Output<string | undefined>;
    /**
     * ETag hash of the function. This is the value for the `DEVELOPMENT` stage of the function.
     */
    readonly etag: pulumi.Output<string>;
    /**
     * List of `aws.cloudfront.KeyValueStore` ARNs to be associated to the function. AWS limits associations to one key value store per function.
     */
    readonly keyValueStoreAssociations: pulumi.Output<string[] | undefined>;
    /**
     * ETag hash of any `LIVE` stage of the function.
     */
    readonly liveStageEtag: pulumi.Output<string>;
    /**
     * Unique name for your CloudFront Function.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Whether to publish creation/change as Live CloudFront Function Version. Defaults to `true`.
     */
    readonly publish: pulumi.Output<boolean | undefined>;
    /**
     * Identifier of the function's runtime. Valid values are `cloudfront-js-1.0` and `cloudfront-js-2.0`.
     *
     * The following arguments are optional:
     */
    readonly runtime: pulumi.Output<string>;
    /**
     * Status of the function. Can be `UNPUBLISHED`, `UNASSOCIATED` or `ASSOCIATED`.
     */
    readonly status: pulumi.Output<string>;
    /**
     * Create a Function resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: FunctionArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Function resources.
 */
export interface FunctionState {
    /**
     * Amazon Resource Name (ARN) identifying your CloudFront Function.
     */
    arn?: pulumi.Input<string>;
    /**
     * Source code of the function
     */
    code?: pulumi.Input<string>;
    /**
     * Comment.
     */
    comment?: pulumi.Input<string>;
    /**
     * ETag hash of the function. This is the value for the `DEVELOPMENT` stage of the function.
     */
    etag?: pulumi.Input<string>;
    /**
     * List of `aws.cloudfront.KeyValueStore` ARNs to be associated to the function. AWS limits associations to one key value store per function.
     */
    keyValueStoreAssociations?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * ETag hash of any `LIVE` stage of the function.
     */
    liveStageEtag?: pulumi.Input<string>;
    /**
     * Unique name for your CloudFront Function.
     */
    name?: pulumi.Input<string>;
    /**
     * Whether to publish creation/change as Live CloudFront Function Version. Defaults to `true`.
     */
    publish?: pulumi.Input<boolean>;
    /**
     * Identifier of the function's runtime. Valid values are `cloudfront-js-1.0` and `cloudfront-js-2.0`.
     *
     * The following arguments are optional:
     */
    runtime?: pulumi.Input<string>;
    /**
     * Status of the function. Can be `UNPUBLISHED`, `UNASSOCIATED` or `ASSOCIATED`.
     */
    status?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a Function resource.
 */
export interface FunctionArgs {
    /**
     * Source code of the function
     */
    code: pulumi.Input<string>;
    /**
     * Comment.
     */
    comment?: pulumi.Input<string>;
    /**
     * List of `aws.cloudfront.KeyValueStore` ARNs to be associated to the function. AWS limits associations to one key value store per function.
     */
    keyValueStoreAssociations?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Unique name for your CloudFront Function.
     */
    name?: pulumi.Input<string>;
    /**
     * Whether to publish creation/change as Live CloudFront Function Version. Defaults to `true`.
     */
    publish?: pulumi.Input<boolean>;
    /**
     * Identifier of the function's runtime. Valid values are `cloudfront-js-1.0` and `cloudfront-js-2.0`.
     *
     * The following arguments are optional:
     */
    runtime: pulumi.Input<string>;
}
