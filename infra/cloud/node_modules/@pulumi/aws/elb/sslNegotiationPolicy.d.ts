import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a load balancer SSL negotiation policy, which allows an ELB to control the ciphers and protocols that are supported during SSL negotiations between a client and a load balancer.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const lb = new aws.elb.LoadBalancer("lb", {
 *     name: "test-lb",
 *     availabilityZones: ["us-east-1a"],
 *     listeners: [{
 *         instancePort: 8000,
 *         instanceProtocol: "https",
 *         lbPort: 443,
 *         lbProtocol: "https",
 *         sslCertificateId: "arn:aws:iam::123456789012:server-certificate/certName",
 *     }],
 * });
 * const foo = new aws.elb.SslNegotiationPolicy("foo", {
 *     name: "foo-policy",
 *     loadBalancer: lb.id,
 *     lbPort: 443,
 *     attributes: [
 *         {
 *             name: "Protocol-TLSv1",
 *             value: "false",
 *         },
 *         {
 *             name: "Protocol-TLSv1.1",
 *             value: "false",
 *         },
 *         {
 *             name: "Protocol-TLSv1.2",
 *             value: "true",
 *         },
 *         {
 *             name: "Server-Defined-Cipher-Order",
 *             value: "true",
 *         },
 *         {
 *             name: "ECDHE-RSA-AES128-GCM-SHA256",
 *             value: "true",
 *         },
 *         {
 *             name: "AES128-GCM-SHA256",
 *             value: "true",
 *         },
 *         {
 *             name: "EDH-RSA-DES-CBC3-SHA",
 *             value: "false",
 *         },
 *     ],
 * });
 * ```
 */
export declare class SslNegotiationPolicy extends pulumi.CustomResource {
    /**
     * Get an existing SslNegotiationPolicy resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: SslNegotiationPolicyState, opts?: pulumi.CustomResourceOptions): SslNegotiationPolicy;
    /**
     * Returns true if the given object is an instance of SslNegotiationPolicy.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is SslNegotiationPolicy;
    /**
     * An SSL Negotiation policy attribute. Each has two properties:
     */
    readonly attributes: pulumi.Output<outputs.elb.SslNegotiationPolicyAttribute[] | undefined>;
    /**
     * The load balancer port to which the policy
     * should be applied. This must be an active listener on the load
     * balancer.
     */
    readonly lbPort: pulumi.Output<number>;
    /**
     * The load balancer to which the policy
     * should be attached.
     */
    readonly loadBalancer: pulumi.Output<string>;
    /**
     * The name of the SSL negotiation policy.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Map of arbitrary keys and values that, when changed, will trigger a redeployment.
     *
     * To set your attributes, please see the [AWS Elastic Load Balancing Developer Guide](http://docs.aws.amazon.com/ElasticLoadBalancing/latest/DeveloperGuide/elb-security-policy-table.html) for a listing of the supported SSL protocols, SSL options, and SSL ciphers.
     *
     * > **NOTE:** The AWS documentation references Server Order Preference, which the AWS Elastic Load Balancing API refers to as `Server-Defined-Cipher-Order`. If you wish to set Server Order Preference, use this value instead.
     */
    readonly triggers: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * Create a SslNegotiationPolicy resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: SslNegotiationPolicyArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering SslNegotiationPolicy resources.
 */
export interface SslNegotiationPolicyState {
    /**
     * An SSL Negotiation policy attribute. Each has two properties:
     */
    attributes?: pulumi.Input<pulumi.Input<inputs.elb.SslNegotiationPolicyAttribute>[]>;
    /**
     * The load balancer port to which the policy
     * should be applied. This must be an active listener on the load
     * balancer.
     */
    lbPort?: pulumi.Input<number>;
    /**
     * The load balancer to which the policy
     * should be attached.
     */
    loadBalancer?: pulumi.Input<string>;
    /**
     * The name of the SSL negotiation policy.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Map of arbitrary keys and values that, when changed, will trigger a redeployment.
     *
     * To set your attributes, please see the [AWS Elastic Load Balancing Developer Guide](http://docs.aws.amazon.com/ElasticLoadBalancing/latest/DeveloperGuide/elb-security-policy-table.html) for a listing of the supported SSL protocols, SSL options, and SSL ciphers.
     *
     * > **NOTE:** The AWS documentation references Server Order Preference, which the AWS Elastic Load Balancing API refers to as `Server-Defined-Cipher-Order`. If you wish to set Server Order Preference, use this value instead.
     */
    triggers?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
/**
 * The set of arguments for constructing a SslNegotiationPolicy resource.
 */
export interface SslNegotiationPolicyArgs {
    /**
     * An SSL Negotiation policy attribute. Each has two properties:
     */
    attributes?: pulumi.Input<pulumi.Input<inputs.elb.SslNegotiationPolicyAttribute>[]>;
    /**
     * The load balancer port to which the policy
     * should be applied. This must be an active listener on the load
     * balancer.
     */
    lbPort: pulumi.Input<number>;
    /**
     * The load balancer to which the policy
     * should be attached.
     */
    loadBalancer: pulumi.Input<string>;
    /**
     * The name of the SSL negotiation policy.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Map of arbitrary keys and values that, when changed, will trigger a redeployment.
     *
     * To set your attributes, please see the [AWS Elastic Load Balancing Developer Guide](http://docs.aws.amazon.com/ElasticLoadBalancing/latest/DeveloperGuide/elb-security-policy-table.html) for a listing of the supported SSL protocols, SSL options, and SSL ciphers.
     *
     * > **NOTE:** The AWS documentation references Server Order Preference, which the AWS Elastic Load Balancing API refers to as `Server-Defined-Cipher-Order`. If you wish to set Server Order Preference, use this value instead.
     */
    triggers?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
