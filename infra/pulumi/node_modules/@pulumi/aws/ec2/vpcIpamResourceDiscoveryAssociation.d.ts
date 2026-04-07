import * as pulumi from "@pulumi/pulumi";
/**
 * Provides an association between an Amazon IP Address Manager (IPAM) and a IPAM Resource Discovery. IPAM Resource Discoveries are resources meant for multi-organization customers. If you wish to use a single IPAM across multiple orgs, a resource discovery can be created and shared from a subordinate organization to the management organizations IPAM delegated admin account.
 *
 * Once an association is created between two organizations via IPAM & a IPAM Resource Discovery, IPAM Pools can be shared via Resource Access Manager (RAM) to accounts in the subordinate organization; these RAM shares must be accepted by the end user account. Pools can then also discover and monitor IPAM resources in the subordinate organization.
 *
 * ## Example Usage
 *
 * Basic usage:
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = new aws.ec2.VpcIpamResourceDiscoveryAssociation("test", {
 *     ipamId: testAwsVpcIpam.id,
 *     ipamResourceDiscoveryId: testAwsVpcIpamResourceDiscovery.id,
 *     tags: {
 *         Name: "test",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import IPAMs using the IPAM resource discovery association `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:ec2/vpcIpamResourceDiscoveryAssociation:VpcIpamResourceDiscoveryAssociation example ipam-res-disco-assoc-0178368ad2146a492
 * ```
 */
export declare class VpcIpamResourceDiscoveryAssociation extends pulumi.CustomResource {
    /**
     * Get an existing VpcIpamResourceDiscoveryAssociation resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: VpcIpamResourceDiscoveryAssociationState, opts?: pulumi.CustomResourceOptions): VpcIpamResourceDiscoveryAssociation;
    /**
     * Returns true if the given object is an instance of VpcIpamResourceDiscoveryAssociation.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is VpcIpamResourceDiscoveryAssociation;
    /**
     * The Amazon Resource Name (ARN) of IPAM Resource Discovery Association.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The Amazon Resource Name (ARN) of the IPAM.
     */
    readonly ipamArn: pulumi.Output<string>;
    /**
     * The ID of the IPAM to associate.
     */
    readonly ipamId: pulumi.Output<string>;
    /**
     * The home region of the IPAM.
     */
    readonly ipamRegion: pulumi.Output<string>;
    /**
     * The ID of the Resource Discovery to associate.
     */
    readonly ipamResourceDiscoveryId: pulumi.Output<string>;
    /**
     * A boolean to identify if the Resource Discovery is the accounts default resource discovery.
     */
    readonly isDefault: pulumi.Output<boolean>;
    /**
     * The account ID for the account that manages the Resource Discovery
     */
    readonly ownerId: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The lifecycle state of the association when you associate or disassociate a resource discovery.
     */
    readonly state: pulumi.Output<string>;
    /**
     * A map of tags to add to the IPAM resource discovery association resource.
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
     * Create a VpcIpamResourceDiscoveryAssociation resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: VpcIpamResourceDiscoveryAssociationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering VpcIpamResourceDiscoveryAssociation resources.
 */
export interface VpcIpamResourceDiscoveryAssociationState {
    /**
     * The Amazon Resource Name (ARN) of IPAM Resource Discovery Association.
     */
    arn?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) of the IPAM.
     */
    ipamArn?: pulumi.Input<string>;
    /**
     * The ID of the IPAM to associate.
     */
    ipamId?: pulumi.Input<string>;
    /**
     * The home region of the IPAM.
     */
    ipamRegion?: pulumi.Input<string>;
    /**
     * The ID of the Resource Discovery to associate.
     */
    ipamResourceDiscoveryId?: pulumi.Input<string>;
    /**
     * A boolean to identify if the Resource Discovery is the accounts default resource discovery.
     */
    isDefault?: pulumi.Input<boolean>;
    /**
     * The account ID for the account that manages the Resource Discovery
     */
    ownerId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The lifecycle state of the association when you associate or disassociate a resource discovery.
     */
    state?: pulumi.Input<string>;
    /**
     * A map of tags to add to the IPAM resource discovery association resource.
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
}
/**
 * The set of arguments for constructing a VpcIpamResourceDiscoveryAssociation resource.
 */
export interface VpcIpamResourceDiscoveryAssociationArgs {
    /**
     * The ID of the IPAM to associate.
     */
    ipamId: pulumi.Input<string>;
    /**
     * The ID of the Resource Discovery to associate.
     */
    ipamResourceDiscoveryId: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A map of tags to add to the IPAM resource discovery association resource.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
