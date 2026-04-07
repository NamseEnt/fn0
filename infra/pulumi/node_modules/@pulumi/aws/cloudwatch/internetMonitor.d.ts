import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a Internet Monitor Monitor resource.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.cloudwatch.InternetMonitor("example", {monitorName: "exmple"});
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Internet Monitor Monitors using the `monitor_name`. For example:
 *
 * ```sh
 * $ pulumi import aws:cloudwatch/internetMonitor:InternetMonitor some some-monitor
 * ```
 */
export declare class InternetMonitor extends pulumi.CustomResource {
    /**
     * Get an existing InternetMonitor resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: InternetMonitorState, opts?: pulumi.CustomResourceOptions): InternetMonitor;
    /**
     * Returns true if the given object is an instance of InternetMonitor.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is InternetMonitor;
    /**
     * ARN of the Monitor.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Health event thresholds. A health event threshold percentage, for performance and availability, determines when Internet Monitor creates a health event when there's an internet issue that affects your application end users. See Health Events Config below.
     */
    readonly healthEventsConfig: pulumi.Output<outputs.cloudwatch.InternetMonitorHealthEventsConfig | undefined>;
    /**
     * Publish internet measurements for Internet Monitor to an Amazon S3 bucket in addition to CloudWatch Logs.
     */
    readonly internetMeasurementsLogDelivery: pulumi.Output<outputs.cloudwatch.InternetMonitorInternetMeasurementsLogDelivery | undefined>;
    /**
     * The maximum number of city-networks to monitor for your resources. A city-network is the location (city) where clients access your application resources from and the network or ASN, such as an internet service provider (ISP), that clients access the resources through. This limit helps control billing costs.
     */
    readonly maxCityNetworksToMonitor: pulumi.Output<number | undefined>;
    /**
     * The name of the monitor.
     *
     * The following arguments are optional:
     */
    readonly monitorName: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The resources to include in a monitor, which you provide as a set of Amazon Resource Names (ARNs).
     */
    readonly resources: pulumi.Output<string[] | undefined>;
    /**
     * The status for a monitor. The accepted values for Status with the UpdateMonitor API call are the following: `ACTIVE` and `INACTIVE`.
     */
    readonly status: pulumi.Output<string | undefined>;
    /**
     * Map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * The percentage of the internet-facing traffic for your application that you want to monitor with this monitor.
     */
    readonly trafficPercentageToMonitor: pulumi.Output<number | undefined>;
    /**
     * Create a InternetMonitor resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: InternetMonitorArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering InternetMonitor resources.
 */
export interface InternetMonitorState {
    /**
     * ARN of the Monitor.
     */
    arn?: pulumi.Input<string>;
    /**
     * Health event thresholds. A health event threshold percentage, for performance and availability, determines when Internet Monitor creates a health event when there's an internet issue that affects your application end users. See Health Events Config below.
     */
    healthEventsConfig?: pulumi.Input<inputs.cloudwatch.InternetMonitorHealthEventsConfig>;
    /**
     * Publish internet measurements for Internet Monitor to an Amazon S3 bucket in addition to CloudWatch Logs.
     */
    internetMeasurementsLogDelivery?: pulumi.Input<inputs.cloudwatch.InternetMonitorInternetMeasurementsLogDelivery>;
    /**
     * The maximum number of city-networks to monitor for your resources. A city-network is the location (city) where clients access your application resources from and the network or ASN, such as an internet service provider (ISP), that clients access the resources through. This limit helps control billing costs.
     */
    maxCityNetworksToMonitor?: pulumi.Input<number>;
    /**
     * The name of the monitor.
     *
     * The following arguments are optional:
     */
    monitorName?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The resources to include in a monitor, which you provide as a set of Amazon Resource Names (ARNs).
     */
    resources?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The status for a monitor. The accepted values for Status with the UpdateMonitor API call are the following: `ACTIVE` and `INACTIVE`.
     */
    status?: pulumi.Input<string>;
    /**
     * Map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * The percentage of the internet-facing traffic for your application that you want to monitor with this monitor.
     */
    trafficPercentageToMonitor?: pulumi.Input<number>;
}
/**
 * The set of arguments for constructing a InternetMonitor resource.
 */
export interface InternetMonitorArgs {
    /**
     * Health event thresholds. A health event threshold percentage, for performance and availability, determines when Internet Monitor creates a health event when there's an internet issue that affects your application end users. See Health Events Config below.
     */
    healthEventsConfig?: pulumi.Input<inputs.cloudwatch.InternetMonitorHealthEventsConfig>;
    /**
     * Publish internet measurements for Internet Monitor to an Amazon S3 bucket in addition to CloudWatch Logs.
     */
    internetMeasurementsLogDelivery?: pulumi.Input<inputs.cloudwatch.InternetMonitorInternetMeasurementsLogDelivery>;
    /**
     * The maximum number of city-networks to monitor for your resources. A city-network is the location (city) where clients access your application resources from and the network or ASN, such as an internet service provider (ISP), that clients access the resources through. This limit helps control billing costs.
     */
    maxCityNetworksToMonitor?: pulumi.Input<number>;
    /**
     * The name of the monitor.
     *
     * The following arguments are optional:
     */
    monitorName: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The resources to include in a monitor, which you provide as a set of Amazon Resource Names (ARNs).
     */
    resources?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The status for a monitor. The accepted values for Status with the UpdateMonitor API call are the following: `ACTIVE` and `INACTIVE`.
     */
    status?: pulumi.Input<string>;
    /**
     * Map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The percentage of the internet-facing traffic for your application that you want to monitor with this monitor.
     */
    trafficPercentageToMonitor?: pulumi.Input<number>;
}
