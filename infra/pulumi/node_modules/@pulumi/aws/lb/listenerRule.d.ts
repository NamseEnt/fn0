import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a Load Balancer Listener Rule resource.
 *
 * > **Note:** `aws.alb.ListenerRule` is known as `aws.lb.ListenerRule`. The functionality is identical.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const frontEnd = new aws.lb.LoadBalancer("front_end", {});
 * const frontEndListener = new aws.lb.Listener("front_end", {});
 * const static = new aws.lb.ListenerRule("static", {
 *     listenerArn: frontEndListener.arn,
 *     priority: 100,
 *     actions: [{
 *         type: "forward",
 *         targetGroupArn: staticAwsLbTargetGroup.arn,
 *     }],
 *     conditions: [
 *         {
 *             pathPattern: {
 *                 values: ["/static/*"],
 *             },
 *         },
 *         {
 *             hostHeader: {
 *                 values: ["example.com"],
 *             },
 *         },
 *     ],
 * });
 * // Forward action
 * const hostBasedWeightedRouting = new aws.lb.ListenerRule("host_based_weighted_routing", {
 *     listenerArn: frontEndListener.arn,
 *     priority: 99,
 *     actions: [{
 *         type: "forward",
 *         targetGroupArn: staticAwsLbTargetGroup.arn,
 *     }],
 *     conditions: [{
 *         hostHeader: {
 *             values: ["my-service.*.mycompany.io"],
 *         },
 *     }],
 * });
 * // Weighted Forward action
 * const hostBasedRouting = new aws.lb.ListenerRule("host_based_routing", {
 *     listenerArn: frontEndListener.arn,
 *     priority: 99,
 *     actions: [{
 *         type: "forward",
 *         forward: {
 *             targetGroups: [
 *                 {
 *                     arn: main.arn,
 *                     weight: 80,
 *                 },
 *                 {
 *                     arn: canary.arn,
 *                     weight: 20,
 *                 },
 *             ],
 *             stickiness: {
 *                 enabled: true,
 *                 duration: 600,
 *             },
 *         },
 *     }],
 *     conditions: [{
 *         hostHeader: {
 *             values: ["my-service.*.mycompany.io"],
 *         },
 *     }],
 * });
 * // Redirect action
 * const redirectHttpToHttps = new aws.lb.ListenerRule("redirect_http_to_https", {
 *     listenerArn: frontEndListener.arn,
 *     actions: [{
 *         type: "redirect",
 *         redirect: {
 *             port: "443",
 *             protocol: "HTTPS",
 *             statusCode: "HTTP_301",
 *         },
 *     }],
 *     conditions: [{
 *         httpHeader: {
 *             httpHeaderName: "X-Forwarded-For",
 *             values: ["192.168.1.*"],
 *         },
 *     }],
 * });
 * // Fixed-response action
 * const healthCheck = new aws.lb.ListenerRule("health_check", {
 *     listenerArn: frontEndListener.arn,
 *     actions: [{
 *         type: "fixed-response",
 *         fixedResponse: {
 *             contentType: "text/plain",
 *             messageBody: "HEALTHY",
 *             statusCode: "200",
 *         },
 *     }],
 *     conditions: [{
 *         queryStrings: [
 *             {
 *                 key: "health",
 *                 value: "check",
 *             },
 *             {
 *                 value: "bar",
 *             },
 *         ],
 *     }],
 * });
 * // Authenticate-cognito Action
 * const pool = new aws.cognito.UserPool("pool", {});
 * const client = new aws.cognito.UserPoolClient("client", {});
 * const domain = new aws.cognito.UserPoolDomain("domain", {});
 * const admin = new aws.lb.ListenerRule("admin", {
 *     listenerArn: frontEndListener.arn,
 *     actions: [
 *         {
 *             type: "authenticate-cognito",
 *             authenticateCognito: {
 *                 userPoolArn: pool.arn,
 *                 userPoolClientId: client.id,
 *                 userPoolDomain: domain.domain,
 *             },
 *         },
 *         {
 *             type: "forward",
 *             targetGroupArn: staticAwsLbTargetGroup.arn,
 *         },
 *     ],
 * });
 * // Authenticate-oidc Action
 * const oidc = new aws.lb.ListenerRule("oidc", {
 *     listenerArn: frontEndListener.arn,
 *     actions: [
 *         {
 *             type: "authenticate-oidc",
 *             authenticateOidc: {
 *                 authorizationEndpoint: "https://example.com/authorization_endpoint",
 *                 clientId: "client_id",
 *                 clientSecret: "client_secret",
 *                 issuer: "https://example.com",
 *                 tokenEndpoint: "https://example.com/token_endpoint",
 *                 userInfoEndpoint: "https://example.com/user_info_endpoint",
 *             },
 *         },
 *         {
 *             type: "forward",
 *             targetGroupArn: staticAwsLbTargetGroup.arn,
 *         },
 *     ],
 * });
 * // With transform
 * const transform = new aws.lb.ListenerRule("transform", {
 *     listenerArn: frontEndListener.arn,
 *     actions: [{
 *         type: "forward",
 *         targetGroupArn: staticAwsLbTargetGroup.arn,
 *     }],
 *     conditions: [{
 *         pathPattern: {
 *             values: ["*"],
 *         },
 *     }],
 *     transforms: [
 *         {
 *             type: "host-header-rewrite",
 *             hostHeaderRewriteConfig: {
 *                 rewrite: {
 *                     regex: "^mywebsite-(.+).com$",
 *                     replace: "internal.dev.$1.myweb.com",
 *                 },
 *             },
 *         },
 *         {
 *             type: "url-rewrite",
 *             urlRewriteConfig: {
 *                 rewrite: {
 *                     regex: "^/dp/([A-Za-z0-9]+)/?$",
 *                     replace: "/product.php?id=$1",
 *                 },
 *             },
 *         },
 *     ],
 * });
 * ```
 *
 * ## Import
 *
 * ### Identity Schema
 *
 * #### Required
 *
 * - `arn` (String) Amazon Resource Name (ARN) of the load balancer listener rule.
 *
 * Using `pulumi import`, import rules using their ARN. For example:
 *
 * console
 *
 * % pulumi import aws_lb_listener_rule.front_end arn:aws:elasticloadbalancing:us-west-2:187416307283:listener-rule/app/test/8e4497da625e2d8a/9ab28ade35828f96/67b3d2d36dd7c26b
 */
export declare class ListenerRule extends pulumi.CustomResource {
    /**
     * Get an existing ListenerRule resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ListenerRuleState, opts?: pulumi.CustomResourceOptions): ListenerRule;
    /**
     * Returns true if the given object is an instance of ListenerRule.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ListenerRule;
    /**
     * An Action block. Action blocks are documented below.
     */
    readonly actions: pulumi.Output<outputs.lb.ListenerRuleAction[]>;
    /**
     * The ARN of the rule (matches `id`)
     */
    readonly arn: pulumi.Output<string>;
    /**
     * A Condition block. Multiple condition blocks of different types can be set and all must be satisfied for the rule to match. Condition blocks are documented below.
     */
    readonly conditions: pulumi.Output<outputs.lb.ListenerRuleCondition[]>;
    /**
     * The ARN of the listener to which to attach the rule.
     */
    readonly listenerArn: pulumi.Output<string>;
    /**
     * The priority for the rule between `1` and `50000`. Leaving it unset will automatically set the rule with next available priority after currently existing highest rule. A listener can't have multiple rules with the same priority.
     */
    readonly priority: pulumi.Output<number>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * A map of tags to assign to the resource. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Configuration block that defines the transform to apply to requests matching this rule. See Transform Blocks below for more details. Once specified, to remove the transform from the rule, remove the `transform` block from the configuration.
     */
    readonly transforms: pulumi.Output<outputs.lb.ListenerRuleTransform[] | undefined>;
    /**
     * Create a ListenerRule resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ListenerRuleArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ListenerRule resources.
 */
export interface ListenerRuleState {
    /**
     * An Action block. Action blocks are documented below.
     */
    actions?: pulumi.Input<pulumi.Input<inputs.lb.ListenerRuleAction>[]>;
    /**
     * The ARN of the rule (matches `id`)
     */
    arn?: pulumi.Input<string>;
    /**
     * A Condition block. Multiple condition blocks of different types can be set and all must be satisfied for the rule to match. Condition blocks are documented below.
     */
    conditions?: pulumi.Input<pulumi.Input<inputs.lb.ListenerRuleCondition>[]>;
    /**
     * The ARN of the listener to which to attach the rule.
     */
    listenerArn?: pulumi.Input<string>;
    /**
     * The priority for the rule between `1` and `50000`. Leaving it unset will automatically set the rule with next available priority after currently existing highest rule. A listener can't have multiple rules with the same priority.
     */
    priority?: pulumi.Input<number>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Configuration block that defines the transform to apply to requests matching this rule. See Transform Blocks below for more details. Once specified, to remove the transform from the rule, remove the `transform` block from the configuration.
     */
    transforms?: pulumi.Input<pulumi.Input<inputs.lb.ListenerRuleTransform>[]>;
}
/**
 * The set of arguments for constructing a ListenerRule resource.
 */
export interface ListenerRuleArgs {
    /**
     * An Action block. Action blocks are documented below.
     */
    actions: pulumi.Input<pulumi.Input<inputs.lb.ListenerRuleAction>[]>;
    /**
     * A Condition block. Multiple condition blocks of different types can be set and all must be satisfied for the rule to match. Condition blocks are documented below.
     */
    conditions: pulumi.Input<pulumi.Input<inputs.lb.ListenerRuleCondition>[]>;
    /**
     * The ARN of the listener to which to attach the rule.
     */
    listenerArn: pulumi.Input<string>;
    /**
     * The priority for the rule between `1` and `50000`. Leaving it unset will automatically set the rule with next available priority after currently existing highest rule. A listener can't have multiple rules with the same priority.
     */
    priority?: pulumi.Input<number>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Configuration block that defines the transform to apply to requests matching this rule. See Transform Blocks below for more details. Once specified, to remove the transform from the rule, remove the `transform` block from the configuration.
     */
    transforms?: pulumi.Input<pulumi.Input<inputs.lb.ListenerRuleTransform>[]>;
}
