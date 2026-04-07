import * as pulumi from "@pulumi/pulumi";
/**
 * Provides details about existing Network Manager links.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.networkmanager.getLinks({
 *     globalNetworkId: globalNetworkId,
 *     tags: {
 *         Env: "test",
 *     },
 * });
 * ```
 */
export declare function getLinks(args: GetLinksArgs, opts?: pulumi.InvokeOptions): Promise<GetLinksResult>;
/**
 * A collection of arguments for invoking getLinks.
 */
export interface GetLinksArgs {
    /**
     * ID of the Global Network of the links to retrieve.
     */
    globalNetworkId: string;
    /**
     * Link provider to retrieve.
     */
    providerName?: string;
    /**
     * ID of the site of the links to retrieve.
     */
    siteId?: string;
    /**
     * Restricts the list to the links with these tags.
     */
    tags?: {
        [key: string]: string;
    };
    /**
     * Link type to retrieve.
     */
    type?: string;
}
/**
 * A collection of values returned by getLinks.
 */
export interface GetLinksResult {
    readonly globalNetworkId: string;
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    /**
     * IDs of the links.
     */
    readonly ids: string[];
    readonly providerName?: string;
    readonly siteId?: string;
    readonly tags?: {
        [key: string]: string;
    };
    readonly type?: string;
}
/**
 * Provides details about existing Network Manager links.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.networkmanager.getLinks({
 *     globalNetworkId: globalNetworkId,
 *     tags: {
 *         Env: "test",
 *     },
 * });
 * ```
 */
export declare function getLinksOutput(args: GetLinksOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetLinksResult>;
/**
 * A collection of arguments for invoking getLinks.
 */
export interface GetLinksOutputArgs {
    /**
     * ID of the Global Network of the links to retrieve.
     */
    globalNetworkId: pulumi.Input<string>;
    /**
     * Link provider to retrieve.
     */
    providerName?: pulumi.Input<string>;
    /**
     * ID of the site of the links to retrieve.
     */
    siteId?: pulumi.Input<string>;
    /**
     * Restricts the list to the links with these tags.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Link type to retrieve.
     */
    type?: pulumi.Input<string>;
}
