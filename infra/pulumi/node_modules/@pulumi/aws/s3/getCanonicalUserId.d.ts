import * as pulumi from "@pulumi/pulumi";
/**
 * The Canonical User ID data source allows access to the [canonical user ID](http://docs.aws.amazon.com/general/latest/gr/acct-identifiers.html)
 * for the effective account in which this provider is working.
 *
 * > **NOTE:** To use this data source, you must have the `s3:ListAllMyBuckets` permission.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const current = aws.s3.getCanonicalUserId({});
 * export const canonicalUserId = current.then(current => current.id);
 * ```
 */
export declare function getCanonicalUserId(opts?: pulumi.InvokeOptions): Promise<GetCanonicalUserIdResult>;
/**
 * A collection of values returned by getCanonicalUserId.
 */
export interface GetCanonicalUserIdResult {
    /**
     * Human-friendly name linked to the canonical user ID. The bucket owner's display name. **NOTE:** [This value](https://docs.aws.amazon.com/AmazonS3/latest/API/RESTServiceGET.html) is only included in the response in the US East (N. Virginia), US West (N. California), US West (Oregon), Asia Pacific (Singapore), Asia Pacific (Sydney), Asia Pacific (Tokyo), EU (Ireland), and South America (São Paulo) regions.
     */
    readonly displayName: string;
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
}
/**
 * The Canonical User ID data source allows access to the [canonical user ID](http://docs.aws.amazon.com/general/latest/gr/acct-identifiers.html)
 * for the effective account in which this provider is working.
 *
 * > **NOTE:** To use this data source, you must have the `s3:ListAllMyBuckets` permission.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const current = aws.s3.getCanonicalUserId({});
 * export const canonicalUserId = current.then(current => current.id);
 * ```
 */
export declare function getCanonicalUserIdOutput(opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetCanonicalUserIdResult>;
