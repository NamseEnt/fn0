import * as pulumi from "@pulumi/pulumi";
/**
 * `aws.waf.IpSet` Retrieves a WAF IP Set Resource Id.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.waf.getIpset({
 *     name: "tfWAFIPSet",
 * });
 * ```
 */
export declare function getIpset(args: GetIpsetArgs, opts?: pulumi.InvokeOptions): Promise<GetIpsetResult>;
/**
 * A collection of arguments for invoking getIpset.
 */
export interface GetIpsetArgs {
    /**
     * Name of the WAF IP set.
     */
    name: string;
}
/**
 * A collection of values returned by getIpset.
 */
export interface GetIpsetResult {
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly name: string;
}
/**
 * `aws.waf.IpSet` Retrieves a WAF IP Set Resource Id.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.waf.getIpset({
 *     name: "tfWAFIPSet",
 * });
 * ```
 */
export declare function getIpsetOutput(args: GetIpsetOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetIpsetResult>;
/**
 * A collection of arguments for invoking getIpset.
 */
export interface GetIpsetOutputArgs {
    /**
     * Name of the WAF IP set.
     */
    name: pulumi.Input<string>;
}
