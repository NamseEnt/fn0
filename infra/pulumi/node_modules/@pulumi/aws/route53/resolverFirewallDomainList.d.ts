import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a Route 53 Resolver DNS Firewall domain list resource.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.route53.ResolverFirewallDomainList("example", {name: "example"});
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import  Route 53 Resolver DNS Firewall domain lists using the Route 53 Resolver DNS Firewall domain list ID. For example:
 *
 * ```sh
 * $ pulumi import aws:route53/resolverFirewallDomainList:ResolverFirewallDomainList example rslvr-fdl-0123456789abcdef
 * ```
 */
export declare class ResolverFirewallDomainList extends pulumi.CustomResource {
    /**
     * Get an existing ResolverFirewallDomainList resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ResolverFirewallDomainListState, opts?: pulumi.CustomResourceOptions): ResolverFirewallDomainList;
    /**
     * Returns true if the given object is an instance of ResolverFirewallDomainList.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ResolverFirewallDomainList;
    /**
     * The ARN (Amazon Resource Name) of the domain list.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * A array of domains for the firewall domain list.
     */
    readonly domains: pulumi.Output<string[] | undefined>;
    /**
     * A name that lets you identify the domain list, to manage and use it.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * A map of tags to assign to the resource. f configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * A map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * Create a ResolverFirewallDomainList resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: ResolverFirewallDomainListArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ResolverFirewallDomainList resources.
 */
export interface ResolverFirewallDomainListState {
    /**
     * The ARN (Amazon Resource Name) of the domain list.
     */
    arn?: pulumi.Input<string>;
    /**
     * A array of domains for the firewall domain list.
     */
    domains?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * A name that lets you identify the domain list, to manage and use it.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. f configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * A map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
/**
 * The set of arguments for constructing a ResolverFirewallDomainList resource.
 */
export interface ResolverFirewallDomainListArgs {
    /**
     * A array of domains for the firewall domain list.
     */
    domains?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * A name that lets you identify the domain list, to manage and use it.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. f configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
