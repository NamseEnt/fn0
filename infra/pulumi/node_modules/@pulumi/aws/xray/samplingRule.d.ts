import * as pulumi from "@pulumi/pulumi";
/**
 * Creates and manages an AWS XRay Sampling Rule.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.xray.SamplingRule("example", {
 *     ruleName: "example",
 *     priority: 9999,
 *     version: 1,
 *     reservoirSize: 1,
 *     fixedRate: 0.05,
 *     urlPath: "*",
 *     host: "*",
 *     httpMethod: "*",
 *     serviceType: "*",
 *     serviceName: "*",
 *     resourceArn: "*",
 *     attributes: {
 *         Hello: "Tris",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import XRay Sampling Rules using the name. For example:
 *
 * ```sh
 * $ pulumi import aws:xray/samplingRule:SamplingRule example example
 * ```
 */
export declare class SamplingRule extends pulumi.CustomResource {
    /**
     * Get an existing SamplingRule resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: SamplingRuleState, opts?: pulumi.CustomResourceOptions): SamplingRule;
    /**
     * Returns true if the given object is an instance of SamplingRule.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is SamplingRule;
    /**
     * The ARN of the sampling rule.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Matches attributes derived from the request.
     */
    readonly attributes: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * The percentage of matching requests to instrument, after the reservoir is exhausted.
     */
    readonly fixedRate: pulumi.Output<number>;
    /**
     * Matches the hostname from a request URL.
     */
    readonly host: pulumi.Output<string>;
    /**
     * Matches the HTTP method of a request.
     */
    readonly httpMethod: pulumi.Output<string>;
    /**
     * The priority of the sampling rule.
     */
    readonly priority: pulumi.Output<number>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * A fixed number of matching requests to instrument per second, prior to applying the fixed rate. The reservoir is not used directly by services, but applies to all services using the rule collectively.
     */
    readonly reservoirSize: pulumi.Output<number>;
    /**
     * Matches the ARN of the AWS resource on which the service runs.
     */
    readonly resourceArn: pulumi.Output<string>;
    /**
     * The name of the sampling rule.
     */
    readonly ruleName: pulumi.Output<string | undefined>;
    /**
     * Matches the `name` that the service uses to identify itself in segments.
     */
    readonly serviceName: pulumi.Output<string>;
    /**
     * Matches the `origin` that the service uses to identify its type in segments.
     */
    readonly serviceType: pulumi.Output<string>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level
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
     * Matches the path from a request URL.
     */
    readonly urlPath: pulumi.Output<string>;
    /**
     * The version of the sampling rule format (`1` )
     */
    readonly version: pulumi.Output<number>;
    /**
     * Create a SamplingRule resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: SamplingRuleArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering SamplingRule resources.
 */
export interface SamplingRuleState {
    /**
     * The ARN of the sampling rule.
     */
    arn?: pulumi.Input<string>;
    /**
     * Matches attributes derived from the request.
     */
    attributes?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The percentage of matching requests to instrument, after the reservoir is exhausted.
     */
    fixedRate?: pulumi.Input<number>;
    /**
     * Matches the hostname from a request URL.
     */
    host?: pulumi.Input<string>;
    /**
     * Matches the HTTP method of a request.
     */
    httpMethod?: pulumi.Input<string>;
    /**
     * The priority of the sampling rule.
     */
    priority?: pulumi.Input<number>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A fixed number of matching requests to instrument per second, prior to applying the fixed rate. The reservoir is not used directly by services, but applies to all services using the rule collectively.
     */
    reservoirSize?: pulumi.Input<number>;
    /**
     * Matches the ARN of the AWS resource on which the service runs.
     */
    resourceArn?: pulumi.Input<string>;
    /**
     * The name of the sampling rule.
     */
    ruleName?: pulumi.Input<string>;
    /**
     * Matches the `name` that the service uses to identify itself in segments.
     */
    serviceName?: pulumi.Input<string>;
    /**
     * Matches the `origin` that the service uses to identify its type in segments.
     */
    serviceType?: pulumi.Input<string>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level
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
     * Matches the path from a request URL.
     */
    urlPath?: pulumi.Input<string>;
    /**
     * The version of the sampling rule format (`1` )
     */
    version?: pulumi.Input<number>;
}
/**
 * The set of arguments for constructing a SamplingRule resource.
 */
export interface SamplingRuleArgs {
    /**
     * Matches attributes derived from the request.
     */
    attributes?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The percentage of matching requests to instrument, after the reservoir is exhausted.
     */
    fixedRate: pulumi.Input<number>;
    /**
     * Matches the hostname from a request URL.
     */
    host: pulumi.Input<string>;
    /**
     * Matches the HTTP method of a request.
     */
    httpMethod: pulumi.Input<string>;
    /**
     * The priority of the sampling rule.
     */
    priority: pulumi.Input<number>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A fixed number of matching requests to instrument per second, prior to applying the fixed rate. The reservoir is not used directly by services, but applies to all services using the rule collectively.
     */
    reservoirSize: pulumi.Input<number>;
    /**
     * Matches the ARN of the AWS resource on which the service runs.
     */
    resourceArn: pulumi.Input<string>;
    /**
     * The name of the sampling rule.
     */
    ruleName?: pulumi.Input<string>;
    /**
     * Matches the `name` that the service uses to identify itself in segments.
     */
    serviceName: pulumi.Input<string>;
    /**
     * Matches the `origin` that the service uses to identify its type in segments.
     */
    serviceType: pulumi.Input<string>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Matches the path from a request URL.
     */
    urlPath: pulumi.Input<string>;
    /**
     * The version of the sampling rule format (`1` )
     */
    version: pulumi.Input<number>;
}
