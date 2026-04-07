import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Grant cross-account access to an Elastic network interface (ENI).
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.ec2.NetworkInterface("example", {
 *     subnetId: exampleAwsSubnet.id,
 *     privateIps: ["10.0.0.50"],
 *     securityGroups: [exampleAwsSecurityGroup.id],
 *     attachments: [{
 *         instance: exampleAwsInstance.id,
 *         deviceIndex: 1,
 *     }],
 * });
 * const exampleNetworkInterfacePermission = new aws.ec2.NetworkInterfacePermission("example", {
 *     networkInterfaceId: example.id,
 *     awsAccountId: "123456789012",
 *     permission: "INSTANCE-ATTACH",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Network Interface Permissions using the `network_interface_permission_id`. For example:
 *
 * ```sh
 * $ pulumi import aws:ec2/networkInterfacePermission:NetworkInterfacePermission example eni-perm-056ad97ce2ac377ed
 * ```
 */
export declare class NetworkInterfacePermission extends pulumi.CustomResource {
    /**
     * Get an existing NetworkInterfacePermission resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: NetworkInterfacePermissionState, opts?: pulumi.CustomResourceOptions): NetworkInterfacePermission;
    /**
     * Returns true if the given object is an instance of NetworkInterfacePermission.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is NetworkInterfacePermission;
    /**
     * The Amazon Web Services account ID.
     */
    readonly awsAccountId: pulumi.Output<string>;
    /**
     * The ID of the network interface.
     */
    readonly networkInterfaceId: pulumi.Output<string>;
    /**
     * ENI permission ID.
     */
    readonly networkInterfacePermissionId: pulumi.Output<string>;
    /**
     * The type of permission to grant. Valid values are `INSTANCE-ATTACH` or `EIP-ASSOCIATE`.
     */
    readonly permission: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    readonly timeouts: pulumi.Output<outputs.ec2.NetworkInterfacePermissionTimeouts | undefined>;
    /**
     * Create a NetworkInterfacePermission resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: NetworkInterfacePermissionArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering NetworkInterfacePermission resources.
 */
export interface NetworkInterfacePermissionState {
    /**
     * The Amazon Web Services account ID.
     */
    awsAccountId?: pulumi.Input<string>;
    /**
     * The ID of the network interface.
     */
    networkInterfaceId?: pulumi.Input<string>;
    /**
     * ENI permission ID.
     */
    networkInterfacePermissionId?: pulumi.Input<string>;
    /**
     * The type of permission to grant. Valid values are `INSTANCE-ATTACH` or `EIP-ASSOCIATE`.
     */
    permission?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    timeouts?: pulumi.Input<inputs.ec2.NetworkInterfacePermissionTimeouts>;
}
/**
 * The set of arguments for constructing a NetworkInterfacePermission resource.
 */
export interface NetworkInterfacePermissionArgs {
    /**
     * The Amazon Web Services account ID.
     */
    awsAccountId: pulumi.Input<string>;
    /**
     * The ID of the network interface.
     */
    networkInterfaceId: pulumi.Input<string>;
    /**
     * The type of permission to grant. Valid values are `INSTANCE-ATTACH` or `EIP-ASSOCIATE`.
     */
    permission: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    timeouts?: pulumi.Input<inputs.ec2.NetworkInterfacePermissionTimeouts>;
}
