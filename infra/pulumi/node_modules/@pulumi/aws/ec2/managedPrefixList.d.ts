import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a managed prefix list resource.
 *
 * > **NOTE on Managed Prefix Lists and Managed Prefix List Entries:** The provider
 * currently provides both a standalone Managed Prefix List Entry resource (a single entry),
 * and a Managed Prefix List resource with entries defined in-line. At this time you
 * cannot use a Managed Prefix List with in-line rules in conjunction with any Managed
 * Prefix List Entry resources. Doing so will cause a conflict of entries and will overwrite entries.
 *
 * > **NOTE on `maxEntries`:** When you reference a Prefix List in a resource,
 * the maximum number of entries for the prefix lists counts as the same number of rules
 * or entries for the resource. For example, if you create a prefix list with a maximum
 * of 20 entries and you reference that prefix list in a security group rule, this counts
 * as 20 rules for the security group.
 *
 * ## Example Usage
 *
 * Basic usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.ec2.ManagedPrefixList("example", {
 *     name: "All VPC CIDR-s",
 *     addressFamily: "IPv4",
 *     maxEntries: 5,
 *     entries: [
 *         {
 *             cidr: exampleAwsVpc.cidrBlock,
 *             description: "Primary",
 *         },
 *         {
 *             cidr: exampleAwsVpcIpv4CidrBlockAssociation.cidrBlock,
 *             description: "Secondary",
 *         },
 *     ],
 *     tags: {
 *         Env: "live",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Prefix Lists using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:ec2/managedPrefixList:ManagedPrefixList default pl-0570a1d2d725c16be
 * ```
 */
export declare class ManagedPrefixList extends pulumi.CustomResource {
    /**
     * Get an existing ManagedPrefixList resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ManagedPrefixListState, opts?: pulumi.CustomResourceOptions): ManagedPrefixList;
    /**
     * Returns true if the given object is an instance of ManagedPrefixList.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ManagedPrefixList;
    /**
     * Address family (`IPv4` or `IPv6`) of this prefix list.
     */
    readonly addressFamily: pulumi.Output<string>;
    /**
     * ARN of the prefix list.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Configuration block for prefix list entry. Detailed below. Different entries may have overlapping CIDR blocks, but a particular CIDR should not be duplicated.
     */
    readonly entries: pulumi.Output<outputs.ec2.ManagedPrefixListEntry[]>;
    /**
     * Maximum number of entries that this prefix list can contain.
     */
    readonly maxEntries: pulumi.Output<number>;
    /**
     * Name of this resource. The name must not start with `com.amazonaws`.
     */
    readonly name: pulumi.Output<string>;
    /**
     * ID of the AWS account that owns this prefix list.
     */
    readonly ownerId: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Map of tags to assign to this resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Latest version of this prefix list.
     */
    readonly version: pulumi.Output<number>;
    /**
     * Create a ManagedPrefixList resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ManagedPrefixListArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ManagedPrefixList resources.
 */
export interface ManagedPrefixListState {
    /**
     * Address family (`IPv4` or `IPv6`) of this prefix list.
     */
    addressFamily?: pulumi.Input<string>;
    /**
     * ARN of the prefix list.
     */
    arn?: pulumi.Input<string>;
    /**
     * Configuration block for prefix list entry. Detailed below. Different entries may have overlapping CIDR blocks, but a particular CIDR should not be duplicated.
     */
    entries?: pulumi.Input<pulumi.Input<inputs.ec2.ManagedPrefixListEntry>[]>;
    /**
     * Maximum number of entries that this prefix list can contain.
     */
    maxEntries?: pulumi.Input<number>;
    /**
     * Name of this resource. The name must not start with `com.amazonaws`.
     */
    name?: pulumi.Input<string>;
    /**
     * ID of the AWS account that owns this prefix list.
     */
    ownerId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Map of tags to assign to this resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
    /**
     * Latest version of this prefix list.
     */
    version?: pulumi.Input<number>;
}
/**
 * The set of arguments for constructing a ManagedPrefixList resource.
 */
export interface ManagedPrefixListArgs {
    /**
     * Address family (`IPv4` or `IPv6`) of this prefix list.
     */
    addressFamily: pulumi.Input<string>;
    /**
     * Configuration block for prefix list entry. Detailed below. Different entries may have overlapping CIDR blocks, but a particular CIDR should not be duplicated.
     */
    entries?: pulumi.Input<pulumi.Input<inputs.ec2.ManagedPrefixListEntry>[]>;
    /**
     * Maximum number of entries that this prefix list can contain.
     */
    maxEntries: pulumi.Input<number>;
    /**
     * Name of this resource. The name must not start with `com.amazonaws`.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Map of tags to assign to this resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
