import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a SSM resource data sync.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const hogeBucket = new aws.s3.Bucket("hoge", {bucket: "tf-test-bucket-1234"});
 * const hoge = aws.iam.getPolicyDocument({
 *     statements: [
 *         {
 *             sid: "SSMBucketPermissionsCheck",
 *             effect: "Allow",
 *             principals: [{
 *                 type: "Service",
 *                 identifiers: ["ssm.amazonaws.com"],
 *             }],
 *             actions: ["s3:GetBucketAcl"],
 *             resources: ["arn:aws:s3:::tf-test-bucket-1234"],
 *         },
 *         {
 *             sid: "SSMBucketDelivery",
 *             effect: "Allow",
 *             principals: [{
 *                 type: "Service",
 *                 identifiers: ["ssm.amazonaws.com"],
 *             }],
 *             actions: ["s3:PutObject"],
 *             resources: ["arn:aws:s3:::tf-test-bucket-1234/*"],
 *             conditions: [{
 *                 test: "StringEquals",
 *                 variable: "s3:x-amz-acl",
 *                 values: ["bucket-owner-full-control"],
 *             }],
 *         },
 *     ],
 * });
 * const hogeBucketPolicy = new aws.s3.BucketPolicy("hoge", {
 *     bucket: hogeBucket.id,
 *     policy: hoge.then(hoge => hoge.json),
 * });
 * const foo = new aws.ssm.ResourceDataSync("foo", {
 *     name: "foo",
 *     s3Destination: {
 *         bucketName: hogeBucket.bucket,
 *         region: hogeBucket.region,
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import SSM resource data sync using the `name`. For example:
 *
 * ```sh
 * $ pulumi import aws:ssm/resourceDataSync:ResourceDataSync example example-name
 * ```
 */
export declare class ResourceDataSync extends pulumi.CustomResource {
    /**
     * Get an existing ResourceDataSync resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ResourceDataSyncState, opts?: pulumi.CustomResourceOptions): ResourceDataSync;
    /**
     * Returns true if the given object is an instance of ResourceDataSync.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ResourceDataSync;
    /**
     * Name for the configuration.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Amazon S3 configuration details for the sync.
     */
    readonly s3Destination: pulumi.Output<outputs.ssm.ResourceDataSyncS3Destination>;
    /**
     * Create a ResourceDataSync resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ResourceDataSyncArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ResourceDataSync resources.
 */
export interface ResourceDataSyncState {
    /**
     * Name for the configuration.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Amazon S3 configuration details for the sync.
     */
    s3Destination?: pulumi.Input<inputs.ssm.ResourceDataSyncS3Destination>;
}
/**
 * The set of arguments for constructing a ResourceDataSync resource.
 */
export interface ResourceDataSyncArgs {
    /**
     * Name for the configuration.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Amazon S3 configuration details for the sync.
     */
    s3Destination: pulumi.Input<inputs.ssm.ResourceDataSyncS3Destination>;
}
