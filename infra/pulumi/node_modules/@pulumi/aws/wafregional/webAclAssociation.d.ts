import * as pulumi from "@pulumi/pulumi";
/**
 * Manages an association with WAF Regional Web ACL.
 *
 * > **Note:** An Application Load Balancer can only be associated with one WAF Regional WebACL.
 *
 * ## Example Usage
 *
 * ### Application Load Balancer Association
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const ipset = new aws.wafregional.IpSet("ipset", {
 *     name: "tfIPSet",
 *     ipSetDescriptors: [{
 *         type: "IPV4",
 *         value: "192.0.7.0/24",
 *     }],
 * });
 * const foo = new aws.wafregional.Rule("foo", {
 *     name: "tfWAFRule",
 *     metricName: "tfWAFRule",
 *     predicates: [{
 *         dataId: ipset.id,
 *         negated: false,
 *         type: "IPMatch",
 *     }],
 * });
 * const fooWebAcl = new aws.wafregional.WebAcl("foo", {
 *     name: "foo",
 *     metricName: "foo",
 *     defaultAction: {
 *         type: "ALLOW",
 *     },
 *     rules: [{
 *         action: {
 *             type: "BLOCK",
 *         },
 *         priority: 1,
 *         ruleId: foo.id,
 *     }],
 * });
 * const fooVpc = new aws.ec2.Vpc("foo", {cidrBlock: "10.1.0.0/16"});
 * const available = aws.getAvailabilityZones({});
 * const fooSubnet = new aws.ec2.Subnet("foo", {
 *     vpcId: fooVpc.id,
 *     cidrBlock: "10.1.1.0/24",
 *     availabilityZone: available.then(available => available.names?.[0]),
 * });
 * const bar = new aws.ec2.Subnet("bar", {
 *     vpcId: fooVpc.id,
 *     cidrBlock: "10.1.2.0/24",
 *     availabilityZone: available.then(available => available.names?.[1]),
 * });
 * const fooLoadBalancer = new aws.alb.LoadBalancer("foo", {
 *     internal: true,
 *     subnets: [
 *         fooSubnet.id,
 *         bar.id,
 *     ],
 * });
 * const fooWebAclAssociation = new aws.wafregional.WebAclAssociation("foo", {
 *     resourceArn: fooLoadBalancer.arn,
 *     webAclId: fooWebAcl.id,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import WAF Regional Web ACL Association using their `web_acl_id:resource_arn`. For example:
 *
 * ```sh
 * $ pulumi import aws:wafregional/webAclAssociation:WebAclAssociation foo web_acl_id:resource_arn
 * ```
 */
export declare class WebAclAssociation extends pulumi.CustomResource {
    /**
     * Get an existing WebAclAssociation resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: WebAclAssociationState, opts?: pulumi.CustomResourceOptions): WebAclAssociation;
    /**
     * Returns true if the given object is an instance of WebAclAssociation.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is WebAclAssociation;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * ARN of the resource to associate with. For example, an Application Load Balancer or API Gateway Stage.
     */
    readonly resourceArn: pulumi.Output<string>;
    /**
     * The ID of the WAF Regional WebACL to create an association.
     */
    readonly webAclId: pulumi.Output<string>;
    /**
     * Create a WebAclAssociation resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: WebAclAssociationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering WebAclAssociation resources.
 */
export interface WebAclAssociationState {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ARN of the resource to associate with. For example, an Application Load Balancer or API Gateway Stage.
     */
    resourceArn?: pulumi.Input<string>;
    /**
     * The ID of the WAF Regional WebACL to create an association.
     */
    webAclId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a WebAclAssociation resource.
 */
export interface WebAclAssociationArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ARN of the resource to associate with. For example, an Application Load Balancer or API Gateway Stage.
     */
    resourceArn: pulumi.Input<string>;
    /**
     * The ID of the WAF Regional WebACL to create an association.
     */
    webAclId: pulumi.Input<string>;
}
