import * as pulumi from "@pulumi/pulumi";
/**
 * Creates an entry (a rule) in a network ACL with the specified rule number.
 *
 * > **NOTE on Network ACLs and Network ACL Rules:** This provider currently
 * provides both a standalone Network ACL Rule resource and a Network ACL resource with rules
 * defined in-line. At this time you cannot use a Network ACL with in-line rules
 * in conjunction with any Network ACL Rule resources. Doing so will cause
 * a conflict of rule settings and will overwrite rules.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const bar = new aws.ec2.NetworkAcl("bar", {vpcId: foo.id});
 * const barNetworkAclRule = new aws.ec2.NetworkAclRule("bar", {
 *     networkAclId: bar.id,
 *     ruleNumber: 200,
 *     egress: false,
 *     protocol: "tcp",
 *     ruleAction: "allow",
 *     cidrBlock: foo.cidrBlock,
 *     fromPort: 22,
 *     toPort: 22,
 * });
 * ```
 *
 * > **Note:** One of either `cidrBlock` or `ipv6CidrBlock` is required.
 *
 * ## Import
 *
 * Using the procotol's decimal value:
 *
 * __Using `pulumi import` to import__ individual rules using `NETWORK_ACL_ID:RULE_NUMBER:PROTOCOL:EGRESS`, where `PROTOCOL` can be a decimal (such as "6") or string (such as "tcp") value. For example:
 *
 * Using the procotol's string value:
 *
 * ```sh
 * $ pulumi import aws:ec2/networkAclRule:NetworkAclRule my_rule acl-7aaabd18:100:tcp:false
 * ```
 * Using the procotol's decimal value:
 *
 * ```sh
 * $ pulumi import aws:ec2/networkAclRule:NetworkAclRule my_rule acl-7aaabd18:100:6:false
 * ```
 */
export declare class NetworkAclRule extends pulumi.CustomResource {
    /**
     * Get an existing NetworkAclRule resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: NetworkAclRuleState, opts?: pulumi.CustomResourceOptions): NetworkAclRule;
    /**
     * Returns true if the given object is an instance of NetworkAclRule.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is NetworkAclRule;
    /**
     * The network range to allow or deny, in CIDR notation (for example 172.16.0.0/24 ).
     */
    readonly cidrBlock: pulumi.Output<string | undefined>;
    /**
     * Indicates whether this is an egress rule (rule is applied to traffic leaving the subnet). Default `false`.
     */
    readonly egress: pulumi.Output<boolean | undefined>;
    /**
     * The from port to match.
     */
    readonly fromPort: pulumi.Output<number | undefined>;
    /**
     * ICMP protocol: The ICMP code. Required if specifying ICMP for the protocolE.g., -1
     *
     * > **NOTE:** If the value of `protocol` is `-1` or `all`, the `fromPort` and `toPort` values will be ignored and the rule will apply to all ports.
     *
     * > **NOTE:** If the value of `icmpType` is `-1` (which results in a wildcard ICMP type), the `icmpCode` must also be set to `-1` (wildcard ICMP code).
     *
     * > Note: For more information on ICMP types and codes, see here: https://www.iana.org/assignments/icmp-parameters/icmp-parameters.xhtml
     */
    readonly icmpCode: pulumi.Output<number | undefined>;
    /**
     * ICMP protocol: The ICMP type. Required if specifying ICMP for the protocolE.g., -1
     */
    readonly icmpType: pulumi.Output<number | undefined>;
    /**
     * The IPv6 CIDR block to allow or deny.
     */
    readonly ipv6CidrBlock: pulumi.Output<string | undefined>;
    /**
     * The ID of the network ACL.
     */
    readonly networkAclId: pulumi.Output<string>;
    /**
     * The protocol. A value of -1 means all protocols.
     */
    readonly protocol: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Indicates whether to allow or deny the traffic that matches the rule. Accepted values: `allow` | `deny`
     */
    readonly ruleAction: pulumi.Output<string>;
    /**
     * The rule number for the entry (for example, 100). ACL entries are processed in ascending order by rule number.
     */
    readonly ruleNumber: pulumi.Output<number>;
    /**
     * The to port to match.
     */
    readonly toPort: pulumi.Output<number | undefined>;
    /**
     * Create a NetworkAclRule resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: NetworkAclRuleArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering NetworkAclRule resources.
 */
export interface NetworkAclRuleState {
    /**
     * The network range to allow or deny, in CIDR notation (for example 172.16.0.0/24 ).
     */
    cidrBlock?: pulumi.Input<string>;
    /**
     * Indicates whether this is an egress rule (rule is applied to traffic leaving the subnet). Default `false`.
     */
    egress?: pulumi.Input<boolean>;
    /**
     * The from port to match.
     */
    fromPort?: pulumi.Input<number>;
    /**
     * ICMP protocol: The ICMP code. Required if specifying ICMP for the protocolE.g., -1
     *
     * > **NOTE:** If the value of `protocol` is `-1` or `all`, the `fromPort` and `toPort` values will be ignored and the rule will apply to all ports.
     *
     * > **NOTE:** If the value of `icmpType` is `-1` (which results in a wildcard ICMP type), the `icmpCode` must also be set to `-1` (wildcard ICMP code).
     *
     * > Note: For more information on ICMP types and codes, see here: https://www.iana.org/assignments/icmp-parameters/icmp-parameters.xhtml
     */
    icmpCode?: pulumi.Input<number>;
    /**
     * ICMP protocol: The ICMP type. Required if specifying ICMP for the protocolE.g., -1
     */
    icmpType?: pulumi.Input<number>;
    /**
     * The IPv6 CIDR block to allow or deny.
     */
    ipv6CidrBlock?: pulumi.Input<string>;
    /**
     * The ID of the network ACL.
     */
    networkAclId?: pulumi.Input<string>;
    /**
     * The protocol. A value of -1 means all protocols.
     */
    protocol?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Indicates whether to allow or deny the traffic that matches the rule. Accepted values: `allow` | `deny`
     */
    ruleAction?: pulumi.Input<string>;
    /**
     * The rule number for the entry (for example, 100). ACL entries are processed in ascending order by rule number.
     */
    ruleNumber?: pulumi.Input<number>;
    /**
     * The to port to match.
     */
    toPort?: pulumi.Input<number>;
}
/**
 * The set of arguments for constructing a NetworkAclRule resource.
 */
export interface NetworkAclRuleArgs {
    /**
     * The network range to allow or deny, in CIDR notation (for example 172.16.0.0/24 ).
     */
    cidrBlock?: pulumi.Input<string>;
    /**
     * Indicates whether this is an egress rule (rule is applied to traffic leaving the subnet). Default `false`.
     */
    egress?: pulumi.Input<boolean>;
    /**
     * The from port to match.
     */
    fromPort?: pulumi.Input<number>;
    /**
     * ICMP protocol: The ICMP code. Required if specifying ICMP for the protocolE.g., -1
     *
     * > **NOTE:** If the value of `protocol` is `-1` or `all`, the `fromPort` and `toPort` values will be ignored and the rule will apply to all ports.
     *
     * > **NOTE:** If the value of `icmpType` is `-1` (which results in a wildcard ICMP type), the `icmpCode` must also be set to `-1` (wildcard ICMP code).
     *
     * > Note: For more information on ICMP types and codes, see here: https://www.iana.org/assignments/icmp-parameters/icmp-parameters.xhtml
     */
    icmpCode?: pulumi.Input<number>;
    /**
     * ICMP protocol: The ICMP type. Required if specifying ICMP for the protocolE.g., -1
     */
    icmpType?: pulumi.Input<number>;
    /**
     * The IPv6 CIDR block to allow or deny.
     */
    ipv6CidrBlock?: pulumi.Input<string>;
    /**
     * The ID of the network ACL.
     */
    networkAclId: pulumi.Input<string>;
    /**
     * The protocol. A value of -1 means all protocols.
     */
    protocol: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Indicates whether to allow or deny the traffic that matches the rule. Accepted values: `allow` | `deny`
     */
    ruleAction: pulumi.Input<string>;
    /**
     * The rule number for the entry (for example, 100). ACL entries are processed in ascending order by rule number.
     */
    ruleNumber: pulumi.Input<number>;
    /**
     * The to port to match.
     */
    toPort?: pulumi.Input<number>;
}
