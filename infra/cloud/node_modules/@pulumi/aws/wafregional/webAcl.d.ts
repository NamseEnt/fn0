import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a WAF Regional Web ACL Resource for use with Application Load Balancer.
 *
 * ## Example Usage
 *
 * ### Regular Rule
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
 * const wafrule = new aws.wafregional.Rule("wafrule", {
 *     name: "tfWAFRule",
 *     metricName: "tfWAFRule",
 *     predicates: [{
 *         dataId: ipset.id,
 *         negated: false,
 *         type: "IPMatch",
 *     }],
 * });
 * const wafacl = new aws.wafregional.WebAcl("wafacl", {
 *     name: "tfWebACL",
 *     metricName: "tfWebACL",
 *     defaultAction: {
 *         type: "ALLOW",
 *     },
 *     rules: [{
 *         action: {
 *             type: "BLOCK",
 *         },
 *         priority: 1,
 *         ruleId: wafrule.id,
 *         type: "REGULAR",
 *     }],
 * });
 * ```
 *
 * ### Group Rule
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.wafregional.WebAcl("example", {
 *     name: "example",
 *     metricName: "example",
 *     defaultAction: {
 *         type: "ALLOW",
 *     },
 *     rules: [{
 *         priority: 1,
 *         ruleId: exampleAwsWafregionalRuleGroup.id,
 *         type: "GROUP",
 *         overrideAction: {
 *             type: "NONE",
 *         },
 *     }],
 * });
 * ```
 *
 * ### Logging
 *
 * > *NOTE:* The Kinesis Firehose Delivery Stream name must begin with `aws-waf-logs-`. See the [AWS WAF Developer Guide](https://docs.aws.amazon.com/waf/latest/developerguide/logging.html) for more information about enabling WAF logging.
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.wafregional.WebAcl("example", {loggingConfiguration: {
 *     logDestination: exampleAwsKinesisFirehoseDeliveryStream.arn,
 *     redactedFields: {
 *         fieldToMatches: [
 *             {
 *                 type: "URI",
 *             },
 *             {
 *                 data: "referer",
 *                 type: "HEADER",
 *             },
 *         ],
 *     },
 * }});
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import WAF Regional Web ACL using the id. For example:
 *
 * ```sh
 * $ pulumi import aws:wafregional/webAcl:WebAcl wafacl a1b2c3d4-d5f6-7777-8888-9999aaaabbbbcccc
 * ```
 */
export declare class WebAcl extends pulumi.CustomResource {
    /**
     * Get an existing WebAcl resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: WebAclState, opts?: pulumi.CustomResourceOptions): WebAcl;
    /**
     * Returns true if the given object is an instance of WebAcl.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is WebAcl;
    /**
     * Amazon Resource Name (ARN) of the WAF Regional WebACL.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The action that you want AWS WAF Regional to take when a request doesn't match the criteria in any of the rules that are associated with the web ACL.
     */
    readonly defaultAction: pulumi.Output<outputs.wafregional.WebAclDefaultAction>;
    /**
     * Configuration block to enable WAF logging. Detailed below.
     */
    readonly loggingConfiguration: pulumi.Output<outputs.wafregional.WebAclLoggingConfiguration | undefined>;
    /**
     * The name or description for the Amazon CloudWatch metric of this web ACL.
     */
    readonly metricName: pulumi.Output<string>;
    /**
     * The name or description of the web ACL.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Set of configuration blocks containing rules for the web ACL. Detailed below.
     */
    readonly rules: pulumi.Output<outputs.wafregional.WebAclRule[] | undefined>;
    /**
     * Key-value map of resource tags. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Create a WebAcl resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: WebAclArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering WebAcl resources.
 */
export interface WebAclState {
    /**
     * Amazon Resource Name (ARN) of the WAF Regional WebACL.
     */
    arn?: pulumi.Input<string>;
    /**
     * The action that you want AWS WAF Regional to take when a request doesn't match the criteria in any of the rules that are associated with the web ACL.
     */
    defaultAction?: pulumi.Input<inputs.wafregional.WebAclDefaultAction>;
    /**
     * Configuration block to enable WAF logging. Detailed below.
     */
    loggingConfiguration?: pulumi.Input<inputs.wafregional.WebAclLoggingConfiguration>;
    /**
     * The name or description for the Amazon CloudWatch metric of this web ACL.
     */
    metricName?: pulumi.Input<string>;
    /**
     * The name or description of the web ACL.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Set of configuration blocks containing rules for the web ACL. Detailed below.
     */
    rules?: pulumi.Input<pulumi.Input<inputs.wafregional.WebAclRule>[]>;
    /**
     * Key-value map of resource tags. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
 * The set of arguments for constructing a WebAcl resource.
 */
export interface WebAclArgs {
    /**
     * The action that you want AWS WAF Regional to take when a request doesn't match the criteria in any of the rules that are associated with the web ACL.
     */
    defaultAction: pulumi.Input<inputs.wafregional.WebAclDefaultAction>;
    /**
     * Configuration block to enable WAF logging. Detailed below.
     */
    loggingConfiguration?: pulumi.Input<inputs.wafregional.WebAclLoggingConfiguration>;
    /**
     * The name or description for the Amazon CloudWatch metric of this web ACL.
     */
    metricName: pulumi.Input<string>;
    /**
     * The name or description of the web ACL.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Set of configuration blocks containing rules for the web ACL. Detailed below.
     */
    rules?: pulumi.Input<pulumi.Input<inputs.wafregional.WebAclRule>[]>;
    /**
     * Key-value map of resource tags. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
