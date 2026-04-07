import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages a firewall endpoint for an AWS Network Firewall firewall.
 *
 * Use `aws.networkfirewall.VpcEndpointAssociation` to establish new firewall endpoints in any Availability Zone where the firewall is already being used. The first use of a firewall in an Availability Zone must be defined by `aws.networkfirewall.Firewall` resource and `subnetMapping` argument.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.networkfirewall.VpcEndpointAssociation("example", {
 *     firewallArn: exampleAwsNetworkfirewallFirewall.arn,
 *     vpcId: exampleAwsVpc.id,
 *     subnetMapping: {
 *         subnetId: exampleTwo.id,
 *     },
 *     tags: {
 *         Name: "example endpoint",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Network Firewall VPC Endpoint Association using the `vpc_endpoint_association_arn`. For example:
 *
 * ```sh
 * $ pulumi import aws:networkfirewall/vpcEndpointAssociation:VpcEndpointAssociation example arn:aws:network-firewall:us-west-1:123456789012:vpc-endpoint-association/example
 * ```
 */
export declare class VpcEndpointAssociation extends pulumi.CustomResource {
    /**
     * Get an existing VpcEndpointAssociation resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: VpcEndpointAssociationState, opts?: pulumi.CustomResourceOptions): VpcEndpointAssociation;
    /**
     * Returns true if the given object is an instance of VpcEndpointAssociation.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is VpcEndpointAssociation;
    /**
     * A description of the VPC endpoint association.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * The Amazon Resource Name (ARN) that identifies the firewall.
     */
    readonly firewallArn: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The ID for a subnet that's used in an association with a firewall. See Subnet Mapping below for details.
     */
    readonly subnetMapping: pulumi.Output<outputs.networkfirewall.VpcEndpointAssociationSubnetMapping | undefined>;
    /**
     * Map of resource tags to associate with the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
    readonly timeouts: pulumi.Output<outputs.networkfirewall.VpcEndpointAssociationTimeouts | undefined>;
    /**
     * ARN of the VPC Endpoint Association.
     */
    readonly vpcEndpointAssociationArn: pulumi.Output<string>;
    /**
     * The unique identifier of the VPC endpoint association.
     */
    readonly vpcEndpointAssociationId: pulumi.Output<string>;
    /**
     * Nested list of information about the current status of the VPC Endpoint Association.
     */
    readonly vpcEndpointAssociationStatuses: pulumi.Output<outputs.networkfirewall.VpcEndpointAssociationVpcEndpointAssociationStatus[]>;
    /**
     * The unique identifier of the VPC for the endpoint association.
     */
    readonly vpcId: pulumi.Output<string>;
    /**
     * Create a VpcEndpointAssociation resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: VpcEndpointAssociationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering VpcEndpointAssociation resources.
 */
export interface VpcEndpointAssociationState {
    /**
     * A description of the VPC endpoint association.
     */
    description?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) that identifies the firewall.
     */
    firewallArn?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The ID for a subnet that's used in an association with a firewall. See Subnet Mapping below for details.
     */
    subnetMapping?: pulumi.Input<inputs.networkfirewall.VpcEndpointAssociationSubnetMapping>;
    /**
     * Map of resource tags to associate with the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
    timeouts?: pulumi.Input<inputs.networkfirewall.VpcEndpointAssociationTimeouts>;
    /**
     * ARN of the VPC Endpoint Association.
     */
    vpcEndpointAssociationArn?: pulumi.Input<string>;
    /**
     * The unique identifier of the VPC endpoint association.
     */
    vpcEndpointAssociationId?: pulumi.Input<string>;
    /**
     * Nested list of information about the current status of the VPC Endpoint Association.
     */
    vpcEndpointAssociationStatuses?: pulumi.Input<pulumi.Input<inputs.networkfirewall.VpcEndpointAssociationVpcEndpointAssociationStatus>[]>;
    /**
     * The unique identifier of the VPC for the endpoint association.
     */
    vpcId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a VpcEndpointAssociation resource.
 */
export interface VpcEndpointAssociationArgs {
    /**
     * A description of the VPC endpoint association.
     */
    description?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) that identifies the firewall.
     */
    firewallArn: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The ID for a subnet that's used in an association with a firewall. See Subnet Mapping below for details.
     */
    subnetMapping?: pulumi.Input<inputs.networkfirewall.VpcEndpointAssociationSubnetMapping>;
    /**
     * Map of resource tags to associate with the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    timeouts?: pulumi.Input<inputs.networkfirewall.VpcEndpointAssociationTimeouts>;
    /**
     * The unique identifier of the VPC for the endpoint association.
     */
    vpcId: pulumi.Input<string>;
}
