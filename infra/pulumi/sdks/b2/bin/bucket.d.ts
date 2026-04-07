import * as pulumi from "@pulumi/pulumi";
import * as inputs from "./types/input";
import * as outputs from "./types/output";
export declare class Bucket extends pulumi.CustomResource {
    /**
     * Get an existing Bucket resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: BucketState, opts?: pulumi.CustomResourceOptions): Bucket;
    /**
     * Returns true if the given object is an instance of Bucket.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Bucket;
    /**
     * Account ID that the bucket belongs to.
     */
    readonly accountId: pulumi.Output<string>;
    readonly b2BucketId: pulumi.Output<string>;
    /**
     * The ID of the bucket.
     */
    readonly bucketId: pulumi.Output<string>;
    /**
     * User-defined information to be stored with the bucket.
     */
    readonly bucketInfo: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * The name of the bucket. **Modifying this attribute will force creation of a new resource.**
     */
    readonly bucketName: pulumi.Output<string>;
    /**
     * The bucket type. Either 'allPublic', meaning that files in this bucket can be downloaded by anybody, or 'allPrivate'.
     */
    readonly bucketType: pulumi.Output<string>;
    /**
     * The initial list of CORS rules for this bucket.
     */
    readonly corsRules: pulumi.Output<outputs.BucketCorsRule[] | undefined>;
    /**
     * The default server-side encryption settings for this bucket.
     */
    readonly defaultServerSideEncryption: pulumi.Output<outputs.BucketDefaultServerSideEncryption | undefined>;
    /**
     * File lock enabled flag, and default retention settings.
     */
    readonly fileLockConfigurations: pulumi.Output<outputs.BucketFileLockConfiguration[] | undefined>;
    /**
     * The initial list of lifecycle rules for this bucket.
     */
    readonly lifecycleRules: pulumi.Output<outputs.BucketLifecycleRule[] | undefined>;
    /**
     * List of bucket options.
     */
    readonly options: pulumi.Output<string[]>;
    /**
     * Bucket revision.
     */
    readonly revision: pulumi.Output<number>;
    /**
     * Create a Bucket resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: BucketArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Bucket resources.
 */
export interface BucketState {
    /**
     * Account ID that the bucket belongs to.
     */
    accountId?: pulumi.Input<string>;
    b2BucketId?: pulumi.Input<string>;
    /**
     * The ID of the bucket.
     */
    bucketId?: pulumi.Input<string>;
    /**
     * User-defined information to be stored with the bucket.
     */
    bucketInfo?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The name of the bucket. **Modifying this attribute will force creation of a new resource.**
     */
    bucketName?: pulumi.Input<string>;
    /**
     * The bucket type. Either 'allPublic', meaning that files in this bucket can be downloaded by anybody, or 'allPrivate'.
     */
    bucketType?: pulumi.Input<string>;
    /**
     * The initial list of CORS rules for this bucket.
     */
    corsRules?: pulumi.Input<pulumi.Input<inputs.BucketCorsRule>[]>;
    /**
     * The default server-side encryption settings for this bucket.
     */
    defaultServerSideEncryption?: pulumi.Input<inputs.BucketDefaultServerSideEncryption>;
    /**
     * File lock enabled flag, and default retention settings.
     */
    fileLockConfigurations?: pulumi.Input<pulumi.Input<inputs.BucketFileLockConfiguration>[]>;
    /**
     * The initial list of lifecycle rules for this bucket.
     */
    lifecycleRules?: pulumi.Input<pulumi.Input<inputs.BucketLifecycleRule>[]>;
    /**
     * List of bucket options.
     */
    options?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Bucket revision.
     */
    revision?: pulumi.Input<number>;
}
/**
 * The set of arguments for constructing a Bucket resource.
 */
export interface BucketArgs {
    b2BucketId?: pulumi.Input<string>;
    /**
     * User-defined information to be stored with the bucket.
     */
    bucketInfo?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The name of the bucket. **Modifying this attribute will force creation of a new resource.**
     */
    bucketName: pulumi.Input<string>;
    /**
     * The bucket type. Either 'allPublic', meaning that files in this bucket can be downloaded by anybody, or 'allPrivate'.
     */
    bucketType: pulumi.Input<string>;
    /**
     * The initial list of CORS rules for this bucket.
     */
    corsRules?: pulumi.Input<pulumi.Input<inputs.BucketCorsRule>[]>;
    /**
     * The default server-side encryption settings for this bucket.
     */
    defaultServerSideEncryption?: pulumi.Input<inputs.BucketDefaultServerSideEncryption>;
    /**
     * File lock enabled flag, and default retention settings.
     */
    fileLockConfigurations?: pulumi.Input<pulumi.Input<inputs.BucketFileLockConfiguration>[]>;
    /**
     * The initial list of lifecycle rules for this bucket.
     */
    lifecycleRules?: pulumi.Input<pulumi.Input<inputs.BucketLifecycleRule>[]>;
}
