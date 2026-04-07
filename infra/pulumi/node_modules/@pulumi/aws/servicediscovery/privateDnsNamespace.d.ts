import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a Service Discovery Private DNS Namespace resource.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.ec2.Vpc("example", {cidrBlock: "10.0.0.0/16"});
 * const examplePrivateDnsNamespace = new aws.servicediscovery.PrivateDnsNamespace("example", {
 *     name: "hoge.example.local",
 *     description: "example",
 *     vpc: example.id,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Service Discovery Private DNS Namespace using the namespace ID and VPC ID. For example:
 *
 * ```sh
 * $ pulumi import aws:servicediscovery/privateDnsNamespace:PrivateDnsNamespace example 0123456789:vpc-123345
 * ```
 */
export declare class PrivateDnsNamespace extends pulumi.CustomResource {
    /**
     * Get an existing PrivateDnsNamespace resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: PrivateDnsNamespaceState, opts?: pulumi.CustomResourceOptions): PrivateDnsNamespace;
    /**
     * Returns true if the given object is an instance of PrivateDnsNamespace.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is PrivateDnsNamespace;
    /**
     * The ARN that Amazon Route 53 assigns to the namespace when you create it.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The description that you specify for the namespace when you create it.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * The ID for the hosted zone that Amazon Route 53 creates when you create a namespace.
     */
    readonly hostedZone: pulumi.Output<string>;
    /**
     * The name of the namespace.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * A map of tags to assign to the namespace. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * The ID of VPC that you want to associate the namespace with.
     */
    readonly vpc: pulumi.Output<string>;
    /**
     * Create a PrivateDnsNamespace resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: PrivateDnsNamespaceArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering PrivateDnsNamespace resources.
 */
export interface PrivateDnsNamespaceState {
    /**
     * The ARN that Amazon Route 53 assigns to the namespace when you create it.
     */
    arn?: pulumi.Input<string>;
    /**
     * The description that you specify for the namespace when you create it.
     */
    description?: pulumi.Input<string>;
    /**
     * The ID for the hosted zone that Amazon Route 53 creates when you create a namespace.
     */
    hostedZone?: pulumi.Input<string>;
    /**
     * The name of the namespace.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the namespace. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
    /**
     * The ID of VPC that you want to associate the namespace with.
     */
    vpc?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a PrivateDnsNamespace resource.
 */
export interface PrivateDnsNamespaceArgs {
    /**
     * The description that you specify for the namespace when you create it.
     */
    description?: pulumi.Input<string>;
    /**
     * The name of the namespace.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the namespace. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The ID of VPC that you want to associate the namespace with.
     */
    vpc: pulumi.Input<string>;
}
