import * as pulumi from "@pulumi/pulumi";
/**
 * Manages an inbound (ingress) rule for a security group.
 *
 * When specifying an inbound rule for your security group in a VPC, the configuration must include a source for the traffic.
 *
 * > **NOTE:** Using `aws.vpc.SecurityGroupEgressRule` and `aws.vpc.SecurityGroupIngressRule` resources is the current best practice. Avoid using the `aws.ec2.SecurityGroupRule` resource and the `ingress` and `egress` arguments of the `aws.ec2.SecurityGroup` resource for configuring in-line rules, as they struggle with managing multiple CIDR blocks, and tags and descriptions due to the historical lack of unique IDs.
 *
 * !> **WARNING:** You should not use the `aws.vpc.SecurityGroupEgressRule` and `aws.vpc.SecurityGroupIngressRule` resources in conjunction with the `aws.ec2.SecurityGroup` resource with _in-line rules_ (using the `ingress` and `egress` arguments of `aws.ec2.SecurityGroup`) or the `aws.ec2.SecurityGroupRule` resource. Doing so may cause rule conflicts, perpetual differences, and result in rules being overwritten.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.ec2.SecurityGroup("example", {
 *     name: "example",
 *     description: "example",
 *     vpcId: main.id,
 *     tags: {
 *         Name: "example",
 *     },
 * });
 * const exampleSecurityGroupIngressRule = new aws.vpc.SecurityGroupIngressRule("example", {
 *     securityGroupId: example.id,
 *     cidrIpv4: "10.0.0.0/8",
 *     fromPort: 80,
 *     ipProtocol: "tcp",
 *     toPort: 80,
 * });
 * ```
 *
 * ## Import
 *
 * ### Identity Schema
 *
 * #### Required
 *
 * * `id` - (String) ID of the security group rule.
 *
 * #### Optional
 *
 * * `account_id` (String) AWS Account where this resource is managed.
 *
 * * `region` (String) Region where this resource is managed.
 *
 * Using `pulumi import`, import security group ingress rules using the `security_group_rule_id`. For example:
 *
 * console
 *
 * % pulumi import aws_vpc_security_group_ingress_rule.example sgr-02108b27edd666983
 */
export declare class SecurityGroupIngressRule extends pulumi.CustomResource {
    /**
     * Get an existing SecurityGroupIngressRule resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: SecurityGroupIngressRuleState, opts?: pulumi.CustomResourceOptions): SecurityGroupIngressRule;
    /**
     * Returns true if the given object is an instance of SecurityGroupIngressRule.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is SecurityGroupIngressRule;
    /**
     * The Amazon Resource Name (ARN) of the security group rule.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The source IPv4 CIDR range.
     */
    readonly cidrIpv4: pulumi.Output<string | undefined>;
    /**
     * The source IPv6 CIDR range.
     */
    readonly cidrIpv6: pulumi.Output<string | undefined>;
    /**
     * The security group rule description.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * The start of port range for the TCP and UDP protocols, or an ICMP/ICMPv6 type.
     */
    readonly fromPort: pulumi.Output<number | undefined>;
    /**
     * The IP protocol name or number. Use `-1` to specify all protocols. Note that if `ipProtocol` is set to `-1`, it translates to all protocols, all port ranges, and `fromPort` and `toPort` values should not be defined.
     */
    readonly ipProtocol: pulumi.Output<string>;
    /**
     * The ID of the source prefix list.
     */
    readonly prefixListId: pulumi.Output<string | undefined>;
    /**
     * The source security group that is referenced in the rule.
     */
    readonly referencedSecurityGroupId: pulumi.Output<string | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The ID of the security group.
     */
    readonly securityGroupId: pulumi.Output<string>;
    /**
     * The ID of the security group rule.
     */
    readonly securityGroupRuleId: pulumi.Output<string>;
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
     * The end of port range for the TCP and UDP protocols, or an ICMP/ICMPv6 code.
     *
     * > **Note** Although `cidrIpv4`, `cidrIpv6`, `prefixListId`, and `referencedSecurityGroupId` are all marked as optional, you *must* provide one of them in order to configure the destination of the traffic. The `fromPort` and `toPort` arguments are required unless `ipProtocol` is set to `-1` or `icmpv6`.
     */
    readonly toPort: pulumi.Output<number | undefined>;
    /**
     * Create a SecurityGroupIngressRule resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: SecurityGroupIngressRuleArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering SecurityGroupIngressRule resources.
 */
export interface SecurityGroupIngressRuleState {
    /**
     * The Amazon Resource Name (ARN) of the security group rule.
     */
    arn?: pulumi.Input<string>;
    /**
     * The source IPv4 CIDR range.
     */
    cidrIpv4?: pulumi.Input<string>;
    /**
     * The source IPv6 CIDR range.
     */
    cidrIpv6?: pulumi.Input<string>;
    /**
     * The security group rule description.
     */
    description?: pulumi.Input<string>;
    /**
     * The start of port range for the TCP and UDP protocols, or an ICMP/ICMPv6 type.
     */
    fromPort?: pulumi.Input<number>;
    /**
     * The IP protocol name or number. Use `-1` to specify all protocols. Note that if `ipProtocol` is set to `-1`, it translates to all protocols, all port ranges, and `fromPort` and `toPort` values should not be defined.
     */
    ipProtocol?: pulumi.Input<string>;
    /**
     * The ID of the source prefix list.
     */
    prefixListId?: pulumi.Input<string>;
    /**
     * The source security group that is referenced in the rule.
     */
    referencedSecurityGroupId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The ID of the security group.
     */
    securityGroupId?: pulumi.Input<string>;
    /**
     * The ID of the security group rule.
     */
    securityGroupRuleId?: pulumi.Input<string>;
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
     * The end of port range for the TCP and UDP protocols, or an ICMP/ICMPv6 code.
     *
     * > **Note** Although `cidrIpv4`, `cidrIpv6`, `prefixListId`, and `referencedSecurityGroupId` are all marked as optional, you *must* provide one of them in order to configure the destination of the traffic. The `fromPort` and `toPort` arguments are required unless `ipProtocol` is set to `-1` or `icmpv6`.
     */
    toPort?: pulumi.Input<number>;
}
/**
 * The set of arguments for constructing a SecurityGroupIngressRule resource.
 */
export interface SecurityGroupIngressRuleArgs {
    /**
     * The source IPv4 CIDR range.
     */
    cidrIpv4?: pulumi.Input<string>;
    /**
     * The source IPv6 CIDR range.
     */
    cidrIpv6?: pulumi.Input<string>;
    /**
     * The security group rule description.
     */
    description?: pulumi.Input<string>;
    /**
     * The start of port range for the TCP and UDP protocols, or an ICMP/ICMPv6 type.
     */
    fromPort?: pulumi.Input<number>;
    /**
     * The IP protocol name or number. Use `-1` to specify all protocols. Note that if `ipProtocol` is set to `-1`, it translates to all protocols, all port ranges, and `fromPort` and `toPort` values should not be defined.
     */
    ipProtocol: pulumi.Input<string>;
    /**
     * The ID of the source prefix list.
     */
    prefixListId?: pulumi.Input<string>;
    /**
     * The source security group that is referenced in the rule.
     */
    referencedSecurityGroupId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The ID of the security group.
     */
    securityGroupId: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The end of port range for the TCP and UDP protocols, or an ICMP/ICMPv6 code.
     *
     * > **Note** Although `cidrIpv4`, `cidrIpv6`, `prefixListId`, and `referencedSecurityGroupId` are all marked as optional, you *must* provide one of them in order to configure the destination of the traffic. The `fromPort` and `toPort` arguments are required unless `ipProtocol` is set to `-1` or `icmpv6`.
     */
    toPort?: pulumi.Input<number>;
}
