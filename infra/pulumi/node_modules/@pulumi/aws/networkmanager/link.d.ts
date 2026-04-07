import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages a Network Manager link. Use this resource to create a link for a site.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.networkmanager.Link("example", {
 *     globalNetworkId: exampleAwsNetworkmanagerGlobalNetwork.id,
 *     siteId: exampleAwsNetworkmanagerSite.id,
 *     bandwidth: {
 *         uploadSpeed: 10,
 *         downloadSpeed: 50,
 *     },
 *     providerName: "MegaCorp",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_networkmanager_link` using the link ARN. For example:
 *
 * ```sh
 * $ pulumi import aws:networkmanager/link:Link example arn:aws:networkmanager::123456789012:link/global-network-0d47f6t230mz46dy4/link-444555aaabbb11223
 * ```
 */
export declare class Link extends pulumi.CustomResource {
    /**
     * Get an existing Link resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: LinkState, opts?: pulumi.CustomResourceOptions): Link;
    /**
     * Returns true if the given object is an instance of Link.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Link;
    /**
     * Link ARN.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Upload speed and download speed in Mbps. See below.
     */
    readonly bandwidth: pulumi.Output<outputs.networkmanager.LinkBandwidth>;
    /**
     * Description of the link.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * ID of the global network.
     */
    readonly globalNetworkId: pulumi.Output<string>;
    /**
     * Provider of the link.
     */
    readonly providerName: pulumi.Output<string | undefined>;
    /**
     * ID of the site.
     *
     * The following arguments are optional:
     */
    readonly siteId: pulumi.Output<string>;
    /**
     * Key-value tags for the link. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * Map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * Type of the link.
     */
    readonly type: pulumi.Output<string | undefined>;
    /**
     * Create a Link resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: LinkArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Link resources.
 */
export interface LinkState {
    /**
     * Link ARN.
     */
    arn?: pulumi.Input<string>;
    /**
     * Upload speed and download speed in Mbps. See below.
     */
    bandwidth?: pulumi.Input<inputs.networkmanager.LinkBandwidth>;
    /**
     * Description of the link.
     */
    description?: pulumi.Input<string>;
    /**
     * ID of the global network.
     */
    globalNetworkId?: pulumi.Input<string>;
    /**
     * Provider of the link.
     */
    providerName?: pulumi.Input<string>;
    /**
     * ID of the site.
     *
     * The following arguments are optional:
     */
    siteId?: pulumi.Input<string>;
    /**
     * Key-value tags for the link. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Type of the link.
     */
    type?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a Link resource.
 */
export interface LinkArgs {
    /**
     * Upload speed and download speed in Mbps. See below.
     */
    bandwidth: pulumi.Input<inputs.networkmanager.LinkBandwidth>;
    /**
     * Description of the link.
     */
    description?: pulumi.Input<string>;
    /**
     * ID of the global network.
     */
    globalNetworkId: pulumi.Input<string>;
    /**
     * Provider of the link.
     */
    providerName?: pulumi.Input<string>;
    /**
     * ID of the site.
     *
     * The following arguments are optional:
     */
    siteId: pulumi.Input<string>;
    /**
     * Key-value tags for the link. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Type of the link.
     */
    type?: pulumi.Input<string>;
}
