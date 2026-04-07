import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a Route 53 Resolver config resource.
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
 * const exampleResolverConfig = new aws.route53.ResolverConfig("example", {
 *     resourceId: example.id,
 *     autodefinedReverseFlag: "DISABLE",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Route 53 Resolver configs using the Route 53 Resolver config ID. For example:
 *
 * ```sh
 * $ pulumi import aws:route53/resolverConfig:ResolverConfig example rslvr-rc-715aa20c73a23da7
 * ```
 */
export declare class ResolverConfig extends pulumi.CustomResource {
    /**
     * Get an existing ResolverConfig resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ResolverConfigState, opts?: pulumi.CustomResourceOptions): ResolverConfig;
    /**
     * Returns true if the given object is an instance of ResolverConfig.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ResolverConfig;
    /**
     * Indicates whether or not the Resolver will create autodefined rules for reverse DNS lookups. Valid values: `ENABLE`, `DISABLE`.
     */
    readonly autodefinedReverseFlag: pulumi.Output<string>;
    /**
     * The AWS account ID of the owner of the VPC that this resolver configuration applies to.
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
     * Create a ResolverConfig resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ResolverConfigArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ResolverConfig resources.
 */
export interface ResolverConfigState {
    /**
     * Indicates whether or not the Resolver will create autodefined rules for reverse DNS lookups. Valid values: `ENABLE`, `DISABLE`.
     */
    autodefinedReverseFlag?: pulumi.Input<string>;
    /**
     * The AWS account ID of the owner of the VPC that this resolver configuration applies to.
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
 * The set of arguments for constructing a ResolverConfig resource.
 */
export interface ResolverConfigArgs {
    /**
     * Indicates whether or not the Resolver will create autodefined rules for reverse DNS lookups. Valid values: `ENABLE`, `DISABLE`.
     */
    autodefinedReverseFlag: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The ID of the VPC that the configuration is for.
     */
    resourceId: pulumi.Input<string>;
}
