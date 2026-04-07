import * as pulumi from "@pulumi/pulumi";
/**
 * Manages a Network Manager Global Network.
 *
 * Use this resource to create and manage a global network, which is a single private network that acts as the high-level container for your network objects.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.networkmanager.GlobalNetwork("example", {description: "example"});
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_networkmanager_global_network` using the global network ID. For example:
 *
 * ```sh
 * $ pulumi import aws:networkmanager/globalNetwork:GlobalNetwork example global-network-0d47f6t230mz46dy4
 * ```
 */
export declare class GlobalNetwork extends pulumi.CustomResource {
    /**
     * Get an existing GlobalNetwork resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: GlobalNetworkState, opts?: pulumi.CustomResourceOptions): GlobalNetwork;
    /**
     * Returns true if the given object is an instance of GlobalNetwork.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is GlobalNetwork;
    /**
     * Global Network ARN.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Description of the Global Network.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * Key-value tags for the Global Network. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Create a GlobalNetwork resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: GlobalNetworkArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering GlobalNetwork resources.
 */
export interface GlobalNetworkState {
    /**
     * Global Network ARN.
     */
    arn?: pulumi.Input<string>;
    /**
     * Description of the Global Network.
     */
    description?: pulumi.Input<string>;
    /**
     * Key-value tags for the Global Network. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
}
/**
 * The set of arguments for constructing a GlobalNetwork resource.
 */
export interface GlobalNetworkArgs {
    /**
     * Description of the Global Network.
     */
    description?: pulumi.Input<string>;
    /**
     * Key-value tags for the Global Network. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
