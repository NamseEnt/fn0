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
 * const example = aws.cloudfront.getResponseHeadersPolicy({
 *     name: "example-policy",
 * });
 * ```
 *
 * ### AWS-Managed Policies
 *
 * AWS managed response header policy names are prefixed with `Managed-`:
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.cloudfront.getResponseHeadersPolicy({
 *     name: "Managed-SimpleCORS",
 * });
 * ```
 */
export declare function getResponseHeadersPolicy(args?: GetResponseHeadersPolicyArgs, opts?: pulumi.InvokeOptions): Promise<GetResponseHeadersPolicyResult>;
/**
 * A collection of arguments for invoking getResponseHeadersPolicy.
 */
export interface GetResponseHeadersPolicyArgs {
    /**
     * Identifier for the response headers policy.
     */
    id?: string;
    /**
     * Unique name to identify the response headers policy.
     */
    name?: string;
}
/**
 * A collection of values returned by getResponseHeadersPolicy.
 */
export interface GetResponseHeadersPolicyResult {
    /**
     * The response headers policy ARN.
     */
    readonly arn: string;
    /**
     * Comment to describe the response headers policy. The comment cannot be longer than 128 characters.
     */
    readonly comment: string;
    /**
     * Configuration for a set of HTTP response headers that are used for Cross-Origin Resource Sharing (CORS). See Cors Config for more information.
     */
    readonly corsConfigs: outputs.cloudfront.GetResponseHeadersPolicyCorsConfig[];
    /**
     * Object that contains an attribute `items` that contains a list of Custom Headers. See Custom Header for more information.
     */
    readonly customHeadersConfigs: outputs.cloudfront.GetResponseHeadersPolicyCustomHeadersConfig[];
    /**
     * Current version of the response headers policy.
     */
    readonly etag: string;
    readonly id: string;
    readonly name: string;
    /**
     * Object that contains an attribute `items` that contains a list of Remove Headers. See Remove Header for more information.
     */
    readonly removeHeadersConfigs: outputs.cloudfront.GetResponseHeadersPolicyRemoveHeadersConfig[];
    /**
     * A configuration for a set of security-related HTTP response headers. See Security Headers Config for more information.
     */
    readonly securityHeadersConfigs: outputs.cloudfront.GetResponseHeadersPolicySecurityHeadersConfig[];
    /**
     * (Optional) Configuration for enabling the Server-Timing header in HTTP responses sent from CloudFront. See Server Timing Headers Config for more information.
     */
    readonly serverTimingHeadersConfigs: outputs.cloudfront.GetResponseHeadersPolicyServerTimingHeadersConfig[];
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
 * const example = aws.cloudfront.getResponseHeadersPolicy({
 *     name: "example-policy",
 * });
 * ```
 *
 * ### AWS-Managed Policies
 *
 * AWS managed response header policy names are prefixed with `Managed-`:
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.cloudfront.getResponseHeadersPolicy({
 *     name: "Managed-SimpleCORS",
 * });
 * ```
 */
export declare function getResponseHeadersPolicyOutput(args?: GetResponseHeadersPolicyOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetResponseHeadersPolicyResult>;
/**
 * A collection of arguments for invoking getResponseHeadersPolicy.
 */
export interface GetResponseHeadersPolicyOutputArgs {
    /**
     * Identifier for the response headers policy.
     */
    id?: pulumi.Input<string>;
    /**
     * Unique name to identify the response headers policy.
     */
    name?: pulumi.Input<string>;
}
