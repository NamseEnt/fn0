import * as pulumi from "@pulumi/pulumi";
import * as outputs from "./types/output";
export declare function getBucket(args: GetBucketArgs, opts?: pulumi.InvokeOptions): Promise<GetBucketResult>;
/**
 * A collection of arguments for invoking getBucket.
 */
export interface GetBucketArgs {
    bucketName: string;
    id?: string;
}
/**
 * A collection of values returned by getBucket.
 */
export interface GetBucketResult {
    readonly accountId: string;
    readonly bucketId: string;
    readonly bucketInfo: {
        [key: string]: string;
    };
    readonly bucketName: string;
    readonly bucketType: string;
    readonly corsRules: outputs.GetBucketCorsRule[];
    readonly defaultServerSideEncryptions: outputs.GetBucketDefaultServerSideEncryption[];
    readonly fileLockConfigurations: outputs.GetBucketFileLockConfiguration[];
    readonly id: string;
    readonly lifecycleRules: outputs.GetBucketLifecycleRule[];
    readonly options: string[];
    readonly revision: number;
}
export declare function getBucketOutput(args: GetBucketOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetBucketResult>;
/**
 * A collection of arguments for invoking getBucket.
 */
export interface GetBucketOutputArgs {
    bucketName: pulumi.Input<string>;
    id?: pulumi.Input<string>;
}
