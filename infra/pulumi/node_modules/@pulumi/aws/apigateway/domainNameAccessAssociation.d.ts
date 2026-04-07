import * as pulumi from "@pulumi/pulumi";
/**
 * Creates a domain name access association resource between an access association source and a private custom domain name.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.apigateway.DomainNameAccessAssociation("example", {
 *     accessAssociationSource: exampleAwsVpcEndpoint.id,
 *     accessAssociationSourceType: "VPCE",
 *     domainNameArn: exampleAwsApiGatewayDomainName.arn,
 * });
 * ```
 *
 * ## Import
 *
 * ### Identity Schema
 *
 * #### Required
 *
 * - `arn` (String) Amazon Resource Name (ARN) of the API Gateway domain name access association.
 *
 * Using `pulumi import`, import API Gateway domain name acces associations as using their `arn`. For example:
 *
 * console
 *
 * % pulumi import aws_api_gateway_domain_name_access_association.example arn:aws:apigateway:us-west-2:123456789012:/domainnameaccessassociations/domainname/12qmzgp2.9m7ilski.test+hykg7a12e7/vpcesource/vpce-05de3f8f82740a748
 */
export declare class DomainNameAccessAssociation extends pulumi.CustomResource {
    /**
     * Get an existing DomainNameAccessAssociation resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: DomainNameAccessAssociationState, opts?: pulumi.CustomResourceOptions): DomainNameAccessAssociation;
    /**
     * Returns true if the given object is an instance of DomainNameAccessAssociation.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is DomainNameAccessAssociation;
    /**
     * The identifier of the domain name access association source. For a `VPCE`, the value is the VPC endpoint ID.
     */
    readonly accessAssociationSource: pulumi.Output<string>;
    /**
     * The type of the domain name access association source. Valid values are `VPCE`.
     */
    readonly accessAssociationSourceType: pulumi.Output<string>;
    /**
     * ARN of the domain name access association.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The ARN of the domain name.
     */
    readonly domainNameArn: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Create a DomainNameAccessAssociation resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: DomainNameAccessAssociationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering DomainNameAccessAssociation resources.
 */
export interface DomainNameAccessAssociationState {
    /**
     * The identifier of the domain name access association source. For a `VPCE`, the value is the VPC endpoint ID.
     */
    accessAssociationSource?: pulumi.Input<string>;
    /**
     * The type of the domain name access association source. Valid values are `VPCE`.
     */
    accessAssociationSourceType?: pulumi.Input<string>;
    /**
     * ARN of the domain name access association.
     */
    arn?: pulumi.Input<string>;
    /**
     * The ARN of the domain name.
     */
    domainNameArn?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
 * The set of arguments for constructing a DomainNameAccessAssociation resource.
 */
export interface DomainNameAccessAssociationArgs {
    /**
     * The identifier of the domain name access association source. For a `VPCE`, the value is the VPC endpoint ID.
     */
    accessAssociationSource: pulumi.Input<string>;
    /**
     * The type of the domain name access association source. Valid values are `VPCE`.
     */
    accessAssociationSourceType: pulumi.Input<string>;
    /**
     * The ARN of the domain name.
     */
    domainNameArn: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
