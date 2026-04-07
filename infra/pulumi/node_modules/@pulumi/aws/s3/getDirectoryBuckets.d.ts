import * as pulumi from "@pulumi/pulumi";
/**
 * Lists Amazon S3 Express directory buckets.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.s3.getDirectoryBuckets({});
 * ```
 */
export declare function getDirectoryBuckets(args?: GetDirectoryBucketsArgs, opts?: pulumi.InvokeOptions): Promise<GetDirectoryBucketsResult>;
/**
 * A collection of arguments for invoking getDirectoryBuckets.
 */
export interface GetDirectoryBucketsArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
}
/**
 * A collection of values returned by getDirectoryBuckets.
 */
export interface GetDirectoryBucketsResult {
    /**
     * Bucket ARNs.
     */
    readonly arns: string[];
    /**
     * Buckets names.
     */
    readonly buckets: string[];
    readonly id: string;
    readonly region: string;
}
/**
 * Lists Amazon S3 Express directory buckets.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.s3.getDirectoryBuckets({});
 * ```
 */
export declare function getDirectoryBucketsOutput(args?: GetDirectoryBucketsOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetDirectoryBucketsResult>;
/**
 * A collection of arguments for invoking getDirectoryBuckets.
 */
export interface GetDirectoryBucketsOutputArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
