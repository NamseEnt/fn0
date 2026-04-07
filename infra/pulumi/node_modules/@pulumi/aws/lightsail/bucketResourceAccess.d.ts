import * as pulumi from "@pulumi/pulumi";
/**
 * Manages a Lightsail bucket resource access. Use this resource to grant a Lightsail resource (such as an instance) access to a specific bucket.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.lightsail.Bucket("example", {
 *     name: "example-bucket",
 *     bundleId: "small_1_0",
 * });
 * const exampleInstance = new aws.lightsail.Instance("example", {
 *     name: "example-instance",
 *     availabilityZone: "us-east-1b",
 *     blueprintId: "amazon_linux_2",
 *     bundleId: "nano_3_0",
 * });
 * const exampleBucketResourceAccess = new aws.lightsail.BucketResourceAccess("example", {
 *     bucketName: example.id,
 *     resourceName: exampleInstance.id,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_lightsail_bucket_resource_access` using the `id` attribute. For example:
 *
 * ```sh
 * $ pulumi import aws:lightsail/bucketResourceAccess:BucketResourceAccess example example-bucket,example-instance
 * ```
 */
export declare class BucketResourceAccess extends pulumi.CustomResource {
    /**
     * Get an existing BucketResourceAccess resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: BucketResourceAccessState, opts?: pulumi.CustomResourceOptions): BucketResourceAccess;
    /**
     * Returns true if the given object is an instance of BucketResourceAccess.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is BucketResourceAccess;
    /**
     * Name of the bucket to grant access to.
     */
    readonly bucketName: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Name of the resource to grant bucket access.
     */
    readonly resourceName: pulumi.Output<string>;
    /**
     * Create a BucketResourceAccess resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: BucketResourceAccessArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering BucketResourceAccess resources.
 */
export interface BucketResourceAccessState {
    /**
     * Name of the bucket to grant access to.
     */
    bucketName?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Name of the resource to grant bucket access.
     */
    resourceName?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a BucketResourceAccess resource.
 */
export interface BucketResourceAccessArgs {
    /**
     * Name of the bucket to grant access to.
     */
    bucketName: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Name of the resource to grant bucket access.
     */
    resourceName: pulumi.Input<string>;
}
