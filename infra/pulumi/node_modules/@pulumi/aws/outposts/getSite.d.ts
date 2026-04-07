import * as pulumi from "@pulumi/pulumi";
/**
 * Provides details about an Outposts Site.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.outposts.getSite({
 *     name: "example",
 * });
 * ```
 */
export declare function getSite(args?: GetSiteArgs, opts?: pulumi.InvokeOptions): Promise<GetSiteResult>;
/**
 * A collection of arguments for invoking getSite.
 */
export interface GetSiteArgs {
    /**
     * Identifier of the Site.
     */
    id?: string;
    /**
     * Name of the Site.
     */
    name?: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
}
/**
 * A collection of values returned by getSite.
 */
export interface GetSiteResult {
    /**
     * AWS Account identifier.
     */
    readonly accountId: string;
    /**
     * Description.
     */
    readonly description: string;
    readonly id: string;
    readonly name: string;
    readonly region: string;
}
/**
 * Provides details about an Outposts Site.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.outposts.getSite({
 *     name: "example",
 * });
 * ```
 */
export declare function getSiteOutput(args?: GetSiteOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetSiteResult>;
/**
 * A collection of arguments for invoking getSite.
 */
export interface GetSiteOutputArgs {
    /**
     * Identifier of the Site.
     */
    id?: pulumi.Input<string>;
    /**
     * Name of the Site.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
