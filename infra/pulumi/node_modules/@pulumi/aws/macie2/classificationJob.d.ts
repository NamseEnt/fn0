import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a resource to manage an [AWS Macie Classification Job](https://docs.aws.amazon.com/macie/latest/APIReference/jobs.html).
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = new aws.macie2.Account("test", {});
 * const testClassificationJob = new aws.macie2.ClassificationJob("test", {
 *     jobType: "ONE_TIME",
 *     name: "NAME OF THE CLASSIFICATION JOB",
 *     s3JobDefinition: {
 *         bucketDefinitions: [{
 *             accountId: "ACCOUNT ID",
 *             buckets: ["S3 BUCKET NAME"],
 *         }],
 *     },
 * }, {
 *     dependsOn: [test],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_macie2_classification_job` using the id. For example:
 *
 * ```sh
 * $ pulumi import aws:macie2/classificationJob:ClassificationJob example abcd1
 * ```
 */
export declare class ClassificationJob extends pulumi.CustomResource {
    /**
     * Get an existing ClassificationJob resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ClassificationJobState, opts?: pulumi.CustomResourceOptions): ClassificationJob;
    /**
     * Returns true if the given object is an instance of ClassificationJob.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ClassificationJob;
    /**
     * The date and time, in UTC and extended RFC 3339 format, when the job was created.
     */
    readonly createdAt: pulumi.Output<string>;
    /**
     * The custom data identifiers to use for data analysis and classification.
     */
    readonly customDataIdentifierIds: pulumi.Output<string[]>;
    /**
     * A custom description of the job. The description can contain as many as 200 characters.
     */
    readonly description: pulumi.Output<string>;
    /**
     * Specifies whether to analyze all existing, eligible objects immediately after the job is created.
     */
    readonly initialRun: pulumi.Output<boolean | undefined>;
    readonly jobArn: pulumi.Output<string>;
    readonly jobId: pulumi.Output<string>;
    /**
     * The status for the job. Valid values are: `CANCELLED`, `RUNNING` and `USER_PAUSED`
     */
    readonly jobStatus: pulumi.Output<string>;
    /**
     * The schedule for running the job. Valid values are: `ONE_TIME` - Run the job only once. If you specify this value, don't specify a value for the `scheduleFrequency` property. `SCHEDULED` - Run the job on a daily, weekly, or monthly basis. If you specify this value, use the `scheduleFrequency` property to define the recurrence pattern for the job.
     */
    readonly jobType: pulumi.Output<string>;
    /**
     * A custom name for the job. The name can contain as many as 500 characters. If omitted, the provider will assign a random, unique name. Conflicts with `namePrefix`.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Creates a unique name beginning with the specified prefix. Conflicts with `name`.
     */
    readonly namePrefix: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The S3 buckets that contain the objects to analyze, and the scope of that analysis. (documented below)
     */
    readonly s3JobDefinition: pulumi.Output<outputs.macie2.ClassificationJobS3JobDefinition>;
    /**
     * The sampling depth, as a percentage, to apply when processing objects. This value determines the percentage of eligible objects that the job analyzes. If this value is less than 100, Amazon Macie selects the objects to analyze at random, up to the specified percentage, and analyzes all the data in those objects.
     */
    readonly samplingPercentage: pulumi.Output<number>;
    /**
     * The recurrence pattern for running the job. To run the job only once, don't specify a value for this property and set the value for the `jobType` property to `ONE_TIME`. (documented below)
     */
    readonly scheduleFrequency: pulumi.Output<outputs.macie2.ClassificationJobScheduleFrequency>;
    /**
     * Map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * If the current status of the job is `USER_PAUSED`, specifies when the job was paused and when the job or job run will expire and be canceled if it isn't resumed. This value is present only if the value for `job-status` is `USER_PAUSED`.
     */
    readonly userPausedDetails: pulumi.Output<outputs.macie2.ClassificationJobUserPausedDetail[]>;
    /**
     * Create a ClassificationJob resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ClassificationJobArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ClassificationJob resources.
 */
export interface ClassificationJobState {
    /**
     * The date and time, in UTC and extended RFC 3339 format, when the job was created.
     */
    createdAt?: pulumi.Input<string>;
    /**
     * The custom data identifiers to use for data analysis and classification.
     */
    customDataIdentifierIds?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * A custom description of the job. The description can contain as many as 200 characters.
     */
    description?: pulumi.Input<string>;
    /**
     * Specifies whether to analyze all existing, eligible objects immediately after the job is created.
     */
    initialRun?: pulumi.Input<boolean>;
    jobArn?: pulumi.Input<string>;
    jobId?: pulumi.Input<string>;
    /**
     * The status for the job. Valid values are: `CANCELLED`, `RUNNING` and `USER_PAUSED`
     */
    jobStatus?: pulumi.Input<string>;
    /**
     * The schedule for running the job. Valid values are: `ONE_TIME` - Run the job only once. If you specify this value, don't specify a value for the `scheduleFrequency` property. `SCHEDULED` - Run the job on a daily, weekly, or monthly basis. If you specify this value, use the `scheduleFrequency` property to define the recurrence pattern for the job.
     */
    jobType?: pulumi.Input<string>;
    /**
     * A custom name for the job. The name can contain as many as 500 characters. If omitted, the provider will assign a random, unique name. Conflicts with `namePrefix`.
     */
    name?: pulumi.Input<string>;
    /**
     * Creates a unique name beginning with the specified prefix. Conflicts with `name`.
     */
    namePrefix?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The S3 buckets that contain the objects to analyze, and the scope of that analysis. (documented below)
     */
    s3JobDefinition?: pulumi.Input<inputs.macie2.ClassificationJobS3JobDefinition>;
    /**
     * The sampling depth, as a percentage, to apply when processing objects. This value determines the percentage of eligible objects that the job analyzes. If this value is less than 100, Amazon Macie selects the objects to analyze at random, up to the specified percentage, and analyzes all the data in those objects.
     */
    samplingPercentage?: pulumi.Input<number>;
    /**
     * The recurrence pattern for running the job. To run the job only once, don't specify a value for this property and set the value for the `jobType` property to `ONE_TIME`. (documented below)
     */
    scheduleFrequency?: pulumi.Input<inputs.macie2.ClassificationJobScheduleFrequency>;
    /**
     * Map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * If the current status of the job is `USER_PAUSED`, specifies when the job was paused and when the job or job run will expire and be canceled if it isn't resumed. This value is present only if the value for `job-status` is `USER_PAUSED`.
     */
    userPausedDetails?: pulumi.Input<pulumi.Input<inputs.macie2.ClassificationJobUserPausedDetail>[]>;
}
/**
 * The set of arguments for constructing a ClassificationJob resource.
 */
export interface ClassificationJobArgs {
    /**
     * The custom data identifiers to use for data analysis and classification.
     */
    customDataIdentifierIds?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * A custom description of the job. The description can contain as many as 200 characters.
     */
    description?: pulumi.Input<string>;
    /**
     * Specifies whether to analyze all existing, eligible objects immediately after the job is created.
     */
    initialRun?: pulumi.Input<boolean>;
    /**
     * The status for the job. Valid values are: `CANCELLED`, `RUNNING` and `USER_PAUSED`
     */
    jobStatus?: pulumi.Input<string>;
    /**
     * The schedule for running the job. Valid values are: `ONE_TIME` - Run the job only once. If you specify this value, don't specify a value for the `scheduleFrequency` property. `SCHEDULED` - Run the job on a daily, weekly, or monthly basis. If you specify this value, use the `scheduleFrequency` property to define the recurrence pattern for the job.
     */
    jobType: pulumi.Input<string>;
    /**
     * A custom name for the job. The name can contain as many as 500 characters. If omitted, the provider will assign a random, unique name. Conflicts with `namePrefix`.
     */
    name?: pulumi.Input<string>;
    /**
     * Creates a unique name beginning with the specified prefix. Conflicts with `name`.
     */
    namePrefix?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The S3 buckets that contain the objects to analyze, and the scope of that analysis. (documented below)
     */
    s3JobDefinition: pulumi.Input<inputs.macie2.ClassificationJobS3JobDefinition>;
    /**
     * The sampling depth, as a percentage, to apply when processing objects. This value determines the percentage of eligible objects that the job analyzes. If this value is less than 100, Amazon Macie selects the objects to analyze at random, up to the specified percentage, and analyzes all the data in those objects.
     */
    samplingPercentage?: pulumi.Input<number>;
    /**
     * The recurrence pattern for running the job. To run the job only once, don't specify a value for this property and set the value for the `jobType` property to `ONE_TIME`. (documented below)
     */
    scheduleFrequency?: pulumi.Input<inputs.macie2.ClassificationJobScheduleFrequency>;
    /**
     * Map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
