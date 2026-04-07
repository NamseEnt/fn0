import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS VPC Lattice Service Network Service Association.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.vpclattice.ServiceNetworkServiceAssociation("example", {
 *     serviceIdentifier: exampleAwsVpclatticeService.id,
 *     serviceNetworkIdentifier: exampleAwsVpclatticeServiceNetwork.id,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import VPC Lattice Service Network Service Association using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:vpclattice/serviceNetworkServiceAssociation:ServiceNetworkServiceAssociation example snsa-05e2474658a88f6ba
 * ```
 */
export declare class ServiceNetworkServiceAssociation extends pulumi.CustomResource {
    /**
     * Get an existing ServiceNetworkServiceAssociation resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ServiceNetworkServiceAssociationState, opts?: pulumi.CustomResourceOptions): ServiceNetworkServiceAssociation;
    /**
     * Returns true if the given object is an instance of ServiceNetworkServiceAssociation.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ServiceNetworkServiceAssociation;
    /**
     * The ARN of the Association.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The account that created the association.
     */
    readonly createdBy: pulumi.Output<string>;
    /**
     * The custom domain name of the service.
     */
    readonly customDomainName: pulumi.Output<string>;
    /**
     * The DNS name of the service.
     */
    readonly dnsEntries: pulumi.Output<outputs.vpclattice.ServiceNetworkServiceAssociationDnsEntry[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The ID or Amazon Resource Identifier (ARN) of the service.
     */
    readonly serviceIdentifier: pulumi.Output<string>;
    /**
     * The ID or Amazon Resource Identifier (ARN) of the service network. You must use the ARN if the resources specified in the operation are in different accounts.
     */
    readonly serviceNetworkIdentifier: pulumi.Output<string>;
    /**
     * The operations status. Valid Values are CREATE_IN_PROGRESS | ACTIVE | DELETE_IN_PROGRESS | CREATE_FAILED | DELETE_FAILED
     */
    readonly status: pulumi.Output<string>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Create a ServiceNetworkServiceAssociation resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ServiceNetworkServiceAssociationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ServiceNetworkServiceAssociation resources.
 */
export interface ServiceNetworkServiceAssociationState {
    /**
     * The ARN of the Association.
     */
    arn?: pulumi.Input<string>;
    /**
     * The account that created the association.
     */
    createdBy?: pulumi.Input<string>;
    /**
     * The custom domain name of the service.
     */
    customDomainName?: pulumi.Input<string>;
    /**
     * The DNS name of the service.
     */
    dnsEntries?: pulumi.Input<pulumi.Input<inputs.vpclattice.ServiceNetworkServiceAssociationDnsEntry>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The ID or Amazon Resource Identifier (ARN) of the service.
     */
    serviceIdentifier?: pulumi.Input<string>;
    /**
     * The ID or Amazon Resource Identifier (ARN) of the service network. You must use the ARN if the resources specified in the operation are in different accounts.
     */
    serviceNetworkIdentifier?: pulumi.Input<string>;
    /**
     * The operations status. Valid Values are CREATE_IN_PROGRESS | ACTIVE | DELETE_IN_PROGRESS | CREATE_FAILED | DELETE_FAILED
     */
    status?: pulumi.Input<string>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
 * The set of arguments for constructing a ServiceNetworkServiceAssociation resource.
 */
export interface ServiceNetworkServiceAssociationArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The ID or Amazon Resource Identifier (ARN) of the service.
     */
    serviceIdentifier: pulumi.Input<string>;
    /**
     * The ID or Amazon Resource Identifier (ARN) of the service network. You must use the ARN if the resources specified in the operation are in different accounts.
     */
    serviceNetworkIdentifier: pulumi.Input<string>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
