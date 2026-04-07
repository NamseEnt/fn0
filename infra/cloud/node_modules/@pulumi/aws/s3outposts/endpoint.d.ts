import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a resource to manage an S3 Outposts Endpoint.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.s3outposts.Endpoint("example", {
 *     outpostId: exampleAwsOutpostsOutpost.id,
 *     securityGroupId: exampleAwsSecurityGroup.id,
 *     subnetId: exampleAwsSubnet.id,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import S3 Outposts Endpoints using Amazon Resource Name (ARN), EC2 Security Group identifier, and EC2 Subnet identifier, separated by commas (`,`). For example:
 *
 * ```sh
 * $ pulumi import aws:s3outposts/endpoint:Endpoint example arn:aws:s3-outposts:us-east-1:123456789012:outpost/op-12345678/endpoint/0123456789abcdef,sg-12345678,subnet-12345678
 * ```
 */
export declare class Endpoint extends pulumi.CustomResource {
    /**
     * Get an existing Endpoint resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: EndpointState, opts?: pulumi.CustomResourceOptions): Endpoint;
    /**
     * Returns true if the given object is an instance of Endpoint.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Endpoint;
    /**
     * Type of access for the network connectivity. Valid values are `Private` or `CustomerOwnedIp`.
     */
    readonly accessType: pulumi.Output<string>;
    /**
     * Amazon Resource Name (ARN) of the endpoint.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * VPC CIDR block of the endpoint.
     */
    readonly cidrBlock: pulumi.Output<string>;
    /**
     * UTC creation time in [RFC3339 format](https://tools.ietf.org/html/rfc3339#section-5.8).
     */
    readonly creationTime: pulumi.Output<string>;
    /**
     * The ID of a Customer Owned IP Pool. For more on customer owned IP addresses see the [User Guide](https://docs.aws.amazon.com/outposts/latest/userguide/local-rack.html#local-gateway-subnet).
     */
    readonly customerOwnedIpv4Pool: pulumi.Output<string | undefined>;
    /**
     * Set of nested attributes for associated Elastic Network Interfaces (ENIs).
     */
    readonly networkInterfaces: pulumi.Output<outputs.s3outposts.EndpointNetworkInterface[]>;
    /**
     * Identifier of the Outpost to contain this endpoint.
     */
    readonly outpostId: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Identifier of the EC2 Security Group.
     */
    readonly securityGroupId: pulumi.Output<string>;
    /**
     * Identifier of the EC2 Subnet.
     */
    readonly subnetId: pulumi.Output<string>;
    /**
     * Create a Endpoint resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: EndpointArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Endpoint resources.
 */
export interface EndpointState {
    /**
     * Type of access for the network connectivity. Valid values are `Private` or `CustomerOwnedIp`.
     */
    accessType?: pulumi.Input<string>;
    /**
     * Amazon Resource Name (ARN) of the endpoint.
     */
    arn?: pulumi.Input<string>;
    /**
     * VPC CIDR block of the endpoint.
     */
    cidrBlock?: pulumi.Input<string>;
    /**
     * UTC creation time in [RFC3339 format](https://tools.ietf.org/html/rfc3339#section-5.8).
     */
    creationTime?: pulumi.Input<string>;
    /**
     * The ID of a Customer Owned IP Pool. For more on customer owned IP addresses see the [User Guide](https://docs.aws.amazon.com/outposts/latest/userguide/local-rack.html#local-gateway-subnet).
     */
    customerOwnedIpv4Pool?: pulumi.Input<string>;
    /**
     * Set of nested attributes for associated Elastic Network Interfaces (ENIs).
     */
    networkInterfaces?: pulumi.Input<pulumi.Input<inputs.s3outposts.EndpointNetworkInterface>[]>;
    /**
     * Identifier of the Outpost to contain this endpoint.
     */
    outpostId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Identifier of the EC2 Security Group.
     */
    securityGroupId?: pulumi.Input<string>;
    /**
     * Identifier of the EC2 Subnet.
     */
    subnetId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a Endpoint resource.
 */
export interface EndpointArgs {
    /**
     * Type of access for the network connectivity. Valid values are `Private` or `CustomerOwnedIp`.
     */
    accessType?: pulumi.Input<string>;
    /**
     * The ID of a Customer Owned IP Pool. For more on customer owned IP addresses see the [User Guide](https://docs.aws.amazon.com/outposts/latest/userguide/local-rack.html#local-gateway-subnet).
     */
    customerOwnedIpv4Pool?: pulumi.Input<string>;
    /**
     * Identifier of the Outpost to contain this endpoint.
     */
    outpostId: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Identifier of the EC2 Security Group.
     */
    securityGroupId: pulumi.Input<string>;
    /**
     * Identifier of the EC2 Subnet.
     */
    subnetId: pulumi.Input<string>;
}
