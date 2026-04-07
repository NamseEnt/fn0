import * as pulumi from "@pulumi/pulumi";
import * as outputs from "./types/output";
export declare function getBucketFiles(args: GetBucketFilesArgs, opts?: pulumi.InvokeOptions): Promise<GetBucketFilesResult>;
/**
 * A collection of arguments for invoking getBucketFiles.
 */
export interface GetBucketFilesArgs {
    bucketId: string;
    folderName?: string;
    id?: string;
    recursive?: boolean;
    showVersions?: boolean;
}
/**
 * A collection of values returned by getBucketFiles.
 */
export interface GetBucketFilesResult {
    readonly bucketId: string;
    readonly fileVersions: outputs.GetBucketFilesFileVersion[];
    readonly folderName?: string;
    readonly id: string;
    readonly recursive?: boolean;
    readonly showVersions?: boolean;
}
export declare function getBucketFilesOutput(args: GetBucketFilesOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetBucketFilesResult>;
/**
 * A collection of arguments for invoking getBucketFiles.
 */
export interface GetBucketFilesOutputArgs {
    bucketId: pulumi.Input<string>;
    folderName?: pulumi.Input<string>;
    id?: pulumi.Input<string>;
    recursive?: pulumi.Input<boolean>;
    showVersions?: pulumi.Input<boolean>;
}
