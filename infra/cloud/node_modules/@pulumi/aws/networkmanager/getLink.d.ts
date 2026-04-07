import * as pulumi from "@pulumi/pulumi";
import * as outputs from "../types/output";
/**
 * Provides details about an existing Network Manager link.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.networkmanager.getLink({
 *     globalNetworkId: globalNetworkId,
 *     linkId: linkId,
 * });
 * ```
 */
export declare function getLink(args: GetLinkArgs, opts?: pulumi.InvokeOptions): Promise<GetLinkResult>;
/**
 * A collection of arguments for invoking getLink.
 */
export interface GetLinkArgs {
    /**
     * ID of the Global Network of the link to retrieve.
     */
    globalNetworkId: string;
    /**
     * ID of the specific link to retrieve.
     */
    linkId: string;
    /**
     * Key-value tags for the link.
     */
    tags?: {
        [key: string]: string;
    };
}
/**
 * A collection of values returned by getLink.
 */
export interface GetLinkResult {
    /**
     * ARN of the link.
     */
    readonly arn: string;
    /**
     * Upload speed and download speed of the link as documented below
     */
    readonly bandwidths: outputs.networkmanager.GetLinkBandwidth[];
    /**
     * Description of the link.
     */
    readonly description: string;
    readonly globalNetworkId: string;
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly linkId: string;
    /**
     * Provider of the link.
     */
    readonly providerName: string;
    /**
     * ID of the site.
     */
    readonly siteId: string;
    /**
     * Key-value tags for the link.
     */
    readonly tags: {
        [key: string]: string;
    };
    /**
     * Type of the link.
     */
    readonly type: string;
}
/**
 * Provides details about an existing Network Manager link.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.networkmanager.getLink({
 *     globalNetworkId: globalNetworkId,
 *     linkId: linkId,
 * });
 * ```
 */
export declare function getLinkOutput(args: GetLinkOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetLinkResult>;
/**
 * A collection of arguments for invoking getLink.
 */
export interface GetLinkOutputArgs {
    /**
     * ID of the Global Network of the link to retrieve.
     */
    globalNetworkId: pulumi.Input<string>;
    /**
     * ID of the specific link to retrieve.
     */
    linkId: pulumi.Input<string>;
    /**
     * Key-value tags for the link.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
