import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a CloudWatch RUM Metrics Destination resource.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.rum.MetricsDestination("example", {
 *     appMonitorName: exampleAwsRumAppMonitor.name,
 *     destination: "CloudWatch",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Cloudwatch RUM Metrics Destination using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:rum/metricsDestination:MetricsDestination example example
 * ```
 */
export declare class MetricsDestination extends pulumi.CustomResource {
    /**
     * Get an existing MetricsDestination resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: MetricsDestinationState, opts?: pulumi.CustomResourceOptions): MetricsDestination;
    /**
     * Returns true if the given object is an instance of MetricsDestination.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is MetricsDestination;
    /**
     * The name of the CloudWatch RUM app monitor that will send the metrics.
     */
    readonly appMonitorName: pulumi.Output<string>;
    /**
     * Defines the destination to send the metrics to. Valid values are `CloudWatch` and `Evidently`. If you specify `Evidently`, you must also specify the ARN of the CloudWatchEvidently experiment that is to be the destination and an IAM role that has permission to write to the experiment.
     */
    readonly destination: pulumi.Output<string>;
    /**
     * Use this parameter only if Destination is Evidently. This parameter specifies the ARN of the Evidently experiment that will receive the extended metrics.
     */
    readonly destinationArn: pulumi.Output<string | undefined>;
    /**
     * This parameter is required if Destination is Evidently. If Destination is CloudWatch, do not use this parameter.
     */
    readonly iamRoleArn: pulumi.Output<string | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Create a MetricsDestination resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: MetricsDestinationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering MetricsDestination resources.
 */
export interface MetricsDestinationState {
    /**
     * The name of the CloudWatch RUM app monitor that will send the metrics.
     */
    appMonitorName?: pulumi.Input<string>;
    /**
     * Defines the destination to send the metrics to. Valid values are `CloudWatch` and `Evidently`. If you specify `Evidently`, you must also specify the ARN of the CloudWatchEvidently experiment that is to be the destination and an IAM role that has permission to write to the experiment.
     */
    destination?: pulumi.Input<string>;
    /**
     * Use this parameter only if Destination is Evidently. This parameter specifies the ARN of the Evidently experiment that will receive the extended metrics.
     */
    destinationArn?: pulumi.Input<string>;
    /**
     * This parameter is required if Destination is Evidently. If Destination is CloudWatch, do not use this parameter.
     */
    iamRoleArn?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a MetricsDestination resource.
 */
export interface MetricsDestinationArgs {
    /**
     * The name of the CloudWatch RUM app monitor that will send the metrics.
     */
    appMonitorName: pulumi.Input<string>;
    /**
     * Defines the destination to send the metrics to. Valid values are `CloudWatch` and `Evidently`. If you specify `Evidently`, you must also specify the ARN of the CloudWatchEvidently experiment that is to be the destination and an IAM role that has permission to write to the experiment.
     */
    destination: pulumi.Input<string>;
    /**
     * Use this parameter only if Destination is Evidently. This parameter specifies the ARN of the Evidently experiment that will receive the extended metrics.
     */
    destinationArn?: pulumi.Input<string>;
    /**
     * This parameter is required if Destination is Evidently. If Destination is CloudWatch, do not use this parameter.
     */
    iamRoleArn?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
