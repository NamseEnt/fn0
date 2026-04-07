import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages an EC2 Instance Connect Endpoint.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.ec2transitgateway.InstanceConnectEndpoint("example", {subnetId: exampleAwsSubnet.id});
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import EC2 Instance Connect Endpoints using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:ec2transitgateway/instanceConnectEndpoint:InstanceConnectEndpoint example eice-012345678
 * ```
 */
export declare class InstanceConnectEndpoint extends pulumi.CustomResource {
    /**
     * Get an existing InstanceConnectEndpoint resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: InstanceConnectEndpointState, opts?: pulumi.CustomResourceOptions): InstanceConnectEndpoint;
    /**
     * Returns true if the given object is an instance of InstanceConnectEndpoint.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is InstanceConnectEndpoint;
    /**
     * The Amazon Resource Name (ARN) of the EC2 Instance Connect Endpoint.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The Availability Zone of the EC2 Instance Connect Endpoint.
     */
    readonly availabilityZone: pulumi.Output<string>;
    /**
     * The DNS name of the EC2 Instance Connect Endpoint.
     */
    readonly dnsName: pulumi.Output<string>;
    /**
     * The DNS name of the EC2 Instance Connect FIPS Endpoint.
     */
    readonly fipsDnsName: pulumi.Output<string>;
    /**
     * IP address type of the endpoint. Valid values are `ipv4`, `ipv6`, and `dualstack`. The default value is determined by the IP address type of the subnet. See the [AWS documentation](https://docs.aws.amazon.com/AWSEC2/latest/APIReference/API_CreateInstanceConnectEndpoint.html) for more details.
     */
    readonly ipAddressType: pulumi.Output<string>;
    /**
     * The IDs of the ENIs that Amazon EC2 automatically created when creating the EC2 Instance Connect Endpoint.
     */
    readonly networkInterfaceIds: pulumi.Output<string[]>;
    /**
     * The ID of the AWS account that created the EC2 Instance Connect Endpoint.
     */
    readonly ownerId: pulumi.Output<string>;
    /**
     * Indicates whether your client's IP address is preserved as the source. Default: `true`.
     */
    readonly preserveClientIp: pulumi.Output<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * One or more security groups to associate with the endpoint. If you don't specify a security group, the default security group for the VPC will be associated with the endpoint.
     */
    readonly securityGroupIds: pulumi.Output<string[]>;
    /**
     * The ID of the subnet in which to create the EC2 Instance Connect Endpoint.
     */
    readonly subnetId: pulumi.Output<string>;
    /**
     * Map of tags to assign to this resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
    readonly timeouts: pulumi.Output<outputs.ec2transitgateway.InstanceConnectEndpointTimeouts | undefined>;
    /**
     * The ID of the VPC in which the EC2 Instance Connect Endpoint was created.
     */
    readonly vpcId: pulumi.Output<string>;
    /**
     * Create a InstanceConnectEndpoint resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: InstanceConnectEndpointArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering InstanceConnectEndpoint resources.
 */
export interface InstanceConnectEndpointState {
    /**
     * The Amazon Resource Name (ARN) of the EC2 Instance Connect Endpoint.
     */
    arn?: pulumi.Input<string>;
    /**
     * The Availability Zone of the EC2 Instance Connect Endpoint.
     */
    availabilityZone?: pulumi.Input<string>;
    /**
     * The DNS name of the EC2 Instance Connect Endpoint.
     */
    dnsName?: pulumi.Input<string>;
    /**
     * The DNS name of the EC2 Instance Connect FIPS Endpoint.
     */
    fipsDnsName?: pulumi.Input<string>;
    /**
     * IP address type of the endpoint. Valid values are `ipv4`, `ipv6`, and `dualstack`. The default value is determined by the IP address type of the subnet. See the [AWS documentation](https://docs.aws.amazon.com/AWSEC2/latest/APIReference/API_CreateInstanceConnectEndpoint.html) for more details.
     */
    ipAddressType?: pulumi.Input<string>;
    /**
     * The IDs of the ENIs that Amazon EC2 automatically created when creating the EC2 Instance Connect Endpoint.
     */
    networkInterfaceIds?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The ID of the AWS account that created the EC2 Instance Connect Endpoint.
     */
    ownerId?: pulumi.Input<string>;
    /**
     * Indicates whether your client's IP address is preserved as the source. Default: `true`.
     */
    preserveClientIp?: pulumi.Input<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * One or more security groups to associate with the endpoint. If you don't specify a security group, the default security group for the VPC will be associated with the endpoint.
     */
    securityGroupIds?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The ID of the subnet in which to create the EC2 Instance Connect Endpoint.
     */
    subnetId?: pulumi.Input<string>;
    /**
     * Map of tags to assign to this resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
    timeouts?: pulumi.Input<inputs.ec2transitgateway.InstanceConnectEndpointTimeouts>;
    /**
     * The ID of the VPC in which the EC2 Instance Connect Endpoint was created.
     */
    vpcId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a InstanceConnectEndpoint resource.
 */
export interface InstanceConnectEndpointArgs {
    /**
     * IP address type of the endpoint. Valid values are `ipv4`, `ipv6`, and `dualstack`. The default value is determined by the IP address type of the subnet. See the [AWS documentation](https://docs.aws.amazon.com/AWSEC2/latest/APIReference/API_CreateInstanceConnectEndpoint.html) for more details.
     */
    ipAddressType?: pulumi.Input<string>;
    /**
     * Indicates whether your client's IP address is preserved as the source. Default: `true`.
     */
    preserveClientIp?: pulumi.Input<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * One or more security groups to associate with the endpoint. If you don't specify a security group, the default security group for the VPC will be associated with the endpoint.
     */
    securityGroupIds?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The ID of the subnet in which to create the EC2 Instance Connect Endpoint.
     */
    subnetId: pulumi.Input<string>;
    /**
     * Map of tags to assign to this resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    timeouts?: pulumi.Input<inputs.ec2transitgateway.InstanceConnectEndpointTimeouts>;
}
