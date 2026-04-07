import * as pulumi from "@pulumi/pulumi";
import * as outputs from "../types/output";
/**
 * Use this data source to retrieve information about a CloudFront cache policy.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.cloudfront.getCachePolicy({
 *     name: "example-policy",
 * });
 * ```
 *
 * ### AWS-Managed Policies
 *
 * AWS managed cache policy names are prefixed with `Managed-`, except for `UseOriginCacheControlHeaders` and `UseOriginCacheControlHeaders-QueryStrings`:
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example1 = aws.cloudfront.getCachePolicy({
 *     name: "Managed-CachingOptimized",
 * });
 * const example2 = aws.cloudfront.getCachePolicy({
 *     name: "UseOriginCacheControlHeaders",
 * });
 * ```
 */
export declare function getCachePolicy(args?: GetCachePolicyArgs, opts?: pulumi.InvokeOptions): Promise<GetCachePolicyResult>;
/**
 * A collection of arguments for invoking getCachePolicy.
 */
export interface GetCachePolicyArgs {
    /**
     * Identifier for the cache policy.
     */
    id?: string;
    /**
     * Unique name to identify the cache policy.
     */
    name?: string;
}
/**
 * A collection of values returned by getCachePolicy.
 */
export interface GetCachePolicyResult {
    /**
     * The cache policy ARN.
     */
    readonly arn: string;
    /**
     * Comment to describe the cache policy.
     */
    readonly comment: string;
    /**
     * Default amount of time, in seconds, that you want objects to stay in the CloudFront cache before CloudFront sends another request to the origin to see if the object has been updated.
     */
    readonly defaultTtl: number;
    /**
     * Current version of the cache policy.
     */
    readonly etag: string;
    readonly id?: string;
    /**
     * Maximum amount of time, in seconds, that objects stay in the CloudFront cache before CloudFront sends another request to the origin to see if the object has been updated.
     */
    readonly maxTtl: number;
    /**
     * Minimum amount of time, in seconds, that you want objects to stay in the CloudFront cache before CloudFront sends another request to the origin to see if the object has been updated.
     */
    readonly minTtl: number;
    readonly name?: string;
    /**
     * The HTTP headers, cookies, and URL query strings to include in the cache key. See Parameters In Cache Key And Forwarded To Origin for more information.
     */
    readonly parametersInCacheKeyAndForwardedToOrigins: outputs.cloudfront.GetCachePolicyParametersInCacheKeyAndForwardedToOrigin[];
}
/**
 * Use this data source to retrieve information about a CloudFront cache policy.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.cloudfront.getCachePolicy({
 *     name: "example-policy",
 * });
 * ```
 *
 * ### AWS-Managed Policies
 *
 * AWS managed cache policy names are prefixed with `Managed-`, except for `UseOriginCacheControlHeaders` and `UseOriginCacheControlHeaders-QueryStrings`:
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example1 = aws.cloudfront.getCachePolicy({
 *     name: "Managed-CachingOptimized",
 * });
 * const example2 = aws.cloudfront.getCachePolicy({
 *     name: "UseOriginCacheControlHeaders",
 * });
 * ```
 */
export declare function getCachePolicyOutput(args?: GetCachePolicyOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetCachePolicyResult>;
/**
 * A collection of arguments for invoking getCachePolicy.
 */
export interface GetCachePolicyOutputArgs {
    /**
     * Identifier for the cache policy.
     */
    id?: pulumi.Input<string>;
    /**
     * Unique name to identify the cache policy.
     */
    name?: pulumi.Input<string>;
}
