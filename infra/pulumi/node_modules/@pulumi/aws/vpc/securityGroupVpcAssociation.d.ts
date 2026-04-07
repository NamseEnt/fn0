import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing Security Group VPC Associations.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.vpc.SecurityGroupVpcAssociation("example", {
 *     securityGroupId: "sg-05f1f54ab49bb39a3",
 *     vpcId: "vpc-01df9d105095412ba",
 * });
 * ```
 *
 * ## Import
 *
 * ### Identity Schema
 *
 * #### Required
 *
 * * `vpc_id` (String) VPC ID.
 *
 * * `security_group_id` (String) Security Group ID.
 *
 * #### Optional
 *
 * * `account_id` (String) AWS Account where this resource is managed.
 *
 * * `region` (String) Region where this resource is managed.
 *
 * Using `pulumi import`, import a Security Group VPC Association using the `security_group_id` and `vpc_id` arguments, separated by a comma (`,`). For example:
 *
 * console
 *
 * % pulumi import aws_vpc_security_group_vpc_association.example sg-12345,vpc-67890
 */
export declare class SecurityGroupVpcAssociation extends pulumi.CustomResource {
    /**
     * Get an existing SecurityGroupVpcAssociation resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: SecurityGroupVpcAssociationState, opts?: pulumi.CustomResourceOptions): SecurityGroupVpcAssociation;
    /**
     * Returns true if the given object is an instance of SecurityGroupVpcAssociation.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is SecurityGroupVpcAssociation;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The ID of the security group.
     */
    readonly securityGroupId: pulumi.Output<string>;
    /**
     * State of the VPC association. See the [AWS documentation](https://docs.aws.amazon.com/AWSEC2/latest/APIReference/API_SecurityGroupVpcAssociation.html) for possible values.
     */
    readonly state: pulumi.Output<string>;
    readonly timeouts: pulumi.Output<outputs.vpc.SecurityGroupVpcAssociationTimeouts | undefined>;
    /**
     * The ID of the VPC to make the association with.
     */
    readonly vpcId: pulumi.Output<string>;
    /**
     * Create a SecurityGroupVpcAssociation resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: SecurityGroupVpcAssociationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering SecurityGroupVpcAssociation resources.
 */
export interface SecurityGroupVpcAssociationState {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The ID of the security group.
     */
    securityGroupId?: pulumi.Input<string>;
    /**
     * State of the VPC association. See the [AWS documentation](https://docs.aws.amazon.com/AWSEC2/latest/APIReference/API_SecurityGroupVpcAssociation.html) for possible values.
     */
    state?: pulumi.Input<string>;
    timeouts?: pulumi.Input<inputs.vpc.SecurityGroupVpcAssociationTimeouts>;
    /**
     * The ID of the VPC to make the association with.
     */
    vpcId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a SecurityGroupVpcAssociation resource.
 */
export interface SecurityGroupVpcAssociationArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The ID of the security group.
     */
    securityGroupId: pulumi.Input<string>;
    timeouts?: pulumi.Input<inputs.vpc.SecurityGroupVpcAssociationTimeouts>;
    /**
     * The ID of the VPC to make the association with.
     */
    vpcId: pulumi.Input<string>;
}
