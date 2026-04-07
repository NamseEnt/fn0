import * as pulumi from "@pulumi/pulumi";
import * as outputs from "./types/output";
export declare function getBucketFile(args: GetBucketFileArgs, opts?: pulumi.InvokeOptions): Promise<GetBucketFileResult>;
/**
 * A collection of arguments for invoking getBucketFile.
 */
export interface GetBucketFileArgs {
    bucketId: string;
    fileName: string;
    id?: string;
    showVersions?: boolean;
}
/**
 * A collection of values returned by getBucketFile.
 */
export interface GetBucketFileResult {
    readonly bucketId: string;
    readonly fileName: string;
    readonly fileVersions: outputs.GetBucketFileFileVersion[];
    readonly id: string;
    readonly showVersions?: boolean;
}
export declare function getBucketFileOutput(args: GetBucketFileOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetBucketFileResult>;
/**
 * A collection of arguments for invoking getBucketFile.
 */
export interface GetBucketFileOutputArgs {
    bucketId: pulumi.Input<string>;
    fileName: pulumi.Input<string>;
    id?: pulumi.Input<string>;
    showVersions?: pulumi.Input<boolean>;
}
