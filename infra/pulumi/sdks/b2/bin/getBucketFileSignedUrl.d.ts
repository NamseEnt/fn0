import * as pulumi from "@pulumi/pulumi";
export declare function getBucketFileSignedUrl(args: GetBucketFileSignedUrlArgs, opts?: pulumi.InvokeOptions): Promise<GetBucketFileSignedUrlResult>;
/**
 * A collection of arguments for invoking getBucketFileSignedUrl.
 */
export interface GetBucketFileSignedUrlArgs {
    bucketId: string;
    duration?: number;
    fileName: string;
    id?: string;
}
/**
 * A collection of values returned by getBucketFileSignedUrl.
 */
export interface GetBucketFileSignedUrlResult {
    readonly bucketId: string;
    readonly duration?: number;
    readonly fileName: string;
    readonly id: string;
    readonly signedUrl: string;
}
export declare function getBucketFileSignedUrlOutput(args: GetBucketFileSignedUrlOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetBucketFileSignedUrlResult>;
/**
 * A collection of arguments for invoking getBucketFileSignedUrl.
 */
export interface GetBucketFileSignedUrlOutputArgs {
    bucketId: pulumi.Input<string>;
    duration?: pulumi.Input<number>;
    fileName: pulumi.Input<string>;
    id?: pulumi.Input<string>;
}
