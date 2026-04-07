import * as pulumi from "@pulumi/pulumi";
/**
 * Provides an Elastic IP resource.
 *
 * > **Note:** EIP may require IGW to exist prior to association. Use `dependsOn` to set an explicit dependency on the IGW.
 *
 * > **Note:** Do not use `networkInterface` to associate the EIP to `aws.lb.LoadBalancer` or `aws.ec2.NatGateway` resources. Instead use the `allocationId` available in those resources to allow AWS to manage the association, otherwise you will see `AuthFailure` errors.
 *
 * ## Example Usage
 *
 * ### Single EIP associated with an instance
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const lb = new aws.ec2.Eip("lb", {
 *     instance: web.id,
 *     domain: "vpc",
 * });
 * ```
 *
 * ### Multiple EIPs associated with a single network interface
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const multi_ip = new aws.ec2.NetworkInterface("multi-ip", {
 *     subnetId: main.id,
 *     privateIps: [
 *         "10.0.0.10",
 *         "10.0.0.11",
 *     ],
 * });
 * const one = new aws.ec2.Eip("one", {
 *     domain: "vpc",
 *     networkInterface: multi_ip.id,
 *     associateWithPrivateIp: "10.0.0.10",
 * });
 * const two = new aws.ec2.Eip("two", {
 *     domain: "vpc",
 *     networkInterface: multi_ip.id,
 *     associateWithPrivateIp: "10.0.0.11",
 * });
 * ```
 *
 * ### Attaching an EIP to an Instance with a pre-assigned private ip (VPC Only)
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const _default = new aws.ec2.Vpc("default", {
 *     cidrBlock: "10.0.0.0/16",
 *     enableDnsHostnames: true,
 * });
 * const gw = new aws.ec2.InternetGateway("gw", {vpcId: _default.id});
 * const myTestSubnet = new aws.ec2.Subnet("my_test_subnet", {
 *     vpcId: _default.id,
 *     cidrBlock: "10.0.0.0/24",
 *     mapPublicIpOnLaunch: true,
 * }, {
 *     dependsOn: [gw],
 * });
 * const foo = new aws.ec2.Instance("foo", {
 *     ami: "ami-5189a661",
 *     instanceType: aws.ec2.InstanceType.T2_Micro,
 *     privateIp: "10.0.0.12",
 *     subnetId: myTestSubnet.id,
 * });
 * const bar = new aws.ec2.Eip("bar", {
 *     domain: "vpc",
 *     instance: foo.id,
 *     associateWithPrivateIp: "10.0.0.12",
 * }, {
 *     dependsOn: [gw],
 * });
 * ```
 *
 * ### Allocating EIP from the BYOIP pool
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const byoip_ip = new aws.ec2.Eip("byoip-ip", {
 *     domain: "vpc",
 *     publicIpv4Pool: "ipv4pool-ec2-012345",
 * });
 * ```
 *
 * ### Allocating EIP from the IPAM Pool
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const ipam_ip = new aws.ec2.Eip("ipam-ip", {
 *     domain: "vpc",
 *     ipamPoolId: "ipam-pool-07ccc86aa41bef7ce",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import EIPs in a VPC using their Allocation ID. For example:
 *
 * ```sh
 * $ pulumi import aws:ec2/eip:Eip bar eipalloc-00a10e96
 * ```
 */
export declare class Eip extends pulumi.CustomResource {
    /**
     * Get an existing Eip resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: EipState, opts?: pulumi.CustomResourceOptions): Eip;
    /**
     * Returns true if the given object is an instance of Eip.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Eip;
    /**
     * IP address from an EC2 BYOIP pool. This option is only available for VPC EIPs.
     */
    readonly address: pulumi.Output<string | undefined>;
    /**
     * ID that AWS assigns to represent the allocation of the Elastic IP address for use with instances in a VPC.
     */
    readonly allocationId: pulumi.Output<string>;
    readonly arn: pulumi.Output<string>;
    /**
     * User-specified primary or secondary private IP address to associate with the Elastic IP address. If no private IP address is specified, the Elastic IP address is associated with the primary private IP address.
     */
    readonly associateWithPrivateIp: pulumi.Output<string | undefined>;
    /**
     * ID representing the association of the address with an instance in a VPC.
     */
    readonly associationId: pulumi.Output<string>;
    /**
     * Carrier IP address.
     */
    readonly carrierIp: pulumi.Output<string>;
    /**
     * Customer owned IP.
     */
    readonly customerOwnedIp: pulumi.Output<string>;
    /**
     * ID  of a customer-owned address pool. For more on customer owned IP addressed check out [Customer-owned IP addresses guide](https://docs.aws.amazon.com/outposts/latest/userguide/outposts-networking-components.html#ip-addressing).
     */
    readonly customerOwnedIpv4Pool: pulumi.Output<string | undefined>;
    /**
     * Indicates if this EIP is for use in VPC (`vpc`).
     */
    readonly domain: pulumi.Output<string>;
    /**
     * EC2 instance ID.
     */
    readonly instance: pulumi.Output<string>;
    /**
     * The ID of an IPAM pool which has an Amazon-provided or BYOIP public IPv4 CIDR provisioned to it.
     */
    readonly ipamPoolId: pulumi.Output<string>;
    /**
     * Location from which the IP address is advertised. Use this parameter to limit the address to this location.
     */
    readonly networkBorderGroup: pulumi.Output<string>;
    /**
     * Network interface ID to associate with.
     */
    readonly networkInterface: pulumi.Output<string>;
    /**
     * The Private DNS associated with the Elastic IP address (if in VPC).
     */
    readonly privateDns: pulumi.Output<string>;
    /**
     * Contains the private IP address (if in VPC).
     */
    readonly privateIp: pulumi.Output<string>;
    /**
     * The DNS pointer (PTR) record for the IP address.
     */
    readonly ptrRecord: pulumi.Output<string>;
    /**
     * Public DNS associated with the Elastic IP address.
     */
    readonly publicDns: pulumi.Output<string>;
    /**
     * Contains the public IP address.
     */
    readonly publicIp: pulumi.Output<string>;
    /**
     * EC2 IPv4 address pool identifier or `amazon`.
     * This option is only available for VPC EIPs.
     */
    readonly publicIpv4Pool: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Map of tags to assign to the resource. Tags can only be applied to EIPs in a VPC. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     *
     * > **NOTE:** You can specify either the `instance` ID or the `networkInterface` ID, but not both.
     * Including both will **not** return an error from the AWS API, but will have undefined behavior.
     * See the relevant [AssociateAddress API Call][1] for more information.
     *
     * > **NOTE:** Specifying both `publicIpv4Pool` and `address` won't cause an error, however, only `address` will be used if both options are defined as the API only requires one of the two.
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
     * Create a Eip resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: EipArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Eip resources.
 */
export interface EipState {
    /**
     * IP address from an EC2 BYOIP pool. This option is only available for VPC EIPs.
     */
    address?: pulumi.Input<string>;
    /**
     * ID that AWS assigns to represent the allocation of the Elastic IP address for use with instances in a VPC.
     */
    allocationId?: pulumi.Input<string>;
    arn?: pulumi.Input<string>;
    /**
     * User-specified primary or secondary private IP address to associate with the Elastic IP address. If no private IP address is specified, the Elastic IP address is associated with the primary private IP address.
     */
    associateWithPrivateIp?: pulumi.Input<string>;
    /**
     * ID representing the association of the address with an instance in a VPC.
     */
    associationId?: pulumi.Input<string>;
    /**
     * Carrier IP address.
     */
    carrierIp?: pulumi.Input<string>;
    /**
     * Customer owned IP.
     */
    customerOwnedIp?: pulumi.Input<string>;
    /**
     * ID  of a customer-owned address pool. For more on customer owned IP addressed check out [Customer-owned IP addresses guide](https://docs.aws.amazon.com/outposts/latest/userguide/outposts-networking-components.html#ip-addressing).
     */
    customerOwnedIpv4Pool?: pulumi.Input<string>;
    /**
     * Indicates if this EIP is for use in VPC (`vpc`).
     */
    domain?: pulumi.Input<string>;
    /**
     * EC2 instance ID.
     */
    instance?: pulumi.Input<string>;
    /**
     * The ID of an IPAM pool which has an Amazon-provided or BYOIP public IPv4 CIDR provisioned to it.
     */
    ipamPoolId?: pulumi.Input<string>;
    /**
     * Location from which the IP address is advertised. Use this parameter to limit the address to this location.
     */
    networkBorderGroup?: pulumi.Input<string>;
    /**
     * Network interface ID to associate with.
     */
    networkInterface?: pulumi.Input<string>;
    /**
     * The Private DNS associated with the Elastic IP address (if in VPC).
     */
    privateDns?: pulumi.Input<string>;
    /**
     * Contains the private IP address (if in VPC).
     */
    privateIp?: pulumi.Input<string>;
    /**
     * The DNS pointer (PTR) record for the IP address.
     */
    ptrRecord?: pulumi.Input<string>;
    /**
     * Public DNS associated with the Elastic IP address.
     */
    publicDns?: pulumi.Input<string>;
    /**
     * Contains the public IP address.
     */
    publicIp?: pulumi.Input<string>;
    /**
     * EC2 IPv4 address pool identifier or `amazon`.
     * This option is only available for VPC EIPs.
     */
    publicIpv4Pool?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Map of tags to assign to the resource. Tags can only be applied to EIPs in a VPC. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     *
     * > **NOTE:** You can specify either the `instance` ID or the `networkInterface` ID, but not both.
     * Including both will **not** return an error from the AWS API, but will have undefined behavior.
     * See the relevant [AssociateAddress API Call][1] for more information.
     *
     * > **NOTE:** Specifying both `publicIpv4Pool` and `address` won't cause an error, however, only `address` will be used if both options are defined as the API only requires one of the two.
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
 * The set of arguments for constructing a Eip resource.
 */
export interface EipArgs {
    /**
     * IP address from an EC2 BYOIP pool. This option is only available for VPC EIPs.
     */
    address?: pulumi.Input<string>;
    /**
     * User-specified primary or secondary private IP address to associate with the Elastic IP address. If no private IP address is specified, the Elastic IP address is associated with the primary private IP address.
     */
    associateWithPrivateIp?: pulumi.Input<string>;
    /**
     * ID  of a customer-owned address pool. For more on customer owned IP addressed check out [Customer-owned IP addresses guide](https://docs.aws.amazon.com/outposts/latest/userguide/outposts-networking-components.html#ip-addressing).
     */
    customerOwnedIpv4Pool?: pulumi.Input<string>;
    /**
     * Indicates if this EIP is for use in VPC (`vpc`).
     */
    domain?: pulumi.Input<string>;
    /**
     * EC2 instance ID.
     */
    instance?: pulumi.Input<string>;
    /**
     * The ID of an IPAM pool which has an Amazon-provided or BYOIP public IPv4 CIDR provisioned to it.
     */
    ipamPoolId?: pulumi.Input<string>;
    /**
     * Location from which the IP address is advertised. Use this parameter to limit the address to this location.
     */
    networkBorderGroup?: pulumi.Input<string>;
    /**
     * Network interface ID to associate with.
     */
    networkInterface?: pulumi.Input<string>;
    /**
     * EC2 IPv4 address pool identifier or `amazon`.
     * This option is only available for VPC EIPs.
     */
    publicIpv4Pool?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Map of tags to assign to the resource. Tags can only be applied to EIPs in a VPC. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     *
     * > **NOTE:** You can specify either the `instance` ID or the `networkInterface` ID, but not both.
     * Including both will **not** return an error from the AWS API, but will have undefined behavior.
     * See the relevant [AssociateAddress API Call][1] for more information.
     *
     * > **NOTE:** Specifying both `publicIpv4Pool` and `address` won't cause an error, however, only `address` will be used if both options are defined as the API only requires one of the two.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
