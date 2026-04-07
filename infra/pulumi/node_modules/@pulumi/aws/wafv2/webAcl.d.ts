import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Creates a WAFv2 Web ACL resource.
 *
 * > **Note** In `fieldToMatch` blocks, *e.g.*, in `byteMatchStatement`, the `body` block includes an optional argument `oversizeHandling`. AWS indicates this argument will be required starting February 2023. To avoid configurations breaking when that change happens, treat the `oversizeHandling` argument as **required** as soon as possible.
 *
 * !> **Warning:** If you use the `aws.wafv2.WebAclRuleGroupAssociation` resource to associate rule groups with this Web ACL, you must add `lifecycle { ignoreChanges = [rule] }` to this resource to prevent configuration drift. The association resource modifies the Web ACL's rules outside of this resource's direct management.
 *
 * ## Import
 *
 * Using `pulumi import`, import WAFv2 Web ACLs using `ID/Name/Scope`. For example:
 *
 * ```sh
 * $ pulumi import aws:wafv2/webAcl:WebAcl example a1b2c3d4-d5f6-7777-8888-9999aaaabbbbcccc/example/REGIONAL
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
     * The URL to use in SDK integrations with managed rule groups.
     */
    readonly applicationIntegrationUrl: pulumi.Output<string>;
    /**
     * The ARN of the WAF WebACL.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Specifies custom configurations for the associations between the web ACL and protected resources. See `associationConfig` below for details.
     */
    readonly associationConfig: pulumi.Output<outputs.wafv2.WebAclAssociationConfig | undefined>;
    /**
     * Web ACL capacity units (WCUs) currently being used by this web ACL.
     */
    readonly capacity: pulumi.Output<number>;
    /**
     * Specifies how AWS WAF should handle CAPTCHA evaluations on the ACL level (used by [AWS Bot Control](https://docs.aws.amazon.com/waf/latest/developerguide/aws-managed-rule-groups-bot.html)). See `captchaConfig` below for details.
     */
    readonly captchaConfig: pulumi.Output<outputs.wafv2.WebAclCaptchaConfig | undefined>;
    /**
     * Specifies how AWS WAF should handle Challenge evaluations on the ACL level (used by [AWS Bot Control](https://docs.aws.amazon.com/waf/latest/developerguide/aws-managed-rule-groups-bot.html)). See `challengeConfig` below for details.
     */
    readonly challengeConfig: pulumi.Output<outputs.wafv2.WebAclChallengeConfig | undefined>;
    /**
     * Defines custom response bodies that can be referenced by `customResponse` actions. See `customResponseBody` below for details.
     */
    readonly customResponseBodies: pulumi.Output<outputs.wafv2.WebAclCustomResponseBody[] | undefined>;
    /**
     * Specifies data protection to apply to the web request data for the web ACL. This is a web ACL level data protection option. See `dataProtectionConfig` below for details.
     */
    readonly dataProtectionConfig: pulumi.Output<outputs.wafv2.WebAclDataProtectionConfig | undefined>;
    /**
     * Action to perform if none of the `rules` contained in the WebACL match. See `defaultAction` below for details.
     */
    readonly defaultAction: pulumi.Output<outputs.wafv2.WebAclDefaultAction>;
    /**
     * Friendly description of the WebACL.
     */
    readonly description: pulumi.Output<string | undefined>;
    readonly lockToken: pulumi.Output<string>;
    /**
     * Friendly name of the WebACL. If omitted, the provider will assign a random, unique name. Conflicts with `namePrefix`.
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
     * Raw JSON string to allow more than three nested statements. Conflicts with `rule` attribute. This is for advanced use cases where more than 3 levels of nested statements are required. **There is no drift detection at this time**. If you use this attribute instead of `rule`, you will be foregoing drift detection. Additionally, importing an existing web ACL into a configuration with `ruleJson` set will result in a one time in-place update as the remote rule configuration is initially written to the `rule` attribute. See the AWS [documentation](https://docs.aws.amazon.com/waf/latest/APIReference/API_CreateWebACL.html) for the JSON structure.
     */
    readonly ruleJson: pulumi.Output<string | undefined>;
    /**
     * Rule blocks used to identify the web requests that you want to `allow`, `block`, or `count`. See `rule` below for details.
     */
    readonly rules: pulumi.Output<outputs.wafv2.WebAclRule[] | undefined>;
    /**
     * Specifies whether this is for an AWS CloudFront distribution or for a regional application. Valid values are `CLOUDFRONT` or `REGIONAL`. To work with CloudFront, you must also specify the region `us-east-1` (N. Virginia) on the AWS provider.
     */
    readonly scope: pulumi.Output<string>;
    /**
     * Map of key-value pairs to associate with the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * Map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * Specifies the domains that AWS WAF should accept in a web request token. This enables the use of tokens across multiple protected websites. When AWS WAF provides a token, it uses the domain of the AWS resource that the web ACL is protecting. If you don't specify a list of token domains, AWS WAF accepts tokens only for the domain of the protected resource. With a token domain list, AWS WAF accepts the resource's host domain plus all domains in the token domain list, including their prefixed subdomains.
     */
    readonly tokenDomains: pulumi.Output<string[] | undefined>;
    /**
     * Defines and enables Amazon CloudWatch metrics and web request sample collection. See `visibilityConfig` below for details.
     */
    readonly visibilityConfig: pulumi.Output<outputs.wafv2.WebAclVisibilityConfig>;
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
     * The URL to use in SDK integrations with managed rule groups.
     */
    applicationIntegrationUrl?: pulumi.Input<string>;
    /**
     * The ARN of the WAF WebACL.
     */
    arn?: pulumi.Input<string>;
    /**
     * Specifies custom configurations for the associations between the web ACL and protected resources. See `associationConfig` below for details.
     */
    associationConfig?: pulumi.Input<inputs.wafv2.WebAclAssociationConfig>;
    /**
     * Web ACL capacity units (WCUs) currently being used by this web ACL.
     */
    capacity?: pulumi.Input<number>;
    /**
     * Specifies how AWS WAF should handle CAPTCHA evaluations on the ACL level (used by [AWS Bot Control](https://docs.aws.amazon.com/waf/latest/developerguide/aws-managed-rule-groups-bot.html)). See `captchaConfig` below for details.
     */
    captchaConfig?: pulumi.Input<inputs.wafv2.WebAclCaptchaConfig>;
    /**
     * Specifies how AWS WAF should handle Challenge evaluations on the ACL level (used by [AWS Bot Control](https://docs.aws.amazon.com/waf/latest/developerguide/aws-managed-rule-groups-bot.html)). See `challengeConfig` below for details.
     */
    challengeConfig?: pulumi.Input<inputs.wafv2.WebAclChallengeConfig>;
    /**
     * Defines custom response bodies that can be referenced by `customResponse` actions. See `customResponseBody` below for details.
     */
    customResponseBodies?: pulumi.Input<pulumi.Input<inputs.wafv2.WebAclCustomResponseBody>[]>;
    /**
     * Specifies data protection to apply to the web request data for the web ACL. This is a web ACL level data protection option. See `dataProtectionConfig` below for details.
     */
    dataProtectionConfig?: pulumi.Input<inputs.wafv2.WebAclDataProtectionConfig>;
    /**
     * Action to perform if none of the `rules` contained in the WebACL match. See `defaultAction` below for details.
     */
    defaultAction?: pulumi.Input<inputs.wafv2.WebAclDefaultAction>;
    /**
     * Friendly description of the WebACL.
     */
    description?: pulumi.Input<string>;
    lockToken?: pulumi.Input<string>;
    /**
     * Friendly name of the WebACL. If omitted, the provider will assign a random, unique name. Conflicts with `namePrefix`.
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
     * Raw JSON string to allow more than three nested statements. Conflicts with `rule` attribute. This is for advanced use cases where more than 3 levels of nested statements are required. **There is no drift detection at this time**. If you use this attribute instead of `rule`, you will be foregoing drift detection. Additionally, importing an existing web ACL into a configuration with `ruleJson` set will result in a one time in-place update as the remote rule configuration is initially written to the `rule` attribute. See the AWS [documentation](https://docs.aws.amazon.com/waf/latest/APIReference/API_CreateWebACL.html) for the JSON structure.
     */
    ruleJson?: pulumi.Input<string>;
    /**
     * Rule blocks used to identify the web requests that you want to `allow`, `block`, or `count`. See `rule` below for details.
     */
    rules?: pulumi.Input<pulumi.Input<inputs.wafv2.WebAclRule>[]>;
    /**
     * Specifies whether this is for an AWS CloudFront distribution or for a regional application. Valid values are `CLOUDFRONT` or `REGIONAL`. To work with CloudFront, you must also specify the region `us-east-1` (N. Virginia) on the AWS provider.
     */
    scope?: pulumi.Input<string>;
    /**
     * Map of key-value pairs to associate with the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Specifies the domains that AWS WAF should accept in a web request token. This enables the use of tokens across multiple protected websites. When AWS WAF provides a token, it uses the domain of the AWS resource that the web ACL is protecting. If you don't specify a list of token domains, AWS WAF accepts tokens only for the domain of the protected resource. With a token domain list, AWS WAF accepts the resource's host domain plus all domains in the token domain list, including their prefixed subdomains.
     */
    tokenDomains?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Defines and enables Amazon CloudWatch metrics and web request sample collection. See `visibilityConfig` below for details.
     */
    visibilityConfig?: pulumi.Input<inputs.wafv2.WebAclVisibilityConfig>;
}
/**
 * The set of arguments for constructing a WebAcl resource.
 */
export interface WebAclArgs {
    /**
     * Specifies custom configurations for the associations between the web ACL and protected resources. See `associationConfig` below for details.
     */
    associationConfig?: pulumi.Input<inputs.wafv2.WebAclAssociationConfig>;
    /**
     * Specifies how AWS WAF should handle CAPTCHA evaluations on the ACL level (used by [AWS Bot Control](https://docs.aws.amazon.com/waf/latest/developerguide/aws-managed-rule-groups-bot.html)). See `captchaConfig` below for details.
     */
    captchaConfig?: pulumi.Input<inputs.wafv2.WebAclCaptchaConfig>;
    /**
     * Specifies how AWS WAF should handle Challenge evaluations on the ACL level (used by [AWS Bot Control](https://docs.aws.amazon.com/waf/latest/developerguide/aws-managed-rule-groups-bot.html)). See `challengeConfig` below for details.
     */
    challengeConfig?: pulumi.Input<inputs.wafv2.WebAclChallengeConfig>;
    /**
     * Defines custom response bodies that can be referenced by `customResponse` actions. See `customResponseBody` below for details.
     */
    customResponseBodies?: pulumi.Input<pulumi.Input<inputs.wafv2.WebAclCustomResponseBody>[]>;
    /**
     * Specifies data protection to apply to the web request data for the web ACL. This is a web ACL level data protection option. See `dataProtectionConfig` below for details.
     */
    dataProtectionConfig?: pulumi.Input<inputs.wafv2.WebAclDataProtectionConfig>;
    /**
     * Action to perform if none of the `rules` contained in the WebACL match. See `defaultAction` below for details.
     */
    defaultAction: pulumi.Input<inputs.wafv2.WebAclDefaultAction>;
    /**
     * Friendly description of the WebACL.
     */
    description?: pulumi.Input<string>;
    /**
     * Friendly name of the WebACL. If omitted, the provider will assign a random, unique name. Conflicts with `namePrefix`.
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
     * Raw JSON string to allow more than three nested statements. Conflicts with `rule` attribute. This is for advanced use cases where more than 3 levels of nested statements are required. **There is no drift detection at this time**. If you use this attribute instead of `rule`, you will be foregoing drift detection. Additionally, importing an existing web ACL into a configuration with `ruleJson` set will result in a one time in-place update as the remote rule configuration is initially written to the `rule` attribute. See the AWS [documentation](https://docs.aws.amazon.com/waf/latest/APIReference/API_CreateWebACL.html) for the JSON structure.
     */
    ruleJson?: pulumi.Input<string>;
    /**
     * Rule blocks used to identify the web requests that you want to `allow`, `block`, or `count`. See `rule` below for details.
     */
    rules?: pulumi.Input<pulumi.Input<inputs.wafv2.WebAclRule>[]>;
    /**
     * Specifies whether this is for an AWS CloudFront distribution or for a regional application. Valid values are `CLOUDFRONT` or `REGIONAL`. To work with CloudFront, you must also specify the region `us-east-1` (N. Virginia) on the AWS provider.
     */
    scope: pulumi.Input<string>;
    /**
     * Map of key-value pairs to associate with the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Specifies the domains that AWS WAF should accept in a web request token. This enables the use of tokens across multiple protected websites. When AWS WAF provides a token, it uses the domain of the AWS resource that the web ACL is protecting. If you don't specify a list of token domains, AWS WAF accepts tokens only for the domain of the protected resource. With a token domain list, AWS WAF accepts the resource's host domain plus all domains in the token domain list, including their prefixed subdomains.
     */
    tokenDomains?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Defines and enables Amazon CloudWatch metrics and web request sample collection. See `visibilityConfig` below for details.
     */
    visibilityConfig: pulumi.Input<inputs.wafv2.WebAclVisibilityConfig>;
}
