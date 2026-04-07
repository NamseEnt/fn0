import * as pulumi from "@pulumi/pulumi";
/**
 * `aws.route53.ResolverFirewallConfig` provides details about a specific a Route 53 Resolver DNS Firewall config.
 *
 * This data source allows to find a details about a specific a Route 53 Resolver DNS Firewall config.
 *
 * ## Example Usage
 *
 * The following example shows how to get a firewall config using the VPC ID.
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.route53.getResolverFirewallConfig({
 *     resourceId: "vpc-exampleid",
 * });
 * ```
 */
export declare function getResolverFirewallConfig(args: GetResolverFirewallConfigArgs, opts?: pulumi.InvokeOptions): Promise<GetResolverFirewallConfigResult>;
/**
 * A collection of arguments for invoking getResolverFirewallConfig.
 */
export interface GetResolverFirewallConfigArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
    /**
     * The ID of the VPC from Amazon VPC that the configuration is for.
     */
    resourceId: string;
}
/**
 * A collection of values returned by getResolverFirewallConfig.
 */
export interface GetResolverFirewallConfigResult {
    /**
     * Determines how DNS Firewall operates during failures, for example when all traffic that is sent to DNS Firewall fails to receive a reply.
     */
    readonly firewallFailOpen: string;
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    /**
     * The Amazon Web Services account ID of the owner of the VPC that this firewall configuration applies to.
     */
    readonly ownerId: string;
    readonly region: string;
    readonly resourceId: string;
}
/**
 * `aws.route53.ResolverFirewallConfig` provides details about a specific a Route 53 Resolver DNS Firewall config.
 *
 * This data source allows to find a details about a specific a Route 53 Resolver DNS Firewall config.
 *
 * ## Example Usage
 *
 * The following example shows how to get a firewall config using the VPC ID.
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.route53.getResolverFirewallConfig({
 *     resourceId: "vpc-exampleid",
 * });
 * ```
 */
export declare function getResolverFirewallConfigOutput(args: GetResolverFirewallConfigOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetResolverFirewallConfigResult>;
/**
 * A collection of arguments for invoking getResolverFirewallConfig.
 */
export interface GetResolverFirewallConfigOutputArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The ID of the VPC from Amazon VPC that the configuration is for.
     */
    resourceId: pulumi.Input<string>;
}
