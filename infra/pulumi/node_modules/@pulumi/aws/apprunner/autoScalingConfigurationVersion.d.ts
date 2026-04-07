import * as pulumi from "@pulumi/pulumi";
/**
 * Manages an App Runner AutoScaling Configuration Version.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.apprunner.AutoScalingConfigurationVersion("example", {
 *     autoScalingConfigurationName: "example",
 *     maxConcurrency: 50,
 *     maxSize: 10,
 *     minSize: 2,
 *     tags: {
 *         Name: "example-apprunner-autoscaling",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * ### Identity Schema
 *
 * #### Required
 *
 * - `arn` (String) Amazon Resource Name (ARN) of the App Runner auto scaling configuration version.
 *
 * Using `pulumi import`, import App Runner AutoScaling Configuration Versions using the `arn`. For example:
 *
 * console
 *
 * % pulumi import aws_apprunner_auto_scaling_configuration_version.example "arn:aws:apprunner:us-east-1:1234567890:autoscalingconfiguration/example/1/69bdfe0115224b0db49398b7beb68e0f
 */
export declare class AutoScalingConfigurationVersion extends pulumi.CustomResource {
    /**
     * Get an existing AutoScalingConfigurationVersion resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: AutoScalingConfigurationVersionState, opts?: pulumi.CustomResourceOptions): AutoScalingConfigurationVersion;
    /**
     * Returns true if the given object is an instance of AutoScalingConfigurationVersion.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is AutoScalingConfigurationVersion;
    /**
     * ARN of this auto scaling configuration version.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Name of the auto scaling configuration.
     */
    readonly autoScalingConfigurationName: pulumi.Output<string>;
    /**
     * The revision of this auto scaling configuration.
     */
    readonly autoScalingConfigurationRevision: pulumi.Output<number>;
    readonly hasAssociatedService: pulumi.Output<boolean>;
    readonly isDefault: pulumi.Output<boolean>;
    /**
     * Whether the auto scaling configuration has the highest `autoScalingConfigurationRevision` among all configurations that share the same `autoScalingConfigurationName`.
     */
    readonly latest: pulumi.Output<boolean>;
    /**
     * Maximal number of concurrent requests that you want an instance to process. When the number of concurrent requests goes over this limit, App Runner scales up your service.
     */
    readonly maxConcurrency: pulumi.Output<number | undefined>;
    /**
     * Maximal number of instances that App Runner provisions for your service.
     */
    readonly maxSize: pulumi.Output<number | undefined>;
    /**
     * Minimal number of instances that App Runner provisions for your service.
     */
    readonly minSize: pulumi.Output<number | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Current state of the auto scaling configuration. An INACTIVE configuration revision has been deleted and can't be used. It is permanently removed some time after deletion.
     */
    readonly status: pulumi.Output<string>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Create a AutoScalingConfigurationVersion resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: AutoScalingConfigurationVersionArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering AutoScalingConfigurationVersion resources.
 */
export interface AutoScalingConfigurationVersionState {
    /**
     * ARN of this auto scaling configuration version.
     */
    arn?: pulumi.Input<string>;
    /**
     * Name of the auto scaling configuration.
     */
    autoScalingConfigurationName?: pulumi.Input<string>;
    /**
     * The revision of this auto scaling configuration.
     */
    autoScalingConfigurationRevision?: pulumi.Input<number>;
    hasAssociatedService?: pulumi.Input<boolean>;
    isDefault?: pulumi.Input<boolean>;
    /**
     * Whether the auto scaling configuration has the highest `autoScalingConfigurationRevision` among all configurations that share the same `autoScalingConfigurationName`.
     */
    latest?: pulumi.Input<boolean>;
    /**
     * Maximal number of concurrent requests that you want an instance to process. When the number of concurrent requests goes over this limit, App Runner scales up your service.
     */
    maxConcurrency?: pulumi.Input<number>;
    /**
     * Maximal number of instances that App Runner provisions for your service.
     */
    maxSize?: pulumi.Input<number>;
    /**
     * Minimal number of instances that App Runner provisions for your service.
     */
    minSize?: pulumi.Input<number>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Current state of the auto scaling configuration. An INACTIVE configuration revision has been deleted and can't be used. It is permanently removed some time after deletion.
     */
    status?: pulumi.Input<string>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
 * The set of arguments for constructing a AutoScalingConfigurationVersion resource.
 */
export interface AutoScalingConfigurationVersionArgs {
    /**
     * Name of the auto scaling configuration.
     */
    autoScalingConfigurationName: pulumi.Input<string>;
    /**
     * Maximal number of concurrent requests that you want an instance to process. When the number of concurrent requests goes over this limit, App Runner scales up your service.
     */
    maxConcurrency?: pulumi.Input<number>;
    /**
     * Maximal number of instances that App Runner provisions for your service.
     */
    maxSize?: pulumi.Input<number>;
    /**
     * Minimal number of instances that App Runner provisions for your service.
     */
    minSize?: pulumi.Input<number>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
