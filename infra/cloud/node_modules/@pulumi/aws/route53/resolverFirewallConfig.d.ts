import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a Route 53 Resolver DNS Firewall config resource.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.ec2.Vpc("example", {
 *     cidrBlock: "10.0.0.0/16",
 *     enableDnsSupport: true,
 *     enableDnsHostnames: true,
 * });
 * const exampleResolverFirewallConfig = new aws.route53.ResolverFirewallConfig("example", {
 *     resourceId: example.id,
 *     firewallFailOpen: "ENABLED",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Route 53 Resolver DNS Firewall configs using the Route 53 Resolver DNS Firewall config ID. For example:
 *
 * ```sh
 * $ pulumi import aws:route53/resolverFirewallConfig:ResolverFirewallConfig example rdsc-be1866ecc1683e95
 * ```
 */
export declare class ResolverFirewallConfig extends pulumi.CustomResource {
    /**
     * Get an existing ResolverFirewallConfig resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ResolverFirewallConfigState, opts?: pulumi.CustomResourceOptions): ResolverFirewallConfig;
    /**
     * Returns true if the given object is an instance of ResolverFirewallConfig.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ResolverFirewallConfig;
    /**
     * Determines how Route 53 Resolver handles queries during failures, for example when all traffic that is sent to DNS Firewall fails to receive a reply. By default, fail open is disabled, which means the failure mode is closed. This approach favors security over availability. DNS Firewall blocks queries that it is unable to evaluate properly. If you enable this option, the failure mode is open. This approach favors availability over security. DNS Firewall allows queries to proceed if it is unable to properly evaluate them. Valid values: `ENABLED`, `DISABLED`.
     */
    readonly firewallFailOpen: pulumi.Output<string>;
    /**
     * The AWS account ID of the owner of the VPC that this firewall configuration applies to.
     */
    readonly ownerId: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The ID of the VPC that the configuration is for.
     */
    readonly resourceId: pulumi.Output<string>;
    /**
     * Create a ResolverFirewallConfig resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ResolverFirewallConfigArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ResolverFirewallConfig resources.
 */
export interface ResolverFirewallConfigState {
    /**
     * Determines how Route 53 Resolver handles queries during failures, for example when all traffic that is sent to DNS Firewall fails to receive a reply. By default, fail open is disabled, which means the failure mode is closed. This approach favors security over availability. DNS Firewall blocks queries that it is unable to evaluate properly. If you enable this option, the failure mode is open. This approach favors availability over security. DNS Firewall allows queries to proceed if it is unable to properly evaluate them. Valid values: `ENABLED`, `DISABLED`.
     */
    firewallFailOpen?: pulumi.Input<string>;
    /**
     * The AWS account ID of the owner of the VPC that this firewall configuration applies to.
     */
    ownerId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The ID of the VPC that the configuration is for.
     */
    resourceId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a ResolverFirewallConfig resource.
 */
export interface ResolverFirewallConfigArgs {
    /**
     * Determines how Route 53 Resolver handles queries during failures, for example when all traffic that is sent to DNS Firewall fails to receive a reply. By default, fail open is disabled, which means the failure mode is closed. This approach favors security over availability. DNS Firewall blocks queries that it is unable to evaluate properly. If you enable this option, the failure mode is open. This approach favors availability over security. DNS Firewall allows queries to proceed if it is unable to properly evaluate them. Valid values: `ENABLED`, `DISABLED`.
     */
    firewallFailOpen?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The ID of the VPC that the configuration is for.
     */
    resourceId: pulumi.Input<string>;
}
