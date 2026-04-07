import * as pulumi from "@pulumi/pulumi";
import * as inputs from "./types/input";
import * as outputs from "./types/output";
export declare class BucketFileVersion extends pulumi.CustomResource {
    /**
     * Get an existing BucketFileVersion resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: BucketFileVersionState, opts?: pulumi.CustomResourceOptions): BucketFileVersion;
    /**
     * Returns true if the given object is an instance of BucketFileVersion.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is BucketFileVersion;
    /**
     * One of 'start', 'upload', 'hide', 'folder', or other values added in the future.
     */
    readonly action: pulumi.Output<string>;
    readonly bucketFileVersionId: pulumi.Output<string>;
    /**
     * The ID of the bucket. **Modifying this attribute will force creation of a new resource.**
     */
    readonly bucketId: pulumi.Output<string>;
    /**
     * MD5 sum of the content.
     */
    readonly contentMd5: pulumi.Output<string>;
    /**
     * SHA1 hash of the content.
     */
    readonly contentSha1: pulumi.Output<string>;
    /**
     * Content type. If not set, it will be set based on the file extension. **Modifying this attribute will force creation of a new resource.**
     */
    readonly contentType: pulumi.Output<string | undefined>;
    /**
     * The unique identifier for this version of this file.
     */
    readonly fileId: pulumi.Output<string>;
    /**
     * The custom information that is uploaded with the file. **Modifying this attribute will force creation of a new resource.**
     */
    readonly fileInfo: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * The name of the B2 file. **Modifying this attribute will force creation of a new resource.**
     */
    readonly fileName: pulumi.Output<string>;
    /**
     * Server-side encryption settings. **Modifying this attribute will force creation of a new resource.**
     */
    readonly serverSideEncryption: pulumi.Output<outputs.BucketFileVersionServerSideEncryption | undefined>;
    /**
     * The file size.
     */
    readonly size: pulumi.Output<number>;
    /**
     * Path to the local file. **Modifying this attribute will force creation of a new resource.**
     */
    readonly source: pulumi.Output<string>;
    /**
     * This is a UTC time when this file was uploaded.
     */
    readonly uploadTimestamp: pulumi.Output<number>;
    /**
     * Create a BucketFileVersion resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: BucketFileVersionArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering BucketFileVersion resources.
 */
export interface BucketFileVersionState {
    /**
     * One of 'start', 'upload', 'hide', 'folder', or other values added in the future.
     */
    action?: pulumi.Input<string>;
    bucketFileVersionId?: pulumi.Input<string>;
    /**
     * The ID of the bucket. **Modifying this attribute will force creation of a new resource.**
     */
    bucketId?: pulumi.Input<string>;
    /**
     * MD5 sum of the content.
     */
    contentMd5?: pulumi.Input<string>;
    /**
     * SHA1 hash of the content.
     */
    contentSha1?: pulumi.Input<string>;
    /**
     * Content type. If not set, it will be set based on the file extension. **Modifying this attribute will force creation of a new resource.**
     */
    contentType?: pulumi.Input<string>;
    /**
     * The unique identifier for this version of this file.
     */
    fileId?: pulumi.Input<string>;
    /**
     * The custom information that is uploaded with the file. **Modifying this attribute will force creation of a new resource.**
     */
    fileInfo?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The name of the B2 file. **Modifying this attribute will force creation of a new resource.**
     */
    fileName?: pulumi.Input<string>;
    /**
     * Server-side encryption settings. **Modifying this attribute will force creation of a new resource.**
     */
    serverSideEncryption?: pulumi.Input<inputs.BucketFileVersionServerSideEncryption>;
    /**
     * The file size.
     */
    size?: pulumi.Input<number>;
    /**
     * Path to the local file. **Modifying this attribute will force creation of a new resource.**
     */
    source?: pulumi.Input<string>;
    /**
     * This is a UTC time when this file was uploaded.
     */
    uploadTimestamp?: pulumi.Input<number>;
}
/**
 * The set of arguments for constructing a BucketFileVersion resource.
 */
export interface BucketFileVersionArgs {
    bucketFileVersionId?: pulumi.Input<string>;
    /**
     * The ID of the bucket. **Modifying this attribute will force creation of a new resource.**
     */
    bucketId: pulumi.Input<string>;
    /**
     * Content type. If not set, it will be set based on the file extension. **Modifying this attribute will force creation of a new resource.**
     */
    contentType?: pulumi.Input<string>;
    /**
     * The custom information that is uploaded with the file. **Modifying this attribute will force creation of a new resource.**
     */
    fileInfo?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The name of the B2 file. **Modifying this attribute will force creation of a new resource.**
     */
    fileName: pulumi.Input<string>;
    /**
     * Server-side encryption settings. **Modifying this attribute will force creation of a new resource.**
     */
    serverSideEncryption?: pulumi.Input<inputs.BucketFileVersionServerSideEncryption>;
    /**
     * Path to the local file. **Modifying this attribute will force creation of a new resource.**
     */
    source: pulumi.Input<string>;
}
