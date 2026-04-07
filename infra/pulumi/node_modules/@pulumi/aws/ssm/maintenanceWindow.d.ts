import * as pulumi from "@pulumi/pulumi";
/**
 * Provides an SSM Maintenance Window resource
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const production = new aws.ssm.MaintenanceWindow("production", {
 *     name: "maintenance-window-application",
 *     schedule: "cron(0 16 ? * TUE *)",
 *     duration: 3,
 *     cutoff: 1,
 * });
 * ```
 *
 * ## Import
 *
 * ### Identity Schema
 *
 * #### Required
 *
 * * `id` - (String) ID of the maintenance window.
 *
 * #### Optional
 *
 * * `account_id` (String) AWS Account where this resource is managed.
 *
 * * `region` (String) Region where this resource is managed.
 *
 * Using `pulumi import`, import SSM  Maintenance Windows using the maintenance window `id`. For example:
 *
 * console
 *
 * % pulumi import aws_ssm_maintenance_window.example mw-0123456789
 */
export declare class MaintenanceWindow extends pulumi.CustomResource {
    /**
     * Get an existing MaintenanceWindow resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: MaintenanceWindowState, opts?: pulumi.CustomResourceOptions): MaintenanceWindow;
    /**
     * Returns true if the given object is an instance of MaintenanceWindow.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is MaintenanceWindow;
    /**
     * Whether targets must be registered with the Maintenance Window before tasks can be defined for those targets.
     */
    readonly allowUnassociatedTargets: pulumi.Output<boolean | undefined>;
    /**
     * The number of hours before the end of the Maintenance Window that Systems Manager stops scheduling new tasks for execution.
     */
    readonly cutoff: pulumi.Output<number>;
    /**
     * A description for the maintenance window.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * The duration of the Maintenance Window in hours.
     */
    readonly duration: pulumi.Output<number>;
    /**
     * Whether the maintenance window is enabled. Default: `true`.
     */
    readonly enabled: pulumi.Output<boolean | undefined>;
    /**
     * Timestamp in [ISO-8601 extended format](https://www.iso.org/iso-8601-date-and-time-format.html) when to no longer run the maintenance window.
     */
    readonly endDate: pulumi.Output<string | undefined>;
    /**
     * The name of the maintenance window.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The schedule of the Maintenance Window in the form of a [cron or rate expression](https://docs.aws.amazon.com/systems-manager/latest/userguide/reference-cron-and-rate-expressions.html).
     */
    readonly schedule: pulumi.Output<string>;
    /**
     * The number of days to wait after the date and time specified by a CRON expression before running the maintenance window. Valid range is `1` to `6`.
     */
    readonly scheduleOffset: pulumi.Output<number | undefined>;
    /**
     * Timezone for schedule in [Internet Assigned Numbers Authority (IANA) Time Zone Database format](https://www.iana.org/time-zones). For example: `America/Los_Angeles`, `etc/UTC`, or `Asia/Seoul`.
     */
    readonly scheduleTimezone: pulumi.Output<string | undefined>;
    /**
     * Timestamp in [ISO-8601 extended format](https://www.iso.org/iso-8601-date-and-time-format.html) when to begin the maintenance window.
     */
    readonly startDate: pulumi.Output<string | undefined>;
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
     * Create a MaintenanceWindow resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: MaintenanceWindowArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering MaintenanceWindow resources.
 */
export interface MaintenanceWindowState {
    /**
     * Whether targets must be registered with the Maintenance Window before tasks can be defined for those targets.
     */
    allowUnassociatedTargets?: pulumi.Input<boolean>;
    /**
     * The number of hours before the end of the Maintenance Window that Systems Manager stops scheduling new tasks for execution.
     */
    cutoff?: pulumi.Input<number>;
    /**
     * A description for the maintenance window.
     */
    description?: pulumi.Input<string>;
    /**
     * The duration of the Maintenance Window in hours.
     */
    duration?: pulumi.Input<number>;
    /**
     * Whether the maintenance window is enabled. Default: `true`.
     */
    enabled?: pulumi.Input<boolean>;
    /**
     * Timestamp in [ISO-8601 extended format](https://www.iso.org/iso-8601-date-and-time-format.html) when to no longer run the maintenance window.
     */
    endDate?: pulumi.Input<string>;
    /**
     * The name of the maintenance window.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The schedule of the Maintenance Window in the form of a [cron or rate expression](https://docs.aws.amazon.com/systems-manager/latest/userguide/reference-cron-and-rate-expressions.html).
     */
    schedule?: pulumi.Input<string>;
    /**
     * The number of days to wait after the date and time specified by a CRON expression before running the maintenance window. Valid range is `1` to `6`.
     */
    scheduleOffset?: pulumi.Input<number>;
    /**
     * Timezone for schedule in [Internet Assigned Numbers Authority (IANA) Time Zone Database format](https://www.iana.org/time-zones). For example: `America/Los_Angeles`, `etc/UTC`, or `Asia/Seoul`.
     */
    scheduleTimezone?: pulumi.Input<string>;
    /**
     * Timestamp in [ISO-8601 extended format](https://www.iso.org/iso-8601-date-and-time-format.html) when to begin the maintenance window.
     */
    startDate?: pulumi.Input<string>;
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
}
/**
 * The set of arguments for constructing a MaintenanceWindow resource.
 */
export interface MaintenanceWindowArgs {
    /**
     * Whether targets must be registered with the Maintenance Window before tasks can be defined for those targets.
     */
    allowUnassociatedTargets?: pulumi.Input<boolean>;
    /**
     * The number of hours before the end of the Maintenance Window that Systems Manager stops scheduling new tasks for execution.
     */
    cutoff: pulumi.Input<number>;
    /**
     * A description for the maintenance window.
     */
    description?: pulumi.Input<string>;
    /**
     * The duration of the Maintenance Window in hours.
     */
    duration: pulumi.Input<number>;
    /**
     * Whether the maintenance window is enabled. Default: `true`.
     */
    enabled?: pulumi.Input<boolean>;
    /**
     * Timestamp in [ISO-8601 extended format](https://www.iso.org/iso-8601-date-and-time-format.html) when to no longer run the maintenance window.
     */
    endDate?: pulumi.Input<string>;
    /**
     * The name of the maintenance window.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The schedule of the Maintenance Window in the form of a [cron or rate expression](https://docs.aws.amazon.com/systems-manager/latest/userguide/reference-cron-and-rate-expressions.html).
     */
    schedule: pulumi.Input<string>;
    /**
     * The number of days to wait after the date and time specified by a CRON expression before running the maintenance window. Valid range is `1` to `6`.
     */
    scheduleOffset?: pulumi.Input<number>;
    /**
     * Timezone for schedule in [Internet Assigned Numbers Authority (IANA) Time Zone Database format](https://www.iana.org/time-zones). For example: `America/Los_Angeles`, `etc/UTC`, or `Asia/Seoul`.
     */
    scheduleTimezone?: pulumi.Input<string>;
    /**
     * Timestamp in [ISO-8601 extended format](https://www.iso.org/iso-8601-date-and-time-format.html) when to begin the maintenance window.
     */
    startDate?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
