import * as pulumi from "@pulumi/pulumi";
/**
 * Provides an AWS Route 53 Recovery Readiness Readiness Check.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.route53recoveryreadiness.ReadinessCheck("example", {
 *     readinessCheckName: my_cw_alarm_check,
 *     resourceSetName: my_cw_alarm_set,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Route53 Recovery Readiness readiness checks using the readiness check name. For example:
 *
 * ```sh
 * $ pulumi import aws:route53recoveryreadiness/readinessCheck:ReadinessCheck my-cw-alarm-check example
 * ```
 */
export declare class ReadinessCheck extends pulumi.CustomResource {
    /**
     * Get an existing ReadinessCheck resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ReadinessCheckState, opts?: pulumi.CustomResourceOptions): ReadinessCheck;
    /**
     * Returns true if the given object is an instance of ReadinessCheck.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ReadinessCheck;
    /**
     * ARN of the readiness_check
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Unique name describing the readiness check.
     */
    readonly readinessCheckName: pulumi.Output<string>;
    /**
     * Name describing the resource set that will be monitored for readiness.
     *
     * The following arguments are optional:
     */
    readonly resourceSetName: pulumi.Output<string>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level
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
     * Create a ReadinessCheck resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ReadinessCheckArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ReadinessCheck resources.
 */
export interface ReadinessCheckState {
    /**
     * ARN of the readiness_check
     */
    arn?: pulumi.Input<string>;
    /**
     * Unique name describing the readiness check.
     */
    readinessCheckName?: pulumi.Input<string>;
    /**
     * Name describing the resource set that will be monitored for readiness.
     *
     * The following arguments are optional:
     */
    resourceSetName?: pulumi.Input<string>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level
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
}
/**
 * The set of arguments for constructing a ReadinessCheck resource.
 */
export interface ReadinessCheckArgs {
    /**
     * Unique name describing the readiness check.
     */
    readinessCheckName: pulumi.Input<string>;
    /**
     * Name describing the resource set that will be monitored for readiness.
     *
     * The following arguments are optional:
     */
    resourceSetName: pulumi.Input<string>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
