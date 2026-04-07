import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides an AWS Network Firewall Firewall Resource
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.networkfirewall.Firewall("example", {
 *     name: "example",
 *     firewallPolicyArn: exampleAwsNetworkfirewallFirewallPolicy.arn,
 *     vpcId: exampleAwsVpc.id,
 *     enabledAnalysisTypes: [
 *         "TLS_SNI",
 *         "HTTP_HOST",
 *     ],
 *     subnetMappings: [{
 *         subnetId: exampleAwsSubnet.id,
 *     }],
 *     tags: {
 *         Tag1: "Value1",
 *         Tag2: "Value2",
 *     },
 * });
 * ```
 *
 * ### Transit Gateway Attached Firewall
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.getAvailabilityZones({
 *     state: "available",
 * });
 * const exampleFirewall = new aws.networkfirewall.Firewall("example", {
 *     name: "example",
 *     firewallPolicyArn: exampleAwsNetworkfirewallFirewallPolicy.arn,
 *     transitGatewayId: exampleAwsEc2TransitGateway.id,
 *     availabilityZoneMappings: [
 *         {
 *             availabilityZoneId: example.then(example => example.zoneIds?.[0]),
 *         },
 *         {
 *             availabilityZoneId: example.then(example => example.zoneIds?.[1]),
 *         },
 *     ],
 * });
 * ```
 *
 * ### Transit Gateway Attached Firewall (Cross Account)
 *
 * A full example of how to create a Transit Gateway in one AWS account, share it with a second AWS account, and create Network Firewall in the second account to the Transit Gateway via the `aws.networkfirewall.Firewall` and `awsNetworkfirewallNetworkFirewallTransitGatewayAttachmentAccepter` resources can be found in the `./examples/network-firewall-cross-account-transit-gateway` directory within the Github Repository
 *
 * ## Import
 *
 * Using `pulumi import`, import Network Firewall Firewalls using their `arn`. For example:
 *
 * ```sh
 * $ pulumi import aws:networkfirewall/firewall:Firewall example arn:aws:network-firewall:us-west-1:123456789012:firewall/example
 * ```
 */
export declare class Firewall extends pulumi.CustomResource {
    /**
     * Get an existing Firewall resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: FirewallState, opts?: pulumi.CustomResourceOptions): Firewall;
    /**
     * Returns true if the given object is an instance of Firewall.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Firewall;
    /**
     * The Amazon Resource Name (ARN) that identifies the firewall.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * A setting indicating whether the firewall is protected against changes to its Availability Zone configuration. When set to `true`, you must first disable this protection before adding or removing Availability Zones.
     */
    readonly availabilityZoneChangeProtection: pulumi.Output<boolean | undefined>;
    /**
     * Required when creating a transit gateway-attached firewall. Set of configuration blocks describing the avaiability availability where you want to create firewall endpoints for a transit gateway-attached firewall.
     */
    readonly availabilityZoneMappings: pulumi.Output<outputs.networkfirewall.FirewallAvailabilityZoneMapping[]>;
    /**
     * A flag indicating whether the firewall is protected against deletion. Use this setting to protect against accidentally deleting a firewall that is in use. Defaults to `false`.
     */
    readonly deleteProtection: pulumi.Output<boolean | undefined>;
    /**
     * A friendly description of the firewall.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * Set of types for which to collect analysis metrics. See [Reporting on network traffic in Network Firewall](https://docs.aws.amazon.com/network-firewall/latest/developerguide/reporting.html) for details on how to use the data. Valid values: `TLS_SNI`, `HTTP_HOST`. Defaults to `[]`.
     */
    readonly enabledAnalysisTypes: pulumi.Output<string[] | undefined>;
    /**
     * KMS encryption configuration settings. See Encryption Configuration below for details.
     */
    readonly encryptionConfiguration: pulumi.Output<outputs.networkfirewall.FirewallEncryptionConfiguration | undefined>;
    /**
     * The Amazon Resource Name (ARN) of the VPC Firewall policy.
     */
    readonly firewallPolicyArn: pulumi.Output<string>;
    /**
     * A flag indicating whether the firewall is protected against a change to the firewall policy association. Use this setting to protect against accidentally modifying the firewall policy for a firewall that is in use. Defaults to `false`.
     */
    readonly firewallPolicyChangeProtection: pulumi.Output<boolean | undefined>;
    /**
     * Nested list of information about the current status of the firewall.
     */
    readonly firewallStatuses: pulumi.Output<outputs.networkfirewall.FirewallFirewallStatus[]>;
    /**
     * A friendly name of the firewall.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * A flag indicating whether the firewall is protected against changes to the subnet associations. Use this setting to protect against accidentally modifying the subnet associations for a firewall that is in use. Defaults to `false`.
     */
    readonly subnetChangeProtection: pulumi.Output<boolean | undefined>;
    /**
     * Required when creating a VPC attached firewall. Set of configuration blocks describing the public subnets. Each subnet must belong to a different Availability Zone in the VPC. AWS Network Firewall creates a firewall endpoint in each subnet. See Subnet Mapping below for details.
     */
    readonly subnetMappings: pulumi.Output<outputs.networkfirewall.FirewallSubnetMapping[] | undefined>;
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
    /**
     * . Required when creating a transit gateway-attached firewall. The unique identifier of the transit gateway to attach to this firewall. You can provide either a transit gateway from your account or one that has been shared with you through AWS Resource Access Manager
     */
    readonly transitGatewayId: pulumi.Output<string | undefined>;
    /**
     * The AWS account ID that owns the transit gateway.
     */
    readonly transitGatewayOwnerAccountId: pulumi.Output<string>;
    /**
     * A string token used when updating a firewall.
     */
    readonly updateToken: pulumi.Output<string>;
    /**
     * Required when creating a VPC attached firewall. The unique identifier of the VPC where AWS Network Firewall should create the firewall.
     */
    readonly vpcId: pulumi.Output<string | undefined>;
    /**
     * Create a Firewall resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: FirewallArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Firewall resources.
 */
export interface FirewallState {
    /**
     * The Amazon Resource Name (ARN) that identifies the firewall.
     */
    arn?: pulumi.Input<string>;
    /**
     * A setting indicating whether the firewall is protected against changes to its Availability Zone configuration. When set to `true`, you must first disable this protection before adding or removing Availability Zones.
     */
    availabilityZoneChangeProtection?: pulumi.Input<boolean>;
    /**
     * Required when creating a transit gateway-attached firewall. Set of configuration blocks describing the avaiability availability where you want to create firewall endpoints for a transit gateway-attached firewall.
     */
    availabilityZoneMappings?: pulumi.Input<pulumi.Input<inputs.networkfirewall.FirewallAvailabilityZoneMapping>[]>;
    /**
     * A flag indicating whether the firewall is protected against deletion. Use this setting to protect against accidentally deleting a firewall that is in use. Defaults to `false`.
     */
    deleteProtection?: pulumi.Input<boolean>;
    /**
     * A friendly description of the firewall.
     */
    description?: pulumi.Input<string>;
    /**
     * Set of types for which to collect analysis metrics. See [Reporting on network traffic in Network Firewall](https://docs.aws.amazon.com/network-firewall/latest/developerguide/reporting.html) for details on how to use the data. Valid values: `TLS_SNI`, `HTTP_HOST`. Defaults to `[]`.
     */
    enabledAnalysisTypes?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * KMS encryption configuration settings. See Encryption Configuration below for details.
     */
    encryptionConfiguration?: pulumi.Input<inputs.networkfirewall.FirewallEncryptionConfiguration>;
    /**
     * The Amazon Resource Name (ARN) of the VPC Firewall policy.
     */
    firewallPolicyArn?: pulumi.Input<string>;
    /**
     * A flag indicating whether the firewall is protected against a change to the firewall policy association. Use this setting to protect against accidentally modifying the firewall policy for a firewall that is in use. Defaults to `false`.
     */
    firewallPolicyChangeProtection?: pulumi.Input<boolean>;
    /**
     * Nested list of information about the current status of the firewall.
     */
    firewallStatuses?: pulumi.Input<pulumi.Input<inputs.networkfirewall.FirewallFirewallStatus>[]>;
    /**
     * A friendly name of the firewall.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A flag indicating whether the firewall is protected against changes to the subnet associations. Use this setting to protect against accidentally modifying the subnet associations for a firewall that is in use. Defaults to `false`.
     */
    subnetChangeProtection?: pulumi.Input<boolean>;
    /**
     * Required when creating a VPC attached firewall. Set of configuration blocks describing the public subnets. Each subnet must belong to a different Availability Zone in the VPC. AWS Network Firewall creates a firewall endpoint in each subnet. See Subnet Mapping below for details.
     */
    subnetMappings?: pulumi.Input<pulumi.Input<inputs.networkfirewall.FirewallSubnetMapping>[]>;
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
    /**
     * . Required when creating a transit gateway-attached firewall. The unique identifier of the transit gateway to attach to this firewall. You can provide either a transit gateway from your account or one that has been shared with you through AWS Resource Access Manager
     */
    transitGatewayId?: pulumi.Input<string>;
    /**
     * The AWS account ID that owns the transit gateway.
     */
    transitGatewayOwnerAccountId?: pulumi.Input<string>;
    /**
     * A string token used when updating a firewall.
     */
    updateToken?: pulumi.Input<string>;
    /**
     * Required when creating a VPC attached firewall. The unique identifier of the VPC where AWS Network Firewall should create the firewall.
     */
    vpcId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a Firewall resource.
 */
export interface FirewallArgs {
    /**
     * A setting indicating whether the firewall is protected against changes to its Availability Zone configuration. When set to `true`, you must first disable this protection before adding or removing Availability Zones.
     */
    availabilityZoneChangeProtection?: pulumi.Input<boolean>;
    /**
     * Required when creating a transit gateway-attached firewall. Set of configuration blocks describing the avaiability availability where you want to create firewall endpoints for a transit gateway-attached firewall.
     */
    availabilityZoneMappings?: pulumi.Input<pulumi.Input<inputs.networkfirewall.FirewallAvailabilityZoneMapping>[]>;
    /**
     * A flag indicating whether the firewall is protected against deletion. Use this setting to protect against accidentally deleting a firewall that is in use. Defaults to `false`.
     */
    deleteProtection?: pulumi.Input<boolean>;
    /**
     * A friendly description of the firewall.
     */
    description?: pulumi.Input<string>;
    /**
     * Set of types for which to collect analysis metrics. See [Reporting on network traffic in Network Firewall](https://docs.aws.amazon.com/network-firewall/latest/developerguide/reporting.html) for details on how to use the data. Valid values: `TLS_SNI`, `HTTP_HOST`. Defaults to `[]`.
     */
    enabledAnalysisTypes?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * KMS encryption configuration settings. See Encryption Configuration below for details.
     */
    encryptionConfiguration?: pulumi.Input<inputs.networkfirewall.FirewallEncryptionConfiguration>;
    /**
     * The Amazon Resource Name (ARN) of the VPC Firewall policy.
     */
    firewallPolicyArn: pulumi.Input<string>;
    /**
     * A flag indicating whether the firewall is protected against a change to the firewall policy association. Use this setting to protect against accidentally modifying the firewall policy for a firewall that is in use. Defaults to `false`.
     */
    firewallPolicyChangeProtection?: pulumi.Input<boolean>;
    /**
     * A friendly name of the firewall.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A flag indicating whether the firewall is protected against changes to the subnet associations. Use this setting to protect against accidentally modifying the subnet associations for a firewall that is in use. Defaults to `false`.
     */
    subnetChangeProtection?: pulumi.Input<boolean>;
    /**
     * Required when creating a VPC attached firewall. Set of configuration blocks describing the public subnets. Each subnet must belong to a different Availability Zone in the VPC. AWS Network Firewall creates a firewall endpoint in each subnet. See Subnet Mapping below for details.
     */
    subnetMappings?: pulumi.Input<pulumi.Input<inputs.networkfirewall.FirewallSubnetMapping>[]>;
    /**
     * Map of resource tags to associate with the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * . Required when creating a transit gateway-attached firewall. The unique identifier of the transit gateway to attach to this firewall. You can provide either a transit gateway from your account or one that has been shared with you through AWS Resource Access Manager
     */
    transitGatewayId?: pulumi.Input<string>;
    /**
     * Required when creating a VPC attached firewall. The unique identifier of the VPC where AWS Network Firewall should create the firewall.
     */
    vpcId?: pulumi.Input<string>;
}
