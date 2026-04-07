import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a CE Anomaly Monitor.
 *
 * ## Example Usage
 *
 * There are two main types of a Cost Anomaly Monitor: `DIMENSIONAL` and `CUSTOM`.
 *
 * ### Dimensional Example
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const serviceMonitor = new aws.costexplorer.AnomalyMonitor("service_monitor", {
 *     name: "AWSServiceMonitor",
 *     monitorType: "DIMENSIONAL",
 *     monitorDimension: "SERVICE",
 * });
 * ```
 *
 * ### Custom Example
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = new aws.costexplorer.AnomalyMonitor("test", {
 *     name: "AWSCustomAnomalyMonitor",
 *     monitorType: "CUSTOM",
 *     monitorSpecification: JSON.stringify({
 *         And: null,
 *         CostCategories: null,
 *         Dimensions: null,
 *         Not: null,
 *         Or: null,
 *         Tags: {
 *             Key: "CostCenter",
 *             MatchOptions: null,
 *             Values: ["10000"],
 *         },
 *     }),
 * });
 * ```
 *
 * ## Import
 *
 * ### Identity Schema
 *
 * #### Required
 *
 * - `arn` (String) Amazon Resource Name (ARN) of the Cost Explorer anomaly monitor.
 *
 * Using `pulumi import`, import `aws_ce_anomaly_monitor` using the `id`. For example:
 *
 * console
 *
 * % pulumi import aws_ce_anomaly_monitor.example costAnomalyMonitorARN
 */
export declare class AnomalyMonitor extends pulumi.CustomResource {
    /**
     * Get an existing AnomalyMonitor resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: AnomalyMonitorState, opts?: pulumi.CustomResourceOptions): AnomalyMonitor;
    /**
     * Returns true if the given object is an instance of AnomalyMonitor.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is AnomalyMonitor;
    /**
     * ARN of the anomaly monitor.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The dimensions to evaluate. Valid values: `SERVICE`.
     */
    readonly monitorDimension: pulumi.Output<string | undefined>;
    /**
     * A valid JSON representation for the [Expression](https://docs.aws.amazon.com/aws-cost-management/latest/APIReference/API_Expression.html) object.
     */
    readonly monitorSpecification: pulumi.Output<string | undefined>;
    /**
     * The possible type values. Valid values: `DIMENSIONAL` | `CUSTOM`.
     */
    readonly monitorType: pulumi.Output<string>;
    /**
     * The name of the monitor.
     */
    readonly name: pulumi.Output<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Create a AnomalyMonitor resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: AnomalyMonitorArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering AnomalyMonitor resources.
 */
export interface AnomalyMonitorState {
    /**
     * ARN of the anomaly monitor.
     */
    arn?: pulumi.Input<string>;
    /**
     * The dimensions to evaluate. Valid values: `SERVICE`.
     */
    monitorDimension?: pulumi.Input<string>;
    /**
     * A valid JSON representation for the [Expression](https://docs.aws.amazon.com/aws-cost-management/latest/APIReference/API_Expression.html) object.
     */
    monitorSpecification?: pulumi.Input<string>;
    /**
     * The possible type values. Valid values: `DIMENSIONAL` | `CUSTOM`.
     */
    monitorType?: pulumi.Input<string>;
    /**
     * The name of the monitor.
     */
    name?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
 * The set of arguments for constructing a AnomalyMonitor resource.
 */
export interface AnomalyMonitorArgs {
    /**
     * The dimensions to evaluate. Valid values: `SERVICE`.
     */
    monitorDimension?: pulumi.Input<string>;
    /**
     * A valid JSON representation for the [Expression](https://docs.aws.amazon.com/aws-cost-management/latest/APIReference/API_Expression.html) object.
     */
    monitorSpecification?: pulumi.Input<string>;
    /**
     * The possible type values. Valid values: `DIMENSIONAL` | `CUSTOM`.
     */
    monitorType: pulumi.Input<string>;
    /**
     * The name of the monitor.
     */
    name?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
