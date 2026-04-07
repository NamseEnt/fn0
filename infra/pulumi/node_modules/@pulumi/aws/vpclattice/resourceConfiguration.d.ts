import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS VPC Lattice Resource Configuration.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.vpclattice.ResourceConfiguration("example", {
 *     name: "Example",
 *     resourceGatewayIdentifier: exampleAwsVpclatticeResourceGateway.id,
 *     portRanges: ["80"],
 *     protocol: "TCP",
 *     resourceConfigurationDefinition: {
 *         dnsResource: {
 *             domainName: "example.com",
 *             ipAddressType: "IPV4",
 *         },
 *     },
 *     tags: {
 *         Environment: "Example",
 *     },
 * });
 * ```
 *
 * ### IP Address Example
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.vpclattice.ResourceConfiguration("example", {
 *     name: "Example",
 *     resourceGatewayIdentifier: exampleAwsVpclatticeResourceGateway.id,
 *     portRanges: ["80"],
 *     protocol: "TCP",
 *     resourceConfigurationDefinition: {
 *         ipResource: {
 *             ipAddress: "10.0.0.1",
 *         },
 *     },
 *     tags: {
 *         Environment: "Example",
 *     },
 * });
 * ```
 *
 * ### ARN Example
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = new aws.vpclattice.ResourceConfiguration("test", {
 *     name: "Example",
 *     resourceGatewayIdentifier: testAwsVpclatticeResourceGateway.id,
 *     type: "ARN",
 *     resourceConfigurationDefinition: {
 *         arnResource: {
 *             arn: example.arn,
 *         },
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import VPC Lattice Resource Configuration using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:vpclattice/resourceConfiguration:ResourceConfiguration example rcfg-1234567890abcdef1
 * ```
 */
export declare class ResourceConfiguration extends pulumi.CustomResource {
    /**
     * Get an existing ResourceConfiguration resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ResourceConfigurationState, opts?: pulumi.CustomResourceOptions): ResourceConfiguration;
    /**
     * Returns true if the given object is an instance of ResourceConfiguration.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ResourceConfiguration;
    /**
     * Allow or Deny the association of this resource to a shareable service network.
     */
    readonly allowAssociationToShareableServiceNetwork: pulumi.Output<boolean>;
    /**
     * ARN of the resource gateway.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Name for the Resource Configuration.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Port ranges to access the Resource either single port `80` or range `80-81` range.
     */
    readonly portRanges: pulumi.Output<string[]>;
    /**
     * Protocol for the Resource `TCP` is currently the only supported value.  MUST be specified if `resourceConfigurationGroupId` is not.
     */
    readonly protocol: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Details of the Resource Configuration. See `resourceConfigurationDefinition` Block for details.
     *
     * The following arguments are optional:
     */
    readonly resourceConfigurationDefinition: pulumi.Output<outputs.vpclattice.ResourceConfigurationResourceConfigurationDefinition | undefined>;
    /**
     * ID of Resource Configuration where `type` is `CHILD`.
     */
    readonly resourceConfigurationGroupId: pulumi.Output<string | undefined>;
    /**
     * ID of the Resource Gateway used to access the resource. MUST be specified if `resourceConfigurationGroupId` is not.
     */
    readonly resourceGatewayIdentifier: pulumi.Output<string>;
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
    readonly timeouts: pulumi.Output<outputs.vpclattice.ResourceConfigurationTimeouts | undefined>;
    /**
     * Type of Resource Configuration. Must be one of `GROUP`, `CHILD`, `SINGLE`, `ARN`.
     */
    readonly type: pulumi.Output<string>;
    /**
     * Create a ResourceConfiguration resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: ResourceConfigurationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ResourceConfiguration resources.
 */
export interface ResourceConfigurationState {
    /**
     * Allow or Deny the association of this resource to a shareable service network.
     */
    allowAssociationToShareableServiceNetwork?: pulumi.Input<boolean>;
    /**
     * ARN of the resource gateway.
     */
    arn?: pulumi.Input<string>;
    /**
     * Name for the Resource Configuration.
     */
    name?: pulumi.Input<string>;
    /**
     * Port ranges to access the Resource either single port `80` or range `80-81` range.
     */
    portRanges?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Protocol for the Resource `TCP` is currently the only supported value.  MUST be specified if `resourceConfigurationGroupId` is not.
     */
    protocol?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Details of the Resource Configuration. See `resourceConfigurationDefinition` Block for details.
     *
     * The following arguments are optional:
     */
    resourceConfigurationDefinition?: pulumi.Input<inputs.vpclattice.ResourceConfigurationResourceConfigurationDefinition>;
    /**
     * ID of Resource Configuration where `type` is `CHILD`.
     */
    resourceConfigurationGroupId?: pulumi.Input<string>;
    /**
     * ID of the Resource Gateway used to access the resource. MUST be specified if `resourceConfigurationGroupId` is not.
     */
    resourceGatewayIdentifier?: pulumi.Input<string>;
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
    timeouts?: pulumi.Input<inputs.vpclattice.ResourceConfigurationTimeouts>;
    /**
     * Type of Resource Configuration. Must be one of `GROUP`, `CHILD`, `SINGLE`, `ARN`.
     */
    type?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a ResourceConfiguration resource.
 */
export interface ResourceConfigurationArgs {
    /**
     * Allow or Deny the association of this resource to a shareable service network.
     */
    allowAssociationToShareableServiceNetwork?: pulumi.Input<boolean>;
    /**
     * Name for the Resource Configuration.
     */
    name?: pulumi.Input<string>;
    /**
     * Port ranges to access the Resource either single port `80` or range `80-81` range.
     */
    portRanges?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Protocol for the Resource `TCP` is currently the only supported value.  MUST be specified if `resourceConfigurationGroupId` is not.
     */
    protocol?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Details of the Resource Configuration. See `resourceConfigurationDefinition` Block for details.
     *
     * The following arguments are optional:
     */
    resourceConfigurationDefinition?: pulumi.Input<inputs.vpclattice.ResourceConfigurationResourceConfigurationDefinition>;
    /**
     * ID of Resource Configuration where `type` is `CHILD`.
     */
    resourceConfigurationGroupId?: pulumi.Input<string>;
    /**
     * ID of the Resource Gateway used to access the resource. MUST be specified if `resourceConfigurationGroupId` is not.
     */
    resourceGatewayIdentifier?: pulumi.Input<string>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    timeouts?: pulumi.Input<inputs.vpclattice.ResourceConfigurationTimeouts>;
    /**
     * Type of Resource Configuration. Must be one of `GROUP`, `CHILD`, `SINGLE`, `ARN`.
     */
    type?: pulumi.Input<string>;
}
