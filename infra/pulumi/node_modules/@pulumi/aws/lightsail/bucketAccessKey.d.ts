import * as pulumi from "@pulumi/pulumi";
/**
 * Manages a Lightsail bucket access key. Use this resource to create credentials that allow programmatic access to your Lightsail bucket via API requests.
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
 * const exampleBucketAccessKey = new aws.lightsail.BucketAccessKey("example", {bucketName: example.id});
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_lightsail_bucket_access_key` using the `id` attribute. For example:
 *
 * ```sh
 * $ pulumi import aws:lightsail/bucketAccessKey:BucketAccessKey example example-bucket,AKIAIOSFODNN7EXAMPLE
 * ```
 */
export declare class BucketAccessKey extends pulumi.CustomResource {
    /**
     * Get an existing BucketAccessKey resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: BucketAccessKeyState, opts?: pulumi.CustomResourceOptions): BucketAccessKey;
    /**
     * Returns true if the given object is an instance of BucketAccessKey.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is BucketAccessKey;
    /**
     * Access key ID.
     */
    readonly accessKeyId: pulumi.Output<string>;
    /**
     * Name of the bucket that the access key will belong to and grant access to.
     */
    readonly bucketName: pulumi.Output<string>;
    /**
     * Date and time when the access key was created.
     */
    readonly createdAt: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Secret access key used to sign requests. This attribute is not available for imported resources. Note that this will be written to the state file.
     */
    readonly secretAccessKey: pulumi.Output<string>;
    /**
     * Status of the access key.
     */
    readonly status: pulumi.Output<string>;
    /**
     * Create a BucketAccessKey resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: BucketAccessKeyArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering BucketAccessKey resources.
 */
export interface BucketAccessKeyState {
    /**
     * Access key ID.
     */
    accessKeyId?: pulumi.Input<string>;
    /**
     * Name of the bucket that the access key will belong to and grant access to.
     */
    bucketName?: pulumi.Input<string>;
    /**
     * Date and time when the access key was created.
     */
    createdAt?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Secret access key used to sign requests. This attribute is not available for imported resources. Note that this will be written to the state file.
     */
    secretAccessKey?: pulumi.Input<string>;
    /**
     * Status of the access key.
     */
    status?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a BucketAccessKey resource.
 */
export interface BucketAccessKeyArgs {
    /**
     * Name of the bucket that the access key will belong to and grant access to.
     */
    bucketName: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
