import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages an App Runner Observability Configuration.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.apprunner.ObservabilityConfiguration("example", {
 *     observabilityConfigurationName: "example",
 *     traceConfiguration: {
 *         vendor: "AWSXRAY",
 *     },
 *     tags: {
 *         Name: "example-apprunner-observability-configuration",
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
 * - `arn` (String) Amazon Resource Name (ARN) of the App Runner observability configuration.
 *
 * Using `pulumi import`, import App Runner Observability Configuration using the `arn`. For example:
 *
 * console
 *
 * % pulumi import aws_apprunner_observability_configuration.example arn:aws:apprunner:us-east-1:1234567890:observabilityconfiguration/example/1/d75bc7ea55b71e724fe5c23452fe22a1
 */
export declare class ObservabilityConfiguration extends pulumi.CustomResource {
    /**
     * Get an existing ObservabilityConfiguration resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ObservabilityConfigurationState, opts?: pulumi.CustomResourceOptions): ObservabilityConfiguration;
    /**
     * Returns true if the given object is an instance of ObservabilityConfiguration.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ObservabilityConfiguration;
    /**
     * ARN of this observability configuration.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Whether the observability configuration has the highest `observabilityConfigurationRevision` among all configurations that share the same `observabilityConfigurationName`.
     */
    readonly latest: pulumi.Output<boolean>;
    /**
     * Name of the observability configuration.
     */
    readonly observabilityConfigurationName: pulumi.Output<string>;
    /**
     * The revision of this observability configuration.
     */
    readonly observabilityConfigurationRevision: pulumi.Output<number>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Current state of the observability configuration. An INACTIVE configuration revision has been deleted and can't be used. It is permanently removed some time after deletion.
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
     * Configuration of the tracing feature within this observability configuration. If you don't specify it, App Runner doesn't enable tracing. See Trace Configuration below for more details.
     */
    readonly traceConfiguration: pulumi.Output<outputs.apprunner.ObservabilityConfigurationTraceConfiguration | undefined>;
    /**
     * Create a ObservabilityConfiguration resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ObservabilityConfigurationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ObservabilityConfiguration resources.
 */
export interface ObservabilityConfigurationState {
    /**
     * ARN of this observability configuration.
     */
    arn?: pulumi.Input<string>;
    /**
     * Whether the observability configuration has the highest `observabilityConfigurationRevision` among all configurations that share the same `observabilityConfigurationName`.
     */
    latest?: pulumi.Input<boolean>;
    /**
     * Name of the observability configuration.
     */
    observabilityConfigurationName?: pulumi.Input<string>;
    /**
     * The revision of this observability configuration.
     */
    observabilityConfigurationRevision?: pulumi.Input<number>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Current state of the observability configuration. An INACTIVE configuration revision has been deleted and can't be used. It is permanently removed some time after deletion.
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
    /**
     * Configuration of the tracing feature within this observability configuration. If you don't specify it, App Runner doesn't enable tracing. See Trace Configuration below for more details.
     */
    traceConfiguration?: pulumi.Input<inputs.apprunner.ObservabilityConfigurationTraceConfiguration>;
}
/**
 * The set of arguments for constructing a ObservabilityConfiguration resource.
 */
export interface ObservabilityConfigurationArgs {
    /**
     * Name of the observability configuration.
     */
    observabilityConfigurationName: pulumi.Input<string>;
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
    /**
     * Configuration of the tracing feature within this observability configuration. If you don't specify it, App Runner doesn't enable tracing. See Trace Configuration below for more details.
     */
    traceConfiguration?: pulumi.Input<inputs.apprunner.ObservabilityConfigurationTraceConfiguration>;
}
