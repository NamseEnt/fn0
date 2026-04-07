import * as pulumi from "@pulumi/pulumi";
/**
 * Provides details about existing Network Manager sites.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.networkmanager.getSites({
 *     globalNetworkId: globalNetworkId,
 *     tags: {
 *         Env: "test",
 *     },
 * });
 * ```
 */
export declare function getSites(args: GetSitesArgs, opts?: pulumi.InvokeOptions): Promise<GetSitesResult>;
/**
 * A collection of arguments for invoking getSites.
 */
export interface GetSitesArgs {
    /**
     * ID of the Global Network of the sites to retrieve.
     */
    globalNetworkId: string;
    /**
     * Restricts the list to the sites with these tags.
     */
    tags?: {
        [key: string]: string;
    };
}
/**
 * A collection of values returned by getSites.
 */
export interface GetSitesResult {
    readonly globalNetworkId: string;
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    /**
     * IDs of the sites.
     */
    readonly ids: string[];
    readonly tags?: {
        [key: string]: string;
    };
}
/**
 * Provides details about existing Network Manager sites.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.networkmanager.getSites({
 *     globalNetworkId: globalNetworkId,
 *     tags: {
 *         Env: "test",
 *     },
 * });
 * ```
 */
export declare function getSitesOutput(args: GetSitesOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetSitesResult>;
/**
 * A collection of arguments for invoking getSites.
 */
export interface GetSitesOutputArgs {
    /**
     * ID of the Global Network of the sites to retrieve.
     */
    globalNetworkId: pulumi.Input<string>;
    /**
     * Restricts the list to the sites with these tags.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
