import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS EC2 (Elastic Compute Cloud) VPC Block Public Access Exclusion.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = new aws.ec2.Vpc("test", {cidrBlock: "10.1.0.0/16"});
 * const testVpcBlockPublicAccessExclusion = new aws.ec2.VpcBlockPublicAccessExclusion("test", {
 *     vpcId: test.id,
 *     internetGatewayExclusionMode: "allow-bidirectional",
 * });
 * ```
 *
 * ### Usage with subnet id
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = new aws.ec2.Vpc("test", {cidrBlock: "10.1.0.0/16"});
 * const testSubnet = new aws.ec2.Subnet("test", {
 *     cidrBlock: "10.1.1.0/24",
 *     vpcId: test.id,
 * });
 * const testVpcBlockPublicAccessExclusion = new aws.ec2.VpcBlockPublicAccessExclusion("test", {
 *     subnetId: testSubnet.id,
 *     internetGatewayExclusionMode: "allow-egress",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import EC2 (Elastic Compute Cloud) VPC Block Public Access Exclusion using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:ec2/vpcBlockPublicAccessExclusion:VpcBlockPublicAccessExclusion example vpcbpa-exclude-1234abcd
 * ```
 */
export declare class VpcBlockPublicAccessExclusion extends pulumi.CustomResource {
    /**
     * Get an existing VpcBlockPublicAccessExclusion resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: VpcBlockPublicAccessExclusionState, opts?: pulumi.CustomResourceOptions): VpcBlockPublicAccessExclusion;
    /**
     * Returns true if the given object is an instance of VpcBlockPublicAccessExclusion.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is VpcBlockPublicAccessExclusion;
    /**
     * Mode of exclusion from Block Public Access. The allowed values are `allow-egress` and `allow-bidirectional`.
     *
     * The following arguments are optional:
     */
    readonly internetGatewayExclusionMode: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The Amazon Resource Name (ARN) the excluded resource.
     */
    readonly resourceArn: pulumi.Output<string>;
    /**
     * Id of the subnet to which this exclusion applies. Either this or the vpcId needs to be provided.
     */
    readonly subnetId: pulumi.Output<string | undefined>;
    /**
     * A map of tags to assign to the exclusion. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
    readonly timeouts: pulumi.Output<outputs.ec2.VpcBlockPublicAccessExclusionTimeouts | undefined>;
    /**
     * Id of the VPC to which this exclusion applies. Either this or the subnetId needs to be provided.
     */
    readonly vpcId: pulumi.Output<string | undefined>;
    /**
     * Create a VpcBlockPublicAccessExclusion resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: VpcBlockPublicAccessExclusionArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering VpcBlockPublicAccessExclusion resources.
 */
export interface VpcBlockPublicAccessExclusionState {
    /**
     * Mode of exclusion from Block Public Access. The allowed values are `allow-egress` and `allow-bidirectional`.
     *
     * The following arguments are optional:
     */
    internetGatewayExclusionMode?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) the excluded resource.
     */
    resourceArn?: pulumi.Input<string>;
    /**
     * Id of the subnet to which this exclusion applies. Either this or the vpcId needs to be provided.
     */
    subnetId?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the exclusion. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
    timeouts?: pulumi.Input<inputs.ec2.VpcBlockPublicAccessExclusionTimeouts>;
    /**
     * Id of the VPC to which this exclusion applies. Either this or the subnetId needs to be provided.
     */
    vpcId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a VpcBlockPublicAccessExclusion resource.
 */
export interface VpcBlockPublicAccessExclusionArgs {
    /**
     * Mode of exclusion from Block Public Access. The allowed values are `allow-egress` and `allow-bidirectional`.
     *
     * The following arguments are optional:
     */
    internetGatewayExclusionMode: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Id of the subnet to which this exclusion applies. Either this or the vpcId needs to be provided.
     */
    subnetId?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the exclusion. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    timeouts?: pulumi.Input<inputs.ec2.VpcBlockPublicAccessExclusionTimeouts>;
    /**
     * Id of the VPC to which this exclusion applies. Either this or the subnetId needs to be provided.
     */
    vpcId?: pulumi.Input<string>;
}
