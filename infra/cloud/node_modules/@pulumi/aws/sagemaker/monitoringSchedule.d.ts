import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a SageMaker AI monitoring schedule resource.
 *
 * ## Example Usage
 *
 * Basic usage:
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = new aws.sagemaker.MonitoringSchedule("test", {
 *     name: "my-monitoring-schedule",
 *     monitoringScheduleConfig: {
 *         monitoringJobDefinitionName: testAwsSagemakerDataQualityJobDefinition.name,
 *         monitoringType: "DataQuality",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import monitoring schedules using the `name`. For example:
 *
 * ```sh
 * $ pulumi import aws:sagemaker/monitoringSchedule:MonitoringSchedule test_monitoring_schedule monitoring-schedule-foo
 * ```
 */
export declare class MonitoringSchedule extends pulumi.CustomResource {
    /**
     * Get an existing MonitoringSchedule resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: MonitoringScheduleState, opts?: pulumi.CustomResourceOptions): MonitoringSchedule;
    /**
     * Returns true if the given object is an instance of MonitoringSchedule.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is MonitoringSchedule;
    /**
     * The Amazon Resource Name (ARN) assigned by AWS to this monitoring schedule.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The configuration object that specifies the monitoring schedule and defines the monitoring job. Fields are documented below.
     */
    readonly monitoringScheduleConfig: pulumi.Output<outputs.sagemaker.MonitoringScheduleMonitoringScheduleConfig>;
    /**
     * The name of the monitoring schedule. The name must be unique within an AWS Region within an AWS account. If omitted, the provider will assign a random, unique name.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * A mapping of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Create a MonitoringSchedule resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: MonitoringScheduleArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering MonitoringSchedule resources.
 */
export interface MonitoringScheduleState {
    /**
     * The Amazon Resource Name (ARN) assigned by AWS to this monitoring schedule.
     */
    arn?: pulumi.Input<string>;
    /**
     * The configuration object that specifies the monitoring schedule and defines the monitoring job. Fields are documented below.
     */
    monitoringScheduleConfig?: pulumi.Input<inputs.sagemaker.MonitoringScheduleMonitoringScheduleConfig>;
    /**
     * The name of the monitoring schedule. The name must be unique within an AWS Region within an AWS account. If omitted, the provider will assign a random, unique name.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A mapping of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
 * The set of arguments for constructing a MonitoringSchedule resource.
 */
export interface MonitoringScheduleArgs {
    /**
     * The configuration object that specifies the monitoring schedule and defines the monitoring job. Fields are documented below.
     */
    monitoringScheduleConfig: pulumi.Input<inputs.sagemaker.MonitoringScheduleMonitoringScheduleConfig>;
    /**
     * The name of the monitoring schedule. The name must be unique within an AWS Region within an AWS account. If omitted, the provider will assign a random, unique name.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A mapping of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
