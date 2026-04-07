import * as pulumi from "@pulumi/pulumi";
/**
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const _default = new aws.redshift.SnapshotSchedule("default", {
 *     identifier: "tf-redshift-snapshot-schedule",
 *     definitions: ["rate(12 hours)"],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Redshift Snapshot Schedule using the `identifier`. For example:
 *
 * ```sh
 * $ pulumi import aws:redshift/snapshotSchedule:SnapshotSchedule default tf-redshift-snapshot-schedule
 * ```
 */
export declare class SnapshotSchedule extends pulumi.CustomResource {
    /**
     * Get an existing SnapshotSchedule resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: SnapshotScheduleState, opts?: pulumi.CustomResourceOptions): SnapshotSchedule;
    /**
     * Returns true if the given object is an instance of SnapshotSchedule.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is SnapshotSchedule;
    /**
     * Amazon Resource Name (ARN) of the Redshift Snapshot Schedule.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The definition of the snapshot schedule. The definition is made up of schedule expressions, for example `cron(30 12 *)` or `rate(12 hours)`.
     */
    readonly definitions: pulumi.Output<string[]>;
    /**
     * The description of the snapshot schedule.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * Whether to destroy all associated clusters with this snapshot schedule on deletion. Must be enabled and applied before attempting deletion.
     */
    readonly forceDestroy: pulumi.Output<boolean | undefined>;
    /**
     * The snapshot schedule identifier. If omitted, this provider will assign a random, unique identifier.
     */
    readonly identifier: pulumi.Output<string>;
    /**
     * Creates a unique
     * identifier beginning with the specified prefix. Conflicts with `identifier`.
     */
    readonly identifierPrefix: pulumi.Output<string>;
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
     * Create a SnapshotSchedule resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: SnapshotScheduleArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering SnapshotSchedule resources.
 */
export interface SnapshotScheduleState {
    /**
     * Amazon Resource Name (ARN) of the Redshift Snapshot Schedule.
     */
    arn?: pulumi.Input<string>;
    /**
     * The definition of the snapshot schedule. The definition is made up of schedule expressions, for example `cron(30 12 *)` or `rate(12 hours)`.
     */
    definitions?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The description of the snapshot schedule.
     */
    description?: pulumi.Input<string>;
    /**
     * Whether to destroy all associated clusters with this snapshot schedule on deletion. Must be enabled and applied before attempting deletion.
     */
    forceDestroy?: pulumi.Input<boolean>;
    /**
     * The snapshot schedule identifier. If omitted, this provider will assign a random, unique identifier.
     */
    identifier?: pulumi.Input<string>;
    /**
     * Creates a unique
     * identifier beginning with the specified prefix. Conflicts with `identifier`.
     */
    identifierPrefix?: pulumi.Input<string>;
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
}
/**
 * The set of arguments for constructing a SnapshotSchedule resource.
 */
export interface SnapshotScheduleArgs {
    /**
     * The definition of the snapshot schedule. The definition is made up of schedule expressions, for example `cron(30 12 *)` or `rate(12 hours)`.
     */
    definitions: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The description of the snapshot schedule.
     */
    description?: pulumi.Input<string>;
    /**
     * Whether to destroy all associated clusters with this snapshot schedule on deletion. Must be enabled and applied before attempting deletion.
     */
    forceDestroy?: pulumi.Input<boolean>;
    /**
     * The snapshot schedule identifier. If omitted, this provider will assign a random, unique identifier.
     */
    identifier?: pulumi.Input<string>;
    /**
     * Creates a unique
     * identifier beginning with the specified prefix. Conflicts with `identifier`.
     */
    identifierPrefix?: pulumi.Input<string>;
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
}
