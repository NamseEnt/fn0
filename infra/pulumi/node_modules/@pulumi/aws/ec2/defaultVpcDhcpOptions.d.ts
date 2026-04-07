import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a resource to manage the [default AWS DHCP Options Set](http://docs.aws.amazon.com/AmazonVPC/latest/UserGuide/VPC_DHCP_Options.html#AmazonDNS)
 * in the current region.
 *
 * Each AWS region comes with a default set of DHCP options.
 * **This is an advanced resource**, and has special caveats to be aware of when
 * using it. Please read this document in its entirety before using this resource.
 *
 * The `aws.ec2.DefaultVpcDhcpOptions` behaves differently from normal resources, in that
 * this provider does not _create_ this resource, but instead "adopts" it
 * into management.
 *
 * ## Example Usage
 *
 * Basic usage with tags:
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const _default = new aws.ec2.DefaultVpcDhcpOptions("default", {tags: {
 *     Name: "Default DHCP Option Set",
 * }});
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import VPC DHCP Options using the DHCP Options `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:ec2/defaultVpcDhcpOptions:DefaultVpcDhcpOptions default_options dopt-d9070ebb
 * ```
 */
export declare class DefaultVpcDhcpOptions extends pulumi.CustomResource {
    /**
     * Get an existing DefaultVpcDhcpOptions resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: DefaultVpcDhcpOptionsState, opts?: pulumi.CustomResourceOptions): DefaultVpcDhcpOptions;
    /**
     * Returns true if the given object is an instance of DefaultVpcDhcpOptions.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is DefaultVpcDhcpOptions;
    /**
     * The ARN of the DHCP Options Set.
     */
    readonly arn: pulumi.Output<string>;
    readonly domainName: pulumi.Output<string>;
    readonly domainNameServers: pulumi.Output<string>;
    readonly ipv6AddressPreferredLeaseTime: pulumi.Output<string>;
    /**
     * List of NETBIOS name servers.
     */
    readonly netbiosNameServers: pulumi.Output<string>;
    /**
     * The NetBIOS node type (1, 2, 4, or 8). AWS recommends to specify 2 since broadcast and multicast are not supported in their network. For more information about these node types, see [RFC 2132](http://www.ietf.org/rfc/rfc2132.txt).
     */
    readonly netbiosNodeType: pulumi.Output<string>;
    readonly ntpServers: pulumi.Output<string>;
    /**
     * The ID of the AWS account that owns the DHCP options set.
     */
    readonly ownerId: pulumi.Output<string>;
    readonly region: pulumi.Output<string>;
    /**
     * A map of tags to assign to the resource.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * Create a DefaultVpcDhcpOptions resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: DefaultVpcDhcpOptionsArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering DefaultVpcDhcpOptions resources.
 */
export interface DefaultVpcDhcpOptionsState {
    /**
     * The ARN of the DHCP Options Set.
     */
    arn?: pulumi.Input<string>;
    domainName?: pulumi.Input<string>;
    domainNameServers?: pulumi.Input<string>;
    ipv6AddressPreferredLeaseTime?: pulumi.Input<string>;
    /**
     * List of NETBIOS name servers.
     */
    netbiosNameServers?: pulumi.Input<string>;
    /**
     * The NetBIOS node type (1, 2, 4, or 8). AWS recommends to specify 2 since broadcast and multicast are not supported in their network. For more information about these node types, see [RFC 2132](http://www.ietf.org/rfc/rfc2132.txt).
     */
    netbiosNodeType?: pulumi.Input<string>;
    ntpServers?: pulumi.Input<string>;
    /**
     * The ID of the AWS account that owns the DHCP options set.
     */
    ownerId?: pulumi.Input<string>;
    region?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
/**
 * The set of arguments for constructing a DefaultVpcDhcpOptions resource.
 */
export interface DefaultVpcDhcpOptionsArgs {
    /**
     * The ID of the AWS account that owns the DHCP options set.
     */
    ownerId?: pulumi.Input<string>;
    region?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
