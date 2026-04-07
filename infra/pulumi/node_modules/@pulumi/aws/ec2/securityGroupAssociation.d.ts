import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a resource to create an association between a VPC endpoint and a security group.
 *
 * > **NOTE on VPC Endpoints and VPC Endpoint Security Group Associations:** The provider provides
 * both a standalone VPC Endpoint Security Group Association (an association between a VPC endpoint
 * and a single `securityGroupId`) and a VPC Endpoint resource with a `securityGroupIds`
 * attribute. Do not use the same security group ID in both a VPC Endpoint resource and a VPC Endpoint Security
 * Group Association resource. Doing so will cause a conflict of associations and will overwrite the association.
 *
 * ## Example Usage
 *
 * Basic usage:
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const sgEc2 = new aws.ec2.SecurityGroupAssociation("sg_ec2", {
 *     vpcEndpointId: ec2.id,
 *     securityGroupId: sg.id,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import VPC Endpoint Security Group Associations using `vpc_endpoint_id` together with `security_group_id`. For example:
 *
 * ```sh
 * $ pulumi import aws:ec2/securityGroupAssociation:SecurityGroupAssociation example vpce-aaaaaaaa/sg-bbbbbbbbbbbbbbbbb
 * ```
 */
export declare class SecurityGroupAssociation extends pulumi.CustomResource {
    /**
     * Get an existing SecurityGroupAssociation resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: SecurityGroupAssociationState, opts?: pulumi.CustomResourceOptions): SecurityGroupAssociation;
    /**
     * Returns true if the given object is an instance of SecurityGroupAssociation.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is SecurityGroupAssociation;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Whether this association should replace the association with the VPC's default security group that is created when no security groups are specified during VPC endpoint creation. At most 1 association per-VPC endpoint should be configured with `replaceDefaultAssociation = true`. `false` should be used when importing resources.
     */
    readonly replaceDefaultAssociation: pulumi.Output<boolean | undefined>;
    /**
     * The ID of the security group to be associated with the VPC endpoint.
     */
    readonly securityGroupId: pulumi.Output<string>;
    /**
     * The ID of the VPC endpoint with which the security group will be associated.
     */
    readonly vpcEndpointId: pulumi.Output<string>;
    /**
     * Create a SecurityGroupAssociation resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: SecurityGroupAssociationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering SecurityGroupAssociation resources.
 */
export interface SecurityGroupAssociationState {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Whether this association should replace the association with the VPC's default security group that is created when no security groups are specified during VPC endpoint creation. At most 1 association per-VPC endpoint should be configured with `replaceDefaultAssociation = true`. `false` should be used when importing resources.
     */
    replaceDefaultAssociation?: pulumi.Input<boolean>;
    /**
     * The ID of the security group to be associated with the VPC endpoint.
     */
    securityGroupId?: pulumi.Input<string>;
    /**
     * The ID of the VPC endpoint with which the security group will be associated.
     */
    vpcEndpointId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a SecurityGroupAssociation resource.
 */
export interface SecurityGroupAssociationArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Whether this association should replace the association with the VPC's default security group that is created when no security groups are specified during VPC endpoint creation. At most 1 association per-VPC endpoint should be configured with `replaceDefaultAssociation = true`. `false` should be used when importing resources.
     */
    replaceDefaultAssociation?: pulumi.Input<boolean>;
    /**
     * The ID of the security group to be associated with the VPC endpoint.
     */
    securityGroupId: pulumi.Input<string>;
    /**
     * The ID of the VPC endpoint with which the security group will be associated.
     */
    vpcEndpointId: pulumi.Input<string>;
}
