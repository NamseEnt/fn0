import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a Route 53 Resolver DNSSEC config resource.
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
 * const exampleResolverDnsSecConfig = new aws.route53.ResolverDnsSecConfig("example", {resourceId: example.id});
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import  Route 53 Resolver DNSSEC configs using the Route 53 Resolver DNSSEC config ID. For example:
 *
 * ```sh
 * $ pulumi import aws:route53/resolverDnsSecConfig:ResolverDnsSecConfig example rdsc-be1866ecc1683e95
 * ```
 */
export declare class ResolverDnsSecConfig extends pulumi.CustomResource {
    /**
     * Get an existing ResolverDnsSecConfig resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ResolverDnsSecConfigState, opts?: pulumi.CustomResourceOptions): ResolverDnsSecConfig;
    /**
     * Returns true if the given object is an instance of ResolverDnsSecConfig.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ResolverDnsSecConfig;
    /**
     * The ARN for a configuration for DNSSEC validation.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The owner account ID of the virtual private cloud (VPC) for a configuration for DNSSEC validation.
     */
    readonly ownerId: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The ID of the virtual private cloud (VPC) that you're updating the DNSSEC validation status for.
     */
    readonly resourceId: pulumi.Output<string>;
    /**
     * The validation status for a DNSSEC configuration. The status can be one of the following: `ENABLING`, `ENABLED`, `DISABLING` and `DISABLED`.
     */
    readonly validationStatus: pulumi.Output<string>;
    /**
     * Create a ResolverDnsSecConfig resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ResolverDnsSecConfigArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ResolverDnsSecConfig resources.
 */
export interface ResolverDnsSecConfigState {
    /**
     * The ARN for a configuration for DNSSEC validation.
     */
    arn?: pulumi.Input<string>;
    /**
     * The owner account ID of the virtual private cloud (VPC) for a configuration for DNSSEC validation.
     */
    ownerId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The ID of the virtual private cloud (VPC) that you're updating the DNSSEC validation status for.
     */
    resourceId?: pulumi.Input<string>;
    /**
     * The validation status for a DNSSEC configuration. The status can be one of the following: `ENABLING`, `ENABLED`, `DISABLING` and `DISABLED`.
     */
    validationStatus?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a ResolverDnsSecConfig resource.
 */
export interface ResolverDnsSecConfigArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The ID of the virtual private cloud (VPC) that you're updating the DNSSEC validation status for.
     */
    resourceId: pulumi.Input<string>;
}
