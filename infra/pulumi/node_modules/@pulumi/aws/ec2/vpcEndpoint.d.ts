import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a VPC Endpoint resource.
 *
 * > **NOTE on VPC Endpoints and VPC Endpoint Associations:** The provider provides both standalone VPC Endpoint Associations for
 * Route Tables - (an association between a VPC endpoint and a single `routeTableId`),
 * Security Groups - (an association between a VPC endpoint and a single `securityGroupId`),
 * and Subnets - (an association between a VPC endpoint and a single `subnetId`) and
 * a VPC Endpoint resource with `routeTableIds` and `subnetIds` attributes.
 * Do not use the same resource ID in both a VPC Endpoint resource and a VPC Endpoint Association resource.
 * Doing so will cause a conflict of associations and will overwrite the association.
 *
 * ## Example Usage
 *
 * ### Basic
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const s3 = new aws.ec2.VpcEndpoint("s3", {
 *     vpcId: main.id,
 *     serviceName: "com.amazonaws.us-west-2.s3",
 * });
 * ```
 *
 * ### Basic w/ Tags
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const s3 = new aws.ec2.VpcEndpoint("s3", {
 *     vpcId: main.id,
 *     serviceName: "com.amazonaws.us-west-2.s3",
 *     tags: {
 *         Environment: "test",
 *     },
 * });
 * ```
 *
 * ### Interface Endpoint Type
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const ec2 = new aws.ec2.VpcEndpoint("ec2", {
 *     vpcId: main.id,
 *     serviceName: "com.amazonaws.us-west-2.ec2",
 *     vpcEndpointType: "Interface",
 *     securityGroupIds: [sg1.id],
 *     privateDnsEnabled: true,
 * });
 * ```
 *
 * ### Interface Endpoint Type with User-Defined IP Address
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const ec2 = new aws.ec2.VpcEndpoint("ec2", {
 *     vpcId: example.id,
 *     serviceName: "com.amazonaws.us-west-2.ec2",
 *     vpcEndpointType: "Interface",
 *     subnetConfigurations: [
 *         {
 *             ipv4: "10.0.1.10",
 *             subnetId: example1.id,
 *         },
 *         {
 *             ipv4: "10.0.2.10",
 *             subnetId: example2.id,
 *         },
 *     ],
 *     subnetIds: [
 *         example1.id,
 *         example2.id,
 *     ],
 * });
 * ```
 *
 * ### Gateway Load Balancer Endpoint Type
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const current = aws.getCallerIdentity({});
 * const example = new aws.ec2.VpcEndpointService("example", {
 *     acceptanceRequired: false,
 *     allowedPrincipals: [current.then(current => current.arn)],
 *     gatewayLoadBalancerArns: [exampleAwsLb.arn],
 * });
 * const exampleVpcEndpoint = new aws.ec2.VpcEndpoint("example", {
 *     serviceName: example.serviceName,
 *     subnetIds: [exampleAwsSubnet.id],
 *     vpcEndpointType: example.serviceType,
 *     vpcId: exampleAwsVpc.id,
 * });
 * ```
 *
 * ### VPC Lattice Resource Configuration Endpoint Type
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.ec2.VpcEndpoint("example", {
 *     resourceConfigurationArn: exampleAwsVpclatticeResourceConfiguration.arn,
 *     subnetIds: [exampleAwsSubnet.id],
 *     vpcEndpointType: "Resource",
 *     vpcId: exampleAwsVpc.id,
 * });
 * ```
 *
 * ### VPC Lattice Service Network Endpoint Type
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.ec2.VpcEndpoint("example", {
 *     serviceNetworkArn: exampleAwsVpclatticeServiceNetwork.arn,
 *     subnetIds: [exampleAwsSubnet.id],
 *     vpcEndpointType: "ServiceNetwork",
 *     vpcId: exampleAwsVpc.id,
 * });
 * ```
 *
 * ### Non-AWS Service
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const ptfeService = new aws.ec2.VpcEndpoint("ptfe_service", {
 *     vpcId: vpcId,
 *     serviceName: ptfeServiceConfig,
 *     vpcEndpointType: "Interface",
 *     securityGroupIds: [ptfeServiceAwsSecurityGroup.id],
 *     subnetIds: [subnetIds],
 *     privateDnsEnabled: false,
 * });
 * const internal = aws.route53.getZone({
 *     name: "vpc.internal.",
 *     privateZone: true,
 *     vpcId: vpcId,
 * });
 * const ptfeServiceRecord = new aws.route53.Record("ptfe_service", {
 *     zoneId: internal.then(internal => internal.zoneId),
 *     name: internal.then(internal => `ptfe.${internal.name}`),
 *     type: aws.route53.RecordType.CNAME,
 *     ttl: 300,
 *     records: [ptfeService.dnsEntries[0].dns_name],
 * });
 * ```
 *
 * > **NOTE The `dnsEntry` output is a list of maps:** This provider interpolation support for lists of maps requires the `lookup` and `[]` until full support of lists of maps is available
 *
 * ## Import
 *
 * ### Identity Schema
 *
 * #### Required
 *
 * * `id` - (String) ID of the VPC endpoint.
 *
 * #### Optional
 *
 * * `account_id` (String) AWS Account where this resource is managed.
 *
 * * `region` (String) Region where this resource is managed.
 *
 * Using `pulumi import`, import VPC Endpoints using the VPC endpoint `id`. For example:
 *
 * console
 *
 * % pulumi import aws_vpc_endpoint.example vpce-3ecf2a57
 */
export declare class VpcEndpoint extends pulumi.CustomResource {
    /**
     * Get an existing VpcEndpoint resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: VpcEndpointState, opts?: pulumi.CustomResourceOptions): VpcEndpoint;
    /**
     * Returns true if the given object is an instance of VpcEndpoint.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is VpcEndpoint;
    /**
     * The Amazon Resource Name (ARN) of the VPC endpoint.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Accept the VPC endpoint (the VPC endpoint and service need to be in the same AWS account).
     */
    readonly autoAccept: pulumi.Output<boolean | undefined>;
    /**
     * The list of CIDR blocks for the exposed AWS service. Applicable for endpoints of type `Gateway`.
     */
    readonly cidrBlocks: pulumi.Output<string[]>;
    /**
     * The DNS entries for the VPC Endpoint. Applicable for endpoints of type `Interface`. DNS blocks are documented below.
     */
    readonly dnsEntries: pulumi.Output<outputs.ec2.VpcEndpointDnsEntry[]>;
    /**
     * The DNS options for the endpoint. See dnsOptions below.
     */
    readonly dnsOptions: pulumi.Output<outputs.ec2.VpcEndpointDnsOptions>;
    /**
     * The IP address type for the endpoint. Valid values are `ipv4`, `dualstack`, and `ipv6`.
     */
    readonly ipAddressType: pulumi.Output<string>;
    /**
     * One or more network interfaces for the VPC Endpoint. Applicable for endpoints of type `Interface`.
     */
    readonly networkInterfaceIds: pulumi.Output<string[]>;
    /**
     * The ID of the AWS account that owns the VPC endpoint.
     */
    readonly ownerId: pulumi.Output<string>;
    /**
     * A policy to attach to the endpoint that controls access to the service. This is a JSON formatted string. Defaults to full access. All `Gateway` and some `Interface` endpoints support policies - see the [relevant AWS documentation](https://docs.aws.amazon.com/vpc/latest/userguide/vpc-endpoints-access.html) for more details.
     */
    readonly policy: pulumi.Output<string>;
    /**
     * The prefix list ID of the exposed AWS service. Applicable for endpoints of type `Gateway`.
     */
    readonly prefixListId: pulumi.Output<string>;
    /**
     * Whether or not to associate a private hosted zone with the specified VPC. Applicable for endpoints of type `Interface`. Most users will want this enabled to allow services within the VPC to automatically use the endpoint.
     * Defaults to `false`.
     */
    readonly privateDnsEnabled: pulumi.Output<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Whether or not the VPC Endpoint is being managed by its service - `true` or `false`.
     */
    readonly requesterManaged: pulumi.Output<boolean>;
    /**
     * The ARN of a Resource Configuration to connect this VPC Endpoint to. Exactly one of `resourceConfigurationArn`, `serviceName` or `serviceNetworkArn` is required.
     */
    readonly resourceConfigurationArn: pulumi.Output<string | undefined>;
    /**
     * One or more route table IDs. Applicable for endpoints of type `Gateway`.
     */
    readonly routeTableIds: pulumi.Output<string[]>;
    /**
     * The ID of one or more security groups to associate with the network interface. Applicable for endpoints of type `Interface`.
     * If no security groups are specified, the VPC's [default security group](https://docs.aws.amazon.com/vpc/latest/userguide/VPC_SecurityGroups.html#DefaultSecurityGroup) is associated with the endpoint.
     */
    readonly securityGroupIds: pulumi.Output<string[]>;
    /**
     * The service name. For AWS services the service name is usually in the form `com.amazonaws.<region>.<service>` (the SageMaker AI Notebook service is an exception to this rule, the service name is in the form `aws.sagemaker.<region>.notebook`). Exactly one of `resourceConfigurationArn`, `serviceName` or `serviceNetworkArn` is required.
     */
    readonly serviceName: pulumi.Output<string | undefined>;
    /**
     * The ARN of a Service Network to connect this VPC Endpoint to. Exactly one of `resourceConfigurationArn`, `serviceName` or `serviceNetworkArn` is required.
     */
    readonly serviceNetworkArn: pulumi.Output<string | undefined>;
    /**
     * The AWS region of the VPC Endpoint Service. If specified, the VPC endpoint will connect to the service in the provided region. Applicable for endpoints of type `Interface`.
     */
    readonly serviceRegion: pulumi.Output<string>;
    /**
     * The state of the VPC endpoint.
     */
    readonly state: pulumi.Output<string>;
    /**
     * Subnet configuration for the endpoint, used to select specific IPv4 and/or IPv6 addresses to the endpoint. See subnetConfiguration below.
     */
    readonly subnetConfigurations: pulumi.Output<outputs.ec2.VpcEndpointSubnetConfiguration[]>;
    /**
     * The ID of one or more subnets in which to create a network interface for the endpoint. Applicable for endpoints of type `GatewayLoadBalancer` and `Interface`. Interface type endpoints cannot function without being assigned to a subnet.
     */
    readonly subnetIds: pulumi.Output<string[]>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * The VPC endpoint type, `Gateway`, `GatewayLoadBalancer`,`Interface`, `Resource` or `ServiceNetwork`. Defaults to `Gateway`.
     */
    readonly vpcEndpointType: pulumi.Output<string | undefined>;
    /**
     * The ID of the VPC in which the endpoint will be used.
     */
    readonly vpcId: pulumi.Output<string>;
    /**
     * Create a VpcEndpoint resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: VpcEndpointArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering VpcEndpoint resources.
 */
export interface VpcEndpointState {
    /**
     * The Amazon Resource Name (ARN) of the VPC endpoint.
     */
    arn?: pulumi.Input<string>;
    /**
     * Accept the VPC endpoint (the VPC endpoint and service need to be in the same AWS account).
     */
    autoAccept?: pulumi.Input<boolean>;
    /**
     * The list of CIDR blocks for the exposed AWS service. Applicable for endpoints of type `Gateway`.
     */
    cidrBlocks?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The DNS entries for the VPC Endpoint. Applicable for endpoints of type `Interface`. DNS blocks are documented below.
     */
    dnsEntries?: pulumi.Input<pulumi.Input<inputs.ec2.VpcEndpointDnsEntry>[]>;
    /**
     * The DNS options for the endpoint. See dnsOptions below.
     */
    dnsOptions?: pulumi.Input<inputs.ec2.VpcEndpointDnsOptions>;
    /**
     * The IP address type for the endpoint. Valid values are `ipv4`, `dualstack`, and `ipv6`.
     */
    ipAddressType?: pulumi.Input<string>;
    /**
     * One or more network interfaces for the VPC Endpoint. Applicable for endpoints of type `Interface`.
     */
    networkInterfaceIds?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The ID of the AWS account that owns the VPC endpoint.
     */
    ownerId?: pulumi.Input<string>;
    /**
     * A policy to attach to the endpoint that controls access to the service. This is a JSON formatted string. Defaults to full access. All `Gateway` and some `Interface` endpoints support policies - see the [relevant AWS documentation](https://docs.aws.amazon.com/vpc/latest/userguide/vpc-endpoints-access.html) for more details.
     */
    policy?: pulumi.Input<string>;
    /**
     * The prefix list ID of the exposed AWS service. Applicable for endpoints of type `Gateway`.
     */
    prefixListId?: pulumi.Input<string>;
    /**
     * Whether or not to associate a private hosted zone with the specified VPC. Applicable for endpoints of type `Interface`. Most users will want this enabled to allow services within the VPC to automatically use the endpoint.
     * Defaults to `false`.
     */
    privateDnsEnabled?: pulumi.Input<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Whether or not the VPC Endpoint is being managed by its service - `true` or `false`.
     */
    requesterManaged?: pulumi.Input<boolean>;
    /**
     * The ARN of a Resource Configuration to connect this VPC Endpoint to. Exactly one of `resourceConfigurationArn`, `serviceName` or `serviceNetworkArn` is required.
     */
    resourceConfigurationArn?: pulumi.Input<string>;
    /**
     * One or more route table IDs. Applicable for endpoints of type `Gateway`.
     */
    routeTableIds?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The ID of one or more security groups to associate with the network interface. Applicable for endpoints of type `Interface`.
     * If no security groups are specified, the VPC's [default security group](https://docs.aws.amazon.com/vpc/latest/userguide/VPC_SecurityGroups.html#DefaultSecurityGroup) is associated with the endpoint.
     */
    securityGroupIds?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The service name. For AWS services the service name is usually in the form `com.amazonaws.<region>.<service>` (the SageMaker AI Notebook service is an exception to this rule, the service name is in the form `aws.sagemaker.<region>.notebook`). Exactly one of `resourceConfigurationArn`, `serviceName` or `serviceNetworkArn` is required.
     */
    serviceName?: pulumi.Input<string>;
    /**
     * The ARN of a Service Network to connect this VPC Endpoint to. Exactly one of `resourceConfigurationArn`, `serviceName` or `serviceNetworkArn` is required.
     */
    serviceNetworkArn?: pulumi.Input<string>;
    /**
     * The AWS region of the VPC Endpoint Service. If specified, the VPC endpoint will connect to the service in the provided region. Applicable for endpoints of type `Interface`.
     */
    serviceRegion?: pulumi.Input<string>;
    /**
     * The state of the VPC endpoint.
     */
    state?: pulumi.Input<string>;
    /**
     * Subnet configuration for the endpoint, used to select specific IPv4 and/or IPv6 addresses to the endpoint. See subnetConfiguration below.
     */
    subnetConfigurations?: pulumi.Input<pulumi.Input<inputs.ec2.VpcEndpointSubnetConfiguration>[]>;
    /**
     * The ID of one or more subnets in which to create a network interface for the endpoint. Applicable for endpoints of type `GatewayLoadBalancer` and `Interface`. Interface type endpoints cannot function without being assigned to a subnet.
     */
    subnetIds?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * The VPC endpoint type, `Gateway`, `GatewayLoadBalancer`,`Interface`, `Resource` or `ServiceNetwork`. Defaults to `Gateway`.
     */
    vpcEndpointType?: pulumi.Input<string>;
    /**
     * The ID of the VPC in which the endpoint will be used.
     */
    vpcId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a VpcEndpoint resource.
 */
export interface VpcEndpointArgs {
    /**
     * Accept the VPC endpoint (the VPC endpoint and service need to be in the same AWS account).
     */
    autoAccept?: pulumi.Input<boolean>;
    /**
     * The DNS options for the endpoint. See dnsOptions below.
     */
    dnsOptions?: pulumi.Input<inputs.ec2.VpcEndpointDnsOptions>;
    /**
     * The IP address type for the endpoint. Valid values are `ipv4`, `dualstack`, and `ipv6`.
     */
    ipAddressType?: pulumi.Input<string>;
    /**
     * A policy to attach to the endpoint that controls access to the service. This is a JSON formatted string. Defaults to full access. All `Gateway` and some `Interface` endpoints support policies - see the [relevant AWS documentation](https://docs.aws.amazon.com/vpc/latest/userguide/vpc-endpoints-access.html) for more details.
     */
    policy?: pulumi.Input<string>;
    /**
     * Whether or not to associate a private hosted zone with the specified VPC. Applicable for endpoints of type `Interface`. Most users will want this enabled to allow services within the VPC to automatically use the endpoint.
     * Defaults to `false`.
     */
    privateDnsEnabled?: pulumi.Input<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The ARN of a Resource Configuration to connect this VPC Endpoint to. Exactly one of `resourceConfigurationArn`, `serviceName` or `serviceNetworkArn` is required.
     */
    resourceConfigurationArn?: pulumi.Input<string>;
    /**
     * One or more route table IDs. Applicable for endpoints of type `Gateway`.
     */
    routeTableIds?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The ID of one or more security groups to associate with the network interface. Applicable for endpoints of type `Interface`.
     * If no security groups are specified, the VPC's [default security group](https://docs.aws.amazon.com/vpc/latest/userguide/VPC_SecurityGroups.html#DefaultSecurityGroup) is associated with the endpoint.
     */
    securityGroupIds?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The service name. For AWS services the service name is usually in the form `com.amazonaws.<region>.<service>` (the SageMaker AI Notebook service is an exception to this rule, the service name is in the form `aws.sagemaker.<region>.notebook`). Exactly one of `resourceConfigurationArn`, `serviceName` or `serviceNetworkArn` is required.
     */
    serviceName?: pulumi.Input<string>;
    /**
     * The ARN of a Service Network to connect this VPC Endpoint to. Exactly one of `resourceConfigurationArn`, `serviceName` or `serviceNetworkArn` is required.
     */
    serviceNetworkArn?: pulumi.Input<string>;
    /**
     * The AWS region of the VPC Endpoint Service. If specified, the VPC endpoint will connect to the service in the provided region. Applicable for endpoints of type `Interface`.
     */
    serviceRegion?: pulumi.Input<string>;
    /**
     * Subnet configuration for the endpoint, used to select specific IPv4 and/or IPv6 addresses to the endpoint. See subnetConfiguration below.
     */
    subnetConfigurations?: pulumi.Input<pulumi.Input<inputs.ec2.VpcEndpointSubnetConfiguration>[]>;
    /**
     * The ID of one or more subnets in which to create a network interface for the endpoint. Applicable for endpoints of type `GatewayLoadBalancer` and `Interface`. Interface type endpoints cannot function without being assigned to a subnet.
     */
    subnetIds?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The VPC endpoint type, `Gateway`, `GatewayLoadBalancer`,`Interface`, `Resource` or `ServiceNetwork`. Defaults to `Gateway`.
     */
    vpcEndpointType?: pulumi.Input<string>;
    /**
     * The ID of the VPC in which the endpoint will be used.
     */
    vpcId: pulumi.Input<string>;
}
