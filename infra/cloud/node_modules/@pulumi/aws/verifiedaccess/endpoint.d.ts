import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS EC2 (Elastic Compute Cloud) Verified Access Endpoint.
 *
 * ## Example Usage
 *
 * ### ALB Example
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.verifiedaccess.Endpoint("example", {
 *     applicationDomain: "example.com",
 *     attachmentType: "vpc",
 *     description: "example",
 *     domainCertificateArn: exampleAwsAcmCertificate.arn,
 *     endpointDomainPrefix: "example",
 *     endpointType: "load-balancer",
 *     loadBalancerOptions: {
 *         loadBalancerArn: exampleAwsLb.arn,
 *         port: 443,
 *         protocol: "https",
 *         subnetIds: .map(subnet => (subnet.id)),
 *     },
 *     securityGroupIds: [exampleAwsSecurityGroup.id],
 *     verifiedAccessGroupId: exampleAwsVerifiedaccessGroup.id,
 * });
 * ```
 *
 * ### Network Interface Example
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.verifiedaccess.Endpoint("example", {
 *     applicationDomain: "example.com",
 *     attachmentType: "vpc",
 *     description: "example",
 *     domainCertificateArn: exampleAwsAcmCertificate.arn,
 *     endpointDomainPrefix: "example",
 *     endpointType: "network-interface",
 *     networkInterfaceOptions: {
 *         networkInterfaceId: exampleAwsNetworkInterface.id,
 *         port: 443,
 *         protocol: "https",
 *     },
 *     securityGroupIds: [exampleAwsSecurityGroup.id],
 *     verifiedAccessGroupId: exampleAwsVerifiedaccessGroup.id,
 * });
 * ```
 *
 * ### Cidr Example
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.verifiedaccess.Endpoint("example", {
 *     attachmentType: "vpc",
 *     description: "example",
 *     endpointType: "cidr",
 *     cidrOptions: {
 *         cidr: test[0].cidrBlock,
 *         portRanges: [{
 *             fromPort: 443,
 *             toPort: 443,
 *         }],
 *         protocol: "tcp",
 *         subnetIds: .map(subnet => (subnet.id)),
 *     },
 *     securityGroupIds: [testAwsSecurityGroup.id],
 *     verifiedAccessGroupId: testAwsVerifiedaccessGroup.id,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Verified Access Instances using the  `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:verifiedaccess/endpoint:Endpoint example vae-8012925589
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
     * The DNS name for users to reach your application. This parameter is required if the endpoint type is `load-balancer` or `network-interface`.
     */
    readonly applicationDomain: pulumi.Output<string | undefined>;
    /**
     * The type of attachment. Currently, only `vpc` is supported.
     */
    readonly attachmentType: pulumi.Output<string>;
    /**
     * The CIDR block details. This parameter is required if the endpoint type is `cidr`.
     */
    readonly cidrOptions: pulumi.Output<outputs.verifiedaccess.EndpointCidrOptions | undefined>;
    /**
     * A description for the Verified Access endpoint.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * Returned if endpoint has a device trust provider attached.
     */
    readonly deviceValidationDomain: pulumi.Output<string>;
    /**
     * The ARN of the public TLS/SSL certificate in AWS Certificate Manager to associate with the endpoint. The CN in the certificate must match the DNS name your end users will use to reach your application. This parameter is required if the endpoint type is `load-balancer` or `network-interface`.
     */
    readonly domainCertificateArn: pulumi.Output<string | undefined>;
    /**
     * A DNS name that is generated for the endpoint.
     */
    readonly endpointDomain: pulumi.Output<string>;
    /**
     * A custom identifier that is prepended to the DNS name that is generated for the endpoint.
     */
    readonly endpointDomainPrefix: pulumi.Output<string | undefined>;
    /**
     * The type of Verified Access endpoint to create. Currently `load-balancer` or `network-interface` are supported.
     */
    readonly endpointType: pulumi.Output<string>;
    /**
     * The load balancer details. This parameter is required if the endpoint type is `load-balancer`.
     */
    readonly loadBalancerOptions: pulumi.Output<outputs.verifiedaccess.EndpointLoadBalancerOptions | undefined>;
    /**
     * The network interface details. This parameter is required if the endpoint type is `network-interface`.
     */
    readonly networkInterfaceOptions: pulumi.Output<outputs.verifiedaccess.EndpointNetworkInterfaceOptions | undefined>;
    /**
     * The policy document that is associated with this resource.
     */
    readonly policyDocument: pulumi.Output<string | undefined>;
    readonly rdsOptions: pulumi.Output<outputs.verifiedaccess.EndpointRdsOptions | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * List of the the security groups IDs to associate with the Verified Access endpoint.
     */
    readonly securityGroupIds: pulumi.Output<string[] | undefined>;
    /**
     * The options in use for server side encryption.
     */
    readonly sseSpecification: pulumi.Output<outputs.verifiedaccess.EndpointSseSpecification>;
    /**
     * Key-value tags for the Verified Access Endpoint. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * The ID of the Verified Access group to associate the endpoint with.
     *
     * The following arguments are optional:
     */
    readonly verifiedAccessGroupId: pulumi.Output<string>;
    readonly verifiedAccessInstanceId: pulumi.Output<string>;
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
     * The DNS name for users to reach your application. This parameter is required if the endpoint type is `load-balancer` or `network-interface`.
     */
    applicationDomain?: pulumi.Input<string>;
    /**
     * The type of attachment. Currently, only `vpc` is supported.
     */
    attachmentType?: pulumi.Input<string>;
    /**
     * The CIDR block details. This parameter is required if the endpoint type is `cidr`.
     */
    cidrOptions?: pulumi.Input<inputs.verifiedaccess.EndpointCidrOptions>;
    /**
     * A description for the Verified Access endpoint.
     */
    description?: pulumi.Input<string>;
    /**
     * Returned if endpoint has a device trust provider attached.
     */
    deviceValidationDomain?: pulumi.Input<string>;
    /**
     * The ARN of the public TLS/SSL certificate in AWS Certificate Manager to associate with the endpoint. The CN in the certificate must match the DNS name your end users will use to reach your application. This parameter is required if the endpoint type is `load-balancer` or `network-interface`.
     */
    domainCertificateArn?: pulumi.Input<string>;
    /**
     * A DNS name that is generated for the endpoint.
     */
    endpointDomain?: pulumi.Input<string>;
    /**
     * A custom identifier that is prepended to the DNS name that is generated for the endpoint.
     */
    endpointDomainPrefix?: pulumi.Input<string>;
    /**
     * The type of Verified Access endpoint to create. Currently `load-balancer` or `network-interface` are supported.
     */
    endpointType?: pulumi.Input<string>;
    /**
     * The load balancer details. This parameter is required if the endpoint type is `load-balancer`.
     */
    loadBalancerOptions?: pulumi.Input<inputs.verifiedaccess.EndpointLoadBalancerOptions>;
    /**
     * The network interface details. This parameter is required if the endpoint type is `network-interface`.
     */
    networkInterfaceOptions?: pulumi.Input<inputs.verifiedaccess.EndpointNetworkInterfaceOptions>;
    /**
     * The policy document that is associated with this resource.
     */
    policyDocument?: pulumi.Input<string>;
    rdsOptions?: pulumi.Input<inputs.verifiedaccess.EndpointRdsOptions>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * List of the the security groups IDs to associate with the Verified Access endpoint.
     */
    securityGroupIds?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The options in use for server side encryption.
     */
    sseSpecification?: pulumi.Input<inputs.verifiedaccess.EndpointSseSpecification>;
    /**
     * Key-value tags for the Verified Access Endpoint. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The ID of the Verified Access group to associate the endpoint with.
     *
     * The following arguments are optional:
     */
    verifiedAccessGroupId?: pulumi.Input<string>;
    verifiedAccessInstanceId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a Endpoint resource.
 */
export interface EndpointArgs {
    /**
     * The DNS name for users to reach your application. This parameter is required if the endpoint type is `load-balancer` or `network-interface`.
     */
    applicationDomain?: pulumi.Input<string>;
    /**
     * The type of attachment. Currently, only `vpc` is supported.
     */
    attachmentType: pulumi.Input<string>;
    /**
     * The CIDR block details. This parameter is required if the endpoint type is `cidr`.
     */
    cidrOptions?: pulumi.Input<inputs.verifiedaccess.EndpointCidrOptions>;
    /**
     * A description for the Verified Access endpoint.
     */
    description?: pulumi.Input<string>;
    /**
     * The ARN of the public TLS/SSL certificate in AWS Certificate Manager to associate with the endpoint. The CN in the certificate must match the DNS name your end users will use to reach your application. This parameter is required if the endpoint type is `load-balancer` or `network-interface`.
     */
    domainCertificateArn?: pulumi.Input<string>;
    /**
     * A custom identifier that is prepended to the DNS name that is generated for the endpoint.
     */
    endpointDomainPrefix?: pulumi.Input<string>;
    /**
     * The type of Verified Access endpoint to create. Currently `load-balancer` or `network-interface` are supported.
     */
    endpointType: pulumi.Input<string>;
    /**
     * The load balancer details. This parameter is required if the endpoint type is `load-balancer`.
     */
    loadBalancerOptions?: pulumi.Input<inputs.verifiedaccess.EndpointLoadBalancerOptions>;
    /**
     * The network interface details. This parameter is required if the endpoint type is `network-interface`.
     */
    networkInterfaceOptions?: pulumi.Input<inputs.verifiedaccess.EndpointNetworkInterfaceOptions>;
    /**
     * The policy document that is associated with this resource.
     */
    policyDocument?: pulumi.Input<string>;
    rdsOptions?: pulumi.Input<inputs.verifiedaccess.EndpointRdsOptions>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * List of the the security groups IDs to associate with the Verified Access endpoint.
     */
    securityGroupIds?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The options in use for server side encryption.
     */
    sseSpecification?: pulumi.Input<inputs.verifiedaccess.EndpointSseSpecification>;
    /**
     * Key-value tags for the Verified Access Endpoint. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The ID of the Verified Access group to associate the endpoint with.
     *
     * The following arguments are optional:
     */
    verifiedAccessGroupId: pulumi.Input<string>;
}
