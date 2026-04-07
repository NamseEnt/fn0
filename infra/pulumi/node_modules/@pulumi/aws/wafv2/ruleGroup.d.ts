import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Creates a WAFv2 Rule Group resource.
 *
 * ## Example Usage
 *
 * ### Simple
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.wafv2.RuleGroup("example", {
 *     name: "example-rule",
 *     scope: "REGIONAL",
 *     capacity: 2,
 *     rules: [{
 *         name: "rule-1",
 *         priority: 1,
 *         action: {
 *             allow: {},
 *         },
 *         statement: {
 *             geoMatchStatement: {
 *                 countryCodes: [
 *                     "US",
 *                     "NL",
 *                 ],
 *             },
 *         },
 *         visibilityConfig: {
 *             cloudwatchMetricsEnabled: false,
 *             metricName: "friendly-rule-metric-name",
 *             sampledRequestsEnabled: false,
 *         },
 *     }],
 *     visibilityConfig: {
 *         cloudwatchMetricsEnabled: false,
 *         metricName: "friendly-metric-name",
 *         sampledRequestsEnabled: false,
 *     },
 * });
 * ```
 *
 * ### Complex
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = new aws.wafv2.IpSet("test", {
 *     name: "test",
 *     scope: "REGIONAL",
 *     ipAddressVersion: "IPV4",
 *     addresses: [
 *         "1.1.1.1/32",
 *         "2.2.2.2/32",
 *     ],
 * });
 * const testRegexPatternSet = new aws.wafv2.RegexPatternSet("test", {
 *     name: "test",
 *     scope: "REGIONAL",
 *     regularExpressions: [{
 *         regexString: "one",
 *     }],
 * });
 * const example = new aws.wafv2.RuleGroup("example", {
 *     name: "complex-example",
 *     description: "An rule group containing all statements",
 *     scope: "REGIONAL",
 *     capacity: 500,
 *     rules: [
 *         {
 *             name: "rule-1",
 *             priority: 1,
 *             action: {
 *                 block: {},
 *             },
 *             statement: {
 *                 notStatement: {
 *                     statements: [{
 *                         andStatement: {
 *                             statements: [
 *                                 {
 *                                     geoMatchStatement: {
 *                                         countryCodes: ["US"],
 *                                     },
 *                                 },
 *                                 {
 *                                     byteMatchStatement: {
 *                                         positionalConstraint: "CONTAINS",
 *                                         searchString: "word",
 *                                         fieldToMatch: {
 *                                             allQueryArguments: {},
 *                                         },
 *                                         textTransformations: [
 *                                             {
 *                                                 priority: 5,
 *                                                 type: "CMD_LINE",
 *                                             },
 *                                             {
 *                                                 priority: 2,
 *                                                 type: "LOWERCASE",
 *                                             },
 *                                         ],
 *                                     },
 *                                 },
 *                             ],
 *                         },
 *                     }],
 *                 },
 *             },
 *             visibilityConfig: {
 *                 cloudwatchMetricsEnabled: false,
 *                 metricName: "rule-1",
 *                 sampledRequestsEnabled: false,
 *             },
 *         },
 *         {
 *             name: "rule-2",
 *             priority: 2,
 *             action: {
 *                 count: {},
 *             },
 *             statement: {
 *                 orStatement: {
 *                     statements: [
 *                         {
 *                             regexMatchStatement: {
 *                                 regexString: "a-z?",
 *                                 fieldToMatch: {
 *                                     singleHeader: {
 *                                         name: "user-agent",
 *                                     },
 *                                 },
 *                                 textTransformations: [{
 *                                     priority: 6,
 *                                     type: "NONE",
 *                                 }],
 *                             },
 *                         },
 *                         {
 *                             sqliMatchStatement: {
 *                                 fieldToMatch: {
 *                                     body: {},
 *                                 },
 *                                 textTransformations: [
 *                                     {
 *                                         priority: 5,
 *                                         type: "URL_DECODE",
 *                                     },
 *                                     {
 *                                         priority: 4,
 *                                         type: "HTML_ENTITY_DECODE",
 *                                     },
 *                                     {
 *                                         priority: 3,
 *                                         type: "COMPRESS_WHITE_SPACE",
 *                                     },
 *                                 ],
 *                             },
 *                         },
 *                         {
 *                             xssMatchStatement: {
 *                                 fieldToMatch: {
 *                                     method: {},
 *                                 },
 *                                 textTransformations: [{
 *                                     priority: 2,
 *                                     type: "NONE",
 *                                 }],
 *                             },
 *                         },
 *                     ],
 *                 },
 *             },
 *             visibilityConfig: {
 *                 cloudwatchMetricsEnabled: false,
 *                 metricName: "rule-2",
 *                 sampledRequestsEnabled: false,
 *             },
 *             captchaConfig: {
 *                 immunityTimeProperty: {
 *                     immunityTime: 240,
 *                 },
 *             },
 *         },
 *         {
 *             name: "rule-3",
 *             priority: 3,
 *             action: {
 *                 block: {},
 *             },
 *             statement: {
 *                 sizeConstraintStatement: {
 *                     comparisonOperator: "GT",
 *                     size: 100,
 *                     fieldToMatch: {
 *                         singleQueryArgument: {
 *                             name: "username",
 *                         },
 *                     },
 *                     textTransformations: [{
 *                         priority: 5,
 *                         type: "NONE",
 *                     }],
 *                 },
 *             },
 *             visibilityConfig: {
 *                 cloudwatchMetricsEnabled: false,
 *                 metricName: "rule-3",
 *                 sampledRequestsEnabled: false,
 *             },
 *         },
 *         {
 *             name: "rule-4",
 *             priority: 4,
 *             action: {
 *                 block: {},
 *             },
 *             statement: {
 *                 orStatement: {
 *                     statements: [
 *                         {
 *                             ipSetReferenceStatement: {
 *                                 arn: test.arn,
 *                             },
 *                         },
 *                         {
 *                             regexPatternSetReferenceStatement: {
 *                                 arn: testRegexPatternSet.arn,
 *                                 fieldToMatch: {
 *                                     singleHeader: {
 *                                         name: "referer",
 *                                     },
 *                                 },
 *                                 textTransformations: [{
 *                                     priority: 2,
 *                                     type: "NONE",
 *                                 }],
 *                             },
 *                         },
 *                     ],
 *                 },
 *             },
 *             visibilityConfig: {
 *                 cloudwatchMetricsEnabled: false,
 *                 metricName: "rule-4",
 *                 sampledRequestsEnabled: false,
 *             },
 *         },
 *     ],
 *     visibilityConfig: {
 *         cloudwatchMetricsEnabled: false,
 *         metricName: "friendly-metric-name",
 *         sampledRequestsEnabled: false,
 *     },
 *     captchaConfig: [{
 *         immunityTimeProperty: [{
 *             immunityTime: 120,
 *         }],
 *     }],
 *     tags: {
 *         Name: "example-and-statement",
 *         Code: "123456",
 *     },
 * });
 * ```
 *
 * ### Using rulesJson
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.wafv2.RuleGroup("example", {
 *     name: "example-rule-group",
 *     scope: "REGIONAL",
 *     capacity: 100,
 *     rulesJson: JSON.stringify([{
 *         Name: "rule-1",
 *         Priority: 1,
 *         Action: {
 *             Count: {},
 *         },
 *         Statement: {
 *             ByteMatchStatement: {
 *                 SearchString: "badbot",
 *                 FieldToMatch: {
 *                     UriPath: {},
 *                 },
 *                 TextTransformations: [{
 *                     Priority: 1,
 *                     Type: "NONE",
 *                 }],
 *                 PositionalConstraint: "CONTAINS",
 *             },
 *         },
 *         VisibilityConfig: {
 *             CloudwatchMetricsEnabled: false,
 *             MetricName: "friendly-rule-metric-name",
 *             SampledRequestsEnabled: false,
 *         },
 *     }]),
 *     visibilityConfig: {
 *         cloudwatchMetricsEnabled: false,
 *         metricName: "friendly-metric-name",
 *         sampledRequestsEnabled: false,
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import WAFv2 Rule Group using `ID/name/scope`. For example:
 *
 * ```sh
 * $ pulumi import aws:wafv2/ruleGroup:RuleGroup example a1b2c3d4-d5f6-7777-8888-9999aaaabbbbcccc/example/REGIONAL
 * ```
 */
export declare class RuleGroup extends pulumi.CustomResource {
    /**
     * Get an existing RuleGroup resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: RuleGroupState, opts?: pulumi.CustomResourceOptions): RuleGroup;
    /**
     * Returns true if the given object is an instance of RuleGroup.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is RuleGroup;
    /**
     * The ARN of the WAF rule group.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The web ACL capacity units (WCUs) required for this rule group. See [here](https://docs.aws.amazon.com/waf/latest/APIReference/API_CreateRuleGroup.html#API_CreateRuleGroup_RequestSyntax) for general information and [here](https://docs.aws.amazon.com/waf/latest/developerguide/waf-rule-statements-list.html) for capacity specific information.
     */
    readonly capacity: pulumi.Output<number>;
    /**
     * Defines custom response bodies that can be referenced by `customResponse` actions. See Custom Response Body below for details.
     */
    readonly customResponseBodies: pulumi.Output<outputs.wafv2.RuleGroupCustomResponseBody[] | undefined>;
    /**
     * A friendly description of the rule group.
     */
    readonly description: pulumi.Output<string | undefined>;
    readonly lockToken: pulumi.Output<string>;
    /**
     * A friendly name of the rule group.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Creates a unique name beginning with the specified prefix. Conflicts with `name`.
     */
    readonly namePrefix: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The rule blocks used to identify the web requests that you want to `allow`, `block`, or `count`. See Rules below for details.
     */
    readonly rules: pulumi.Output<outputs.wafv2.RuleGroupRule[] | undefined>;
    /**
     * Raw JSON string to allow more than three nested statements. Conflicts with `rule` attribute. This is for advanced use cases where more than 3 levels of nested statements are required. **There is no drift detection at this time**. If you use this attribute instead of `rule`, you will be foregoing drift detection. Additionally, importing an existing rule group into a configuration with `rulesJson` set will result in a one time in-place update as the remote rule configuration is initially written to the `rule` attribute. See the AWS [documentation](https://docs.aws.amazon.com/waf/latest/APIReference/API_CreateRuleGroup.html) for the JSON structure.
     */
    readonly rulesJson: pulumi.Output<string | undefined>;
    /**
     * Specifies whether this is for an AWS CloudFront distribution or for a regional application. Valid values are `CLOUDFRONT` or `REGIONAL`. To work with CloudFront, you must also specify the region `us-east-1` (N. Virginia) on the AWS provider.
     */
    readonly scope: pulumi.Output<string>;
    /**
     * An array of key:value pairs to associate with the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Defines and enables Amazon CloudWatch metrics and web request sample collection. See Visibility Configuration below for details.
     */
    readonly visibilityConfig: pulumi.Output<outputs.wafv2.RuleGroupVisibilityConfig>;
    /**
     * Create a RuleGroup resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: RuleGroupArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering RuleGroup resources.
 */
export interface RuleGroupState {
    /**
     * The ARN of the WAF rule group.
     */
    arn?: pulumi.Input<string>;
    /**
     * The web ACL capacity units (WCUs) required for this rule group. See [here](https://docs.aws.amazon.com/waf/latest/APIReference/API_CreateRuleGroup.html#API_CreateRuleGroup_RequestSyntax) for general information and [here](https://docs.aws.amazon.com/waf/latest/developerguide/waf-rule-statements-list.html) for capacity specific information.
     */
    capacity?: pulumi.Input<number>;
    /**
     * Defines custom response bodies that can be referenced by `customResponse` actions. See Custom Response Body below for details.
     */
    customResponseBodies?: pulumi.Input<pulumi.Input<inputs.wafv2.RuleGroupCustomResponseBody>[]>;
    /**
     * A friendly description of the rule group.
     */
    description?: pulumi.Input<string>;
    lockToken?: pulumi.Input<string>;
    /**
     * A friendly name of the rule group.
     */
    name?: pulumi.Input<string>;
    /**
     * Creates a unique name beginning with the specified prefix. Conflicts with `name`.
     */
    namePrefix?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The rule blocks used to identify the web requests that you want to `allow`, `block`, or `count`. See Rules below for details.
     */
    rules?: pulumi.Input<pulumi.Input<inputs.wafv2.RuleGroupRule>[]>;
    /**
     * Raw JSON string to allow more than three nested statements. Conflicts with `rule` attribute. This is for advanced use cases where more than 3 levels of nested statements are required. **There is no drift detection at this time**. If you use this attribute instead of `rule`, you will be foregoing drift detection. Additionally, importing an existing rule group into a configuration with `rulesJson` set will result in a one time in-place update as the remote rule configuration is initially written to the `rule` attribute. See the AWS [documentation](https://docs.aws.amazon.com/waf/latest/APIReference/API_CreateRuleGroup.html) for the JSON structure.
     */
    rulesJson?: pulumi.Input<string>;
    /**
     * Specifies whether this is for an AWS CloudFront distribution or for a regional application. Valid values are `CLOUDFRONT` or `REGIONAL`. To work with CloudFront, you must also specify the region `us-east-1` (N. Virginia) on the AWS provider.
     */
    scope?: pulumi.Input<string>;
    /**
     * An array of key:value pairs to associate with the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Defines and enables Amazon CloudWatch metrics and web request sample collection. See Visibility Configuration below for details.
     */
    visibilityConfig?: pulumi.Input<inputs.wafv2.RuleGroupVisibilityConfig>;
}
/**
 * The set of arguments for constructing a RuleGroup resource.
 */
export interface RuleGroupArgs {
    /**
     * The web ACL capacity units (WCUs) required for this rule group. See [here](https://docs.aws.amazon.com/waf/latest/APIReference/API_CreateRuleGroup.html#API_CreateRuleGroup_RequestSyntax) for general information and [here](https://docs.aws.amazon.com/waf/latest/developerguide/waf-rule-statements-list.html) for capacity specific information.
     */
    capacity: pulumi.Input<number>;
    /**
     * Defines custom response bodies that can be referenced by `customResponse` actions. See Custom Response Body below for details.
     */
    customResponseBodies?: pulumi.Input<pulumi.Input<inputs.wafv2.RuleGroupCustomResponseBody>[]>;
    /**
     * A friendly description of the rule group.
     */
    description?: pulumi.Input<string>;
    /**
     * A friendly name of the rule group.
     */
    name?: pulumi.Input<string>;
    /**
     * Creates a unique name beginning with the specified prefix. Conflicts with `name`.
     */
    namePrefix?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The rule blocks used to identify the web requests that you want to `allow`, `block`, or `count`. See Rules below for details.
     */
    rules?: pulumi.Input<pulumi.Input<inputs.wafv2.RuleGroupRule>[]>;
    /**
     * Raw JSON string to allow more than three nested statements. Conflicts with `rule` attribute. This is for advanced use cases where more than 3 levels of nested statements are required. **There is no drift detection at this time**. If you use this attribute instead of `rule`, you will be foregoing drift detection. Additionally, importing an existing rule group into a configuration with `rulesJson` set will result in a one time in-place update as the remote rule configuration is initially written to the `rule` attribute. See the AWS [documentation](https://docs.aws.amazon.com/waf/latest/APIReference/API_CreateRuleGroup.html) for the JSON structure.
     */
    rulesJson?: pulumi.Input<string>;
    /**
     * Specifies whether this is for an AWS CloudFront distribution or for a regional application. Valid values are `CLOUDFRONT` or `REGIONAL`. To work with CloudFront, you must also specify the region `us-east-1` (N. Virginia) on the AWS provider.
     */
    scope: pulumi.Input<string>;
    /**
     * An array of key:value pairs to associate with the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Defines and enables Amazon CloudWatch metrics and web request sample collection. See Visibility Configuration below for details.
     */
    visibilityConfig: pulumi.Input<inputs.wafv2.RuleGroupVisibilityConfig>;
}
