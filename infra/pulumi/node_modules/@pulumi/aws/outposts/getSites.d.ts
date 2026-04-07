import * as pulumi from "@pulumi/pulumi";
/**
 * Provides details about multiple Outposts Sites.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const all = aws.outposts.getSites({});
 * ```
 */
export declare function getSites(args?: GetSitesArgs, opts?: pulumi.InvokeOptions): Promise<GetSitesResult>;
/**
 * A collection of arguments for invoking getSites.
 */
export interface GetSitesArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
}
/**
 * A collection of values returned by getSites.
 */
export interface GetSitesResult {
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    /**
     * Set of Outposts Site identifiers.
     */
    readonly ids: string[];
    readonly region: string;
}
/**
 * Provides details about multiple Outposts Sites.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const all = aws.outposts.getSites({});
 * ```
 */
export declare function getSitesOutput(args?: GetSitesOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetSitesResult>;
/**
 * A collection of arguments for invoking getSites.
 */
export interface GetSitesOutputArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
