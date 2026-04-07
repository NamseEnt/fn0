import * as pulumi from "@pulumi/pulumi";
/**
 * `aws.wafregional.IpSet` Retrieves a WAF Regional IP Set Resource Id.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.wafregional.getIpset({
 *     name: "tfWAFRegionalIPSet",
 * });
 * ```
 */
export declare function getIpset(args: GetIpsetArgs, opts?: pulumi.InvokeOptions): Promise<GetIpsetResult>;
/**
 * A collection of arguments for invoking getIpset.
 */
export interface GetIpsetArgs {
    /**
     * Name of the WAF Regional IP set.
     */
    name: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
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
    readonly region: string;
}
/**
 * `aws.wafregional.IpSet` Retrieves a WAF Regional IP Set Resource Id.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.wafregional.getIpset({
 *     name: "tfWAFRegionalIPSet",
 * });
 * ```
 */
export declare function getIpsetOutput(args: GetIpsetOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetIpsetResult>;
/**
 * A collection of arguments for invoking getIpset.
 */
export interface GetIpsetOutputArgs {
    /**
     * Name of the WAF Regional IP set.
     */
    name: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
