import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.ssm.ContactsRotation("example", {
 *     contactIds: [exampleAwsSsmcontactsContact.arn],
 *     name: "rotation",
 *     recurrence: {
 *         numberOfOnCalls: 1,
 *         recurrenceMultiplier: 1,
 *         dailySettings: [{
 *             hourOfDay: 9,
 *             minuteOfHour: 0,
 *         }],
 *     },
 *     timeZoneId: "Australia/Sydney",
 * }, {
 *     dependsOn: [exampleAwsSsmincidentsReplicationSet],
 * });
 * ```
 *
 * ### Usage with Weekly Settings and Shift Coverages Fields
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.ssm.ContactsRotation("example", {
 *     contactIds: [exampleAwsSsmcontactsContact.arn],
 *     name: "rotation",
 *     recurrence: {
 *         numberOfOnCalls: 1,
 *         recurrenceMultiplier: 1,
 *         weeklySettings: [
 *             {
 *                 dayOfWeek: "WED",
 *                 handOffTime: {
 *                     hourOfDay: 4,
 *                     minuteOfHour: 25,
 *                 },
 *             },
 *             {
 *                 dayOfWeek: "FRI",
 *                 handOffTime: {
 *                     hourOfDay: 15,
 *                     minuteOfHour: 57,
 *                 },
 *             },
 *         ],
 *         shiftCoverages: [{
 *             mapBlockKey: "MON",
 *             coverageTimes: [{
 *                 start: {
 *                     hourOfDay: 1,
 *                     minuteOfHour: 0,
 *                 },
 *                 end: {
 *                     hourOfDay: 23,
 *                     minuteOfHour: 0,
 *                 },
 *             }],
 *         }],
 *     },
 *     startTime: "2023-07-20T02:21:49+00:00",
 *     timeZoneId: "Australia/Sydney",
 *     tags: {
 *         key1: "tag1",
 *         key2: "tag2",
 *     },
 * }, {
 *     dependsOn: [exampleAwsSsmincidentsReplicationSet],
 * });
 * ```
 *
 * ### Usage with Monthly Settings Fields
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.ssm.ContactsRotation("example", {
 *     contactIds: [exampleAwsSsmcontactsContact.arn],
 *     name: "rotation",
 *     recurrence: {
 *         numberOfOnCalls: 1,
 *         recurrenceMultiplier: 1,
 *         monthlySettings: [
 *             {
 *                 dayOfMonth: 20,
 *                 handOffTime: {
 *                     hourOfDay: 8,
 *                     minuteOfHour: 0,
 *                 },
 *             },
 *             {
 *                 dayOfMonth: 13,
 *                 handOffTime: {
 *                     hourOfDay: 12,
 *                     minuteOfHour: 34,
 *                 },
 *             },
 *         ],
 *     },
 *     timeZoneId: "Australia/Sydney",
 * }, {
 *     dependsOn: [exampleAwsSsmincidentsReplicationSet],
 * });
 * ```
 *
 * ## Import
 *
 * ### Identity Schema
 *
 * #### Required
 *
 * - `arn` (String) Amazon Resource Name (ARN) of the SSM Contacts rotation.
 *
 * Using `pulumi import`, import CodeGuru Profiler Profiling Group using the `arn`. For example:
 *
 * console
 *
 * % pulumi import aws_ssmcontacts_rotation.example arn:aws:ssm-contacts:us-east-1:012345678910:rotation/example
 */
export declare class ContactsRotation extends pulumi.CustomResource {
    /**
     * Get an existing ContactsRotation resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ContactsRotationState, opts?: pulumi.CustomResourceOptions): ContactsRotation;
    /**
     * Returns true if the given object is an instance of ContactsRotation.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ContactsRotation;
    /**
     * The Amazon Resource Name (ARN) of the rotation.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Amazon Resource Names (ARNs) of the contacts to add to the rotation. The order in which you list the contacts is their shift order in the rotation schedule.
     */
    readonly contactIds: pulumi.Output<string[]>;
    /**
     * The name for the rotation.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Information about when an on-call rotation is in effect and how long the rotation period lasts. Exactly one of either `dailySettings`, `monthlySettings`, or `weeklySettings` must be populated. See Recurrence for more details.
     *
     * The following arguments are optional:
     */
    readonly recurrence: pulumi.Output<outputs.ssm.ContactsRotationRecurrence | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The date and time, in RFC 3339 format, that the rotation goes into effect.
     */
    readonly startTime: pulumi.Output<string | undefined>;
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
     * The time zone to base the rotation’s activity on in Internet Assigned Numbers Authority (IANA) format.
     */
    readonly timeZoneId: pulumi.Output<string>;
    /**
     * Create a ContactsRotation resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ContactsRotationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ContactsRotation resources.
 */
export interface ContactsRotationState {
    /**
     * The Amazon Resource Name (ARN) of the rotation.
     */
    arn?: pulumi.Input<string>;
    /**
     * Amazon Resource Names (ARNs) of the contacts to add to the rotation. The order in which you list the contacts is their shift order in the rotation schedule.
     */
    contactIds?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The name for the rotation.
     */
    name?: pulumi.Input<string>;
    /**
     * Information about when an on-call rotation is in effect and how long the rotation period lasts. Exactly one of either `dailySettings`, `monthlySettings`, or `weeklySettings` must be populated. See Recurrence for more details.
     *
     * The following arguments are optional:
     */
    recurrence?: pulumi.Input<inputs.ssm.ContactsRotationRecurrence>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The date and time, in RFC 3339 format, that the rotation goes into effect.
     */
    startTime?: pulumi.Input<string>;
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
    /**
     * The time zone to base the rotation’s activity on in Internet Assigned Numbers Authority (IANA) format.
     */
    timeZoneId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a ContactsRotation resource.
 */
export interface ContactsRotationArgs {
    /**
     * Amazon Resource Names (ARNs) of the contacts to add to the rotation. The order in which you list the contacts is their shift order in the rotation schedule.
     */
    contactIds: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The name for the rotation.
     */
    name?: pulumi.Input<string>;
    /**
     * Information about when an on-call rotation is in effect and how long the rotation period lasts. Exactly one of either `dailySettings`, `monthlySettings`, or `weeklySettings` must be populated. See Recurrence for more details.
     *
     * The following arguments are optional:
     */
    recurrence?: pulumi.Input<inputs.ssm.ContactsRotationRecurrence>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The date and time, in RFC 3339 format, that the rotation goes into effect.
     */
    startTime?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The time zone to base the rotation’s activity on in Internet Assigned Numbers Authority (IANA) format.
     */
    timeZoneId: pulumi.Input<string>;
}
