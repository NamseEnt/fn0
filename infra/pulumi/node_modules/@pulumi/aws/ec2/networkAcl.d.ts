import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides an network ACL resource. You might set up network ACLs with rules similar
 * to your security groups in order to add an additional layer of security to your VPC.
 *
 * > **NOTE on Network ACLs and Network ACL Rules:** This provider currently
 * provides both a standalone Network ACL Rule resource and a Network ACL resource with rules
 * defined in-line. At this time you cannot use a Network ACL with in-line rules
 * in conjunction with any Network ACL Rule resources. Doing so will cause
 * a conflict of rule settings and will overwrite rules.
 *
 * > **NOTE on Network ACLs and Network ACL Associations:** the provider provides both a standalone network ACL association
 * resource and a network ACL resource with a `subnetIds` attribute. Do not use the same subnet ID in both a network ACL
 * resource and a network ACL association resource. Doing so will cause a conflict of associations and will overwrite the association.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const main = new aws.ec2.NetworkAcl("main", {
 *     vpcId: mainAwsVpc.id,
 *     egress: [{
 *         protocol: "tcp",
 *         ruleNo: 200,
 *         action: "allow",
 *         cidrBlock: "10.3.0.0/18",
 *         fromPort: 443,
 *         toPort: 443,
 *     }],
 *     ingress: [{
 *         protocol: "tcp",
 *         ruleNo: 100,
 *         action: "allow",
 *         cidrBlock: "10.3.0.0/18",
 *         fromPort: 80,
 *         toPort: 80,
 *     }],
 *     tags: {
 *         Name: "main",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Network ACLs using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:ec2/networkAcl:NetworkAcl main acl-7aaabd18
 * ```
 */
export declare class NetworkAcl extends pulumi.CustomResource {
    /**
     * Get an existing NetworkAcl resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: NetworkAclState, opts?: pulumi.CustomResourceOptions): NetworkAcl;
    /**
     * Returns true if the given object is an instance of NetworkAcl.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is NetworkAcl;
    /**
     * The ARN of the network ACL
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Specifies an egress rule. Parameters defined below.
     */
    readonly egress: pulumi.Output<outputs.ec2.NetworkAclEgress[]>;
    /**
     * Specifies an ingress rule. Parameters defined below.
     */
    readonly ingress: pulumi.Output<outputs.ec2.NetworkAclIngress[]>;
    /**
     * The ID of the AWS account that owns the network ACL.
     */
    readonly ownerId: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * A list of Subnet IDs to apply the ACL to
     */
    readonly subnetIds: pulumi.Output<string[]>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * The ID of the associated VPC.
     */
    readonly vpcId: pulumi.Output<string>;
    /**
     * Create a NetworkAcl resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: NetworkAclArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering NetworkAcl resources.
 */
export interface NetworkAclState {
    /**
     * The ARN of the network ACL
     */
    arn?: pulumi.Input<string>;
    /**
     * Specifies an egress rule. Parameters defined below.
     */
    egress?: pulumi.Input<pulumi.Input<inputs.ec2.NetworkAclEgress>[]>;
    /**
     * Specifies an ingress rule. Parameters defined below.
     */
    ingress?: pulumi.Input<pulumi.Input<inputs.ec2.NetworkAclIngress>[]>;
    /**
     * The ID of the AWS account that owns the network ACL.
     */
    ownerId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A list of Subnet IDs to apply the ACL to
     */
    subnetIds?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * The ID of the associated VPC.
     */
    vpcId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a NetworkAcl resource.
 */
export interface NetworkAclArgs {
    /**
     * Specifies an egress rule. Parameters defined below.
     */
    egress?: pulumi.Input<pulumi.Input<inputs.ec2.NetworkAclEgress>[]>;
    /**
     * Specifies an ingress rule. Parameters defined below.
     */
    ingress?: pulumi.Input<pulumi.Input<inputs.ec2.NetworkAclIngress>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A list of Subnet IDs to apply the ACL to
     */
    subnetIds?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The ID of the associated VPC.
     */
    vpcId: pulumi.Input<string>;
}
