import * as pulumi from "@pulumi/pulumi";
/**
 * Resource for enabling private DNS on an AWS VPC (Virtual Private Cloud) Endpoint.
 *
 * > When using this resource, the `privateDnsEnabled` argument should be omitted on the parent `aws.ec2.VpcEndpoint` resource.
 * Setting the value both places can lead to unintended behavior and persistent differences.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.vpc.EndpointPrivateDns("example", {
 *     vpcEndpointId: exampleAwsVpcEndpoint.id,
 *     privateDnsEnabled: true,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import a VPC (Virtual Private Cloud) Endpoint Private DNS using the `vpc_endpoint_id`. For example:
 *
 * ```sh
 * $ pulumi import aws:vpc/endpointPrivateDns:EndpointPrivateDns example vpce-abcd-1234
 * ```
 */
export declare class EndpointPrivateDns extends pulumi.CustomResource {
    /**
     * Get an existing EndpointPrivateDns resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: EndpointPrivateDnsState, opts?: pulumi.CustomResourceOptions): EndpointPrivateDns;
    /**
     * Returns true if the given object is an instance of EndpointPrivateDns.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is EndpointPrivateDns;
    /**
     * Indicates whether a private hosted zone is associated with the VPC. Only applicable for `Interface` endpoints.
     */
    readonly privateDnsEnabled: pulumi.Output<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * VPC endpoint identifier.
     */
    readonly vpcEndpointId: pulumi.Output<string>;
    /**
     * Create a EndpointPrivateDns resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: EndpointPrivateDnsArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering EndpointPrivateDns resources.
 */
export interface EndpointPrivateDnsState {
    /**
     * Indicates whether a private hosted zone is associated with the VPC. Only applicable for `Interface` endpoints.
     */
    privateDnsEnabled?: pulumi.Input<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * VPC endpoint identifier.
     */
    vpcEndpointId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a EndpointPrivateDns resource.
 */
export interface EndpointPrivateDnsArgs {
    /**
     * Indicates whether a private hosted zone is associated with the VPC. Only applicable for `Interface` endpoints.
     */
    privateDnsEnabled: pulumi.Input<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * VPC endpoint identifier.
     */
    vpcEndpointId: pulumi.Input<string>;
}
