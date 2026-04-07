import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS VPC Lattice Service Network Resource Association.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.vpclattice.ServiceNetworkResourceAssociation("example", {
 *     resourceConfigurationIdentifier: exampleAwsVpclatticeResourceConfiguration.id,
 *     serviceNetworkIdentifier: exampleAwsVpclatticeServiceNetwork.id,
 *     tags: {
 *         Name: "Example",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import VPC Lattice Service Network Resource Association using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:vpclattice/serviceNetworkResourceAssociation:ServiceNetworkResourceAssociation example snra-1234567890abcef12
 * ```
 */
export declare class ServiceNetworkResourceAssociation extends pulumi.CustomResource {
    /**
     * Get an existing ServiceNetworkResourceAssociation resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ServiceNetworkResourceAssociationState, opts?: pulumi.CustomResourceOptions): ServiceNetworkResourceAssociation;
    /**
     * Returns true if the given object is an instance of ServiceNetworkResourceAssociation.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ServiceNetworkResourceAssociation;
    /**
     * ARN of the Service Network Resource Association.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * DNS entry of the association in the service network.
     */
    readonly dnsEntries: pulumi.Output<outputs.vpclattice.ServiceNetworkResourceAssociationDnsEntry[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Identifier of Resource Configuration to associate to the Service Network.
     */
    readonly resourceConfigurationIdentifier: pulumi.Output<string>;
    /**
     * Identifier of the Service Network to associate the Resource to.
     *
     * The following arguments are optional:
     */
    readonly serviceNetworkIdentifier: pulumi.Output<string>;
    /**
     * Map of tags assigned to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
    readonly timeouts: pulumi.Output<outputs.vpclattice.ServiceNetworkResourceAssociationTimeouts | undefined>;
    /**
     * Create a ServiceNetworkResourceAssociation resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ServiceNetworkResourceAssociationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ServiceNetworkResourceAssociation resources.
 */
export interface ServiceNetworkResourceAssociationState {
    /**
     * ARN of the Service Network Resource Association.
     */
    arn?: pulumi.Input<string>;
    /**
     * DNS entry of the association in the service network.
     */
    dnsEntries?: pulumi.Input<pulumi.Input<inputs.vpclattice.ServiceNetworkResourceAssociationDnsEntry>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Identifier of Resource Configuration to associate to the Service Network.
     */
    resourceConfigurationIdentifier?: pulumi.Input<string>;
    /**
     * Identifier of the Service Network to associate the Resource to.
     *
     * The following arguments are optional:
     */
    serviceNetworkIdentifier?: pulumi.Input<string>;
    /**
     * Map of tags assigned to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
    timeouts?: pulumi.Input<inputs.vpclattice.ServiceNetworkResourceAssociationTimeouts>;
}
/**
 * The set of arguments for constructing a ServiceNetworkResourceAssociation resource.
 */
export interface ServiceNetworkResourceAssociationArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Identifier of Resource Configuration to associate to the Service Network.
     */
    resourceConfigurationIdentifier: pulumi.Input<string>;
    /**
     * Identifier of the Service Network to associate the Resource to.
     *
     * The following arguments are optional:
     */
    serviceNetworkIdentifier: pulumi.Input<string>;
    /**
     * Map of tags assigned to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    timeouts?: pulumi.Input<inputs.vpclattice.ServiceNetworkResourceAssociationTimeouts>;
}
