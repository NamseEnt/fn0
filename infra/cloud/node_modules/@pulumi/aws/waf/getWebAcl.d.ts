import * as pulumi from "@pulumi/pulumi";
/**
 * `aws.waf.WebAcl` Retrieves a WAF Web ACL Resource Id.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.waf.getWebAcl({
 *     name: "tfWAFWebACL",
 * });
 * ```
 */
export declare function getWebAcl(args: GetWebAclArgs, opts?: pulumi.InvokeOptions): Promise<GetWebAclResult>;
/**
 * A collection of arguments for invoking getWebAcl.
 */
export interface GetWebAclArgs {
    /**
     * Name of the WAF Web ACL.
     */
    name: string;
}
/**
 * A collection of values returned by getWebAcl.
 */
export interface GetWebAclResult {
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly name: string;
}
/**
 * `aws.waf.WebAcl` Retrieves a WAF Web ACL Resource Id.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.waf.getWebAcl({
 *     name: "tfWAFWebACL",
 * });
 * ```
 */
export declare function getWebAclOutput(args: GetWebAclOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetWebAclResult>;
/**
 * A collection of arguments for invoking getWebAcl.
 */
export interface GetWebAclOutputArgs {
    /**
     * Name of the WAF Web ACL.
     */
    name: pulumi.Input<string>;
}
