import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a CloudTrail resource.
 *
 * > **Tip:** For a multi-region trail, this resource must be in the home region of the trail.
 *
 * > **Tip:** For an organization trail, this resource must be in the master account of the organization.
 *
 * ## Example Usage
 *
 * ### Basic
 *
 * Enable CloudTrail to capture all compatible management events in region.
 * For capturing events from services like IAM, `includeGlobalServiceEvents` must be enabled.
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const exampleBucket = new aws.s3.Bucket("example", {
 *     bucket: "my-test-trail",
 *     forceDestroy: true,
 * });
 * const current = aws.getCallerIdentity({});
 * const currentGetPartition = aws.getPartition({});
 * const currentGetRegion = aws.getRegion({});
 * const example = aws.iam.getPolicyDocumentOutput({
 *     statements: [
 *         {
 *             sid: "AWSCloudTrailAclCheck",
 *             effect: "Allow",
 *             principals: [{
 *                 type: "Service",
 *                 identifiers: ["cloudtrail.amazonaws.com"],
 *             }],
 *             actions: ["s3:GetBucketAcl"],
 *             resources: [exampleBucket.arn],
 *             conditions: [{
 *                 test: "StringEquals",
 *                 variable: "aws:SourceArn",
 *                 values: [Promise.all([currentGetPartition, currentGetRegion, current]).then(([currentGetPartition, currentGetRegion, current]) => `arn:${currentGetPartition.partition}:cloudtrail:${currentGetRegion.region}:${current.accountId}:trail/example`)],
 *             }],
 *         },
 *         {
 *             sid: "AWSCloudTrailWrite",
 *             effect: "Allow",
 *             principals: [{
 *                 type: "Service",
 *                 identifiers: ["cloudtrail.amazonaws.com"],
 *             }],
 *             actions: ["s3:PutObject"],
 *             resources: [Promise.all([exampleBucket.arn, current]).then(([arn, current]) => `${arn}/prefix/AWSLogs/${current.accountId}/*`)],
 *             conditions: [
 *                 {
 *                     test: "StringEquals",
 *                     variable: "s3:x-amz-acl",
 *                     values: ["bucket-owner-full-control"],
 *                 },
 *                 {
 *                     test: "StringEquals",
 *                     variable: "aws:SourceArn",
 *                     values: [Promise.all([currentGetPartition, currentGetRegion, current]).then(([currentGetPartition, currentGetRegion, current]) => `arn:${currentGetPartition.partition}:cloudtrail:${currentGetRegion.region}:${current.accountId}:trail/example`)],
 *                 },
 *             ],
 *         },
 *     ],
 * });
 * const exampleBucketPolicy = new aws.s3.BucketPolicy("example", {
 *     bucket: exampleBucket.id,
 *     policy: example.apply(example => example.json),
 * });
 * const exampleTrail = new aws.cloudtrail.Trail("example", {
 *     name: "example",
 *     s3BucketName: exampleBucket.id,
 *     s3KeyPrefix: "prefix",
 *     includeGlobalServiceEvents: false,
 * }, {
 *     dependsOn: [exampleBucketPolicy],
 * });
 * ```
 *
 * ### Data Event Logging
 *
 * CloudTrail can log [Data Events](https://docs.aws.amazon.com/awscloudtrail/latest/userguide/logging-data-events-with-cloudtrail.html) for certain services such as S3 objects and Lambda function invocations. Additional information about data event configuration can be found in the following links:
 *
 * * [CloudTrail API DataResource documentation](https://docs.aws.amazon.com/awscloudtrail/latest/APIReference/API_DataResource.html) (for basic event selector).
 * * [CloudTrail API AdvancedFieldSelector documentation](https://docs.aws.amazon.com/awscloudtrail/latest/APIReference/API_AdvancedFieldSelector.html) (for advanced event selector).
 *
 * ### Logging All Lambda Function Invocations By Using Basic Event Selectors
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.cloudtrail.Trail("example", {eventSelectors: [{
 *     readWriteType: "All",
 *     includeManagementEvents: true,
 *     dataResources: [{
 *         type: "AWS::Lambda::Function",
 *         values: ["arn:aws:lambda"],
 *     }],
 * }]});
 * ```
 *
 * ### Logging All S3 Object Events By Using Basic Event Selectors
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.cloudtrail.Trail("example", {eventSelectors: [{
 *     readWriteType: "All",
 *     includeManagementEvents: true,
 *     dataResources: [{
 *         type: "AWS::S3::Object",
 *         values: ["arn:aws:s3"],
 *     }],
 * }]});
 * ```
 *
 * ### Logging Individual S3 Bucket Events By Using Basic Event Selectors
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const important_bucket = aws.s3.getBucket({
 *     bucket: "important-bucket",
 * });
 * const example = new aws.cloudtrail.Trail("example", {eventSelectors: [{
 *     readWriteType: "All",
 *     includeManagementEvents: true,
 *     dataResources: [{
 *         type: "AWS::S3::Object",
 *         values: [important_bucket.then(important_bucket => `${important_bucket.arn}/`)],
 *     }],
 * }]});
 * ```
 *
 * ### Logging All S3 Object Events Except For Two S3 Buckets By Using Advanced Event Selectors
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const not_important_bucket_1 = aws.s3.getBucket({
 *     bucket: "not-important-bucket-1",
 * });
 * const not_important_bucket_2 = aws.s3.getBucket({
 *     bucket: "not-important-bucket-2",
 * });
 * const example = new aws.cloudtrail.Trail("example", {advancedEventSelectors: [
 *     {
 *         name: "Log all S3 objects events except for two S3 buckets",
 *         fieldSelectors: [
 *             {
 *                 field: "eventCategory",
 *                 equals: ["Data"],
 *             },
 *             {
 *                 field: "resources.ARN",
 *                 notStartsWiths: [
 *                     not_important_bucket_1.then(not_important_bucket_1 => `${not_important_bucket_1.arn}/`),
 *                     not_important_bucket_2.then(not_important_bucket_2 => `${not_important_bucket_2.arn}/`),
 *                 ],
 *             },
 *             {
 *                 field: "resources.type",
 *                 equals: ["AWS::S3::Object"],
 *             },
 *         ],
 *     },
 *     {
 *         name: "Log readOnly and writeOnly management events",
 *         fieldSelectors: [{
 *             field: "eventCategory",
 *             equals: ["Management"],
 *         }],
 *     },
 * ]});
 * ```
 *
 * ### Logging Individual S3 Buckets And Specific Event Names By Using Advanced Event Selectors
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const important_bucket_1 = aws.s3.getBucket({
 *     bucket: "important-bucket-1",
 * });
 * const important_bucket_2 = aws.s3.getBucket({
 *     bucket: "important-bucket-2",
 * });
 * const important_bucket_3 = aws.s3.getBucket({
 *     bucket: "important-bucket-3",
 * });
 * const example = new aws.cloudtrail.Trail("example", {advancedEventSelectors: [
 *     {
 *         name: "Log PutObject and DeleteObject events for two S3 buckets",
 *         fieldSelectors: [
 *             {
 *                 field: "eventCategory",
 *                 equals: ["Data"],
 *             },
 *             {
 *                 field: "eventName",
 *                 equals: [
 *                     "PutObject",
 *                     "DeleteObject",
 *                 ],
 *             },
 *             {
 *                 field: "resources.ARN",
 *                 startsWiths: [
 *                     important_bucket_1.then(important_bucket_1 => `${important_bucket_1.arn}/`),
 *                     important_bucket_2.then(important_bucket_2 => `${important_bucket_2.arn}/`),
 *                 ],
 *             },
 *             {
 *                 field: "readOnly",
 *                 equals: ["false"],
 *             },
 *             {
 *                 field: "resources.type",
 *                 equals: ["AWS::S3::Object"],
 *             },
 *         ],
 *     },
 *     {
 *         name: "Log Delete* events for one S3 bucket",
 *         fieldSelectors: [
 *             {
 *                 field: "eventCategory",
 *                 equals: ["Data"],
 *             },
 *             {
 *                 field: "eventName",
 *                 startsWiths: ["Delete"],
 *             },
 *             {
 *                 field: "resources.ARN",
 *                 equals: [important_bucket_3.then(important_bucket_3 => `${important_bucket_3.arn}/important-prefix`)],
 *             },
 *             {
 *                 field: "readOnly",
 *                 equals: ["false"],
 *             },
 *             {
 *                 field: "resources.type",
 *                 equals: ["AWS::S3::Object"],
 *             },
 *         ],
 *     },
 * ]});
 * ```
 *
 * ### Sending Events to CloudWatch Logs
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.cloudwatch.LogGroup("example", {name: "Example"});
 * const exampleTrail = new aws.cloudtrail.Trail("example", {cloudWatchLogsGroupArn: pulumi.interpolate`${example.arn}:*`});
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Cloudtrails using the `arn`. For example:
 *
 * ```sh
 * $ pulumi import aws:cloudtrail/trail:Trail sample arn:aws:cloudtrail:us-east-1:123456789012:trail/my-sample-trail
 * ```
 */
export declare class Trail extends pulumi.CustomResource {
    /**
     * Get an existing Trail resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: TrailState, opts?: pulumi.CustomResourceOptions): Trail;
    /**
     * Returns true if the given object is an instance of Trail.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Trail;
    /**
     * Specifies an advanced event selector for enabling data event logging. Fields documented below. Conflicts with `eventSelector`.
     */
    readonly advancedEventSelectors: pulumi.Output<outputs.cloudtrail.TrailAdvancedEventSelector[] | undefined>;
    /**
     * ARN of the trail.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Log group name using an ARN that represents the log group to which CloudTrail logs will be delivered. Note that CloudTrail requires the Log Stream wildcard.
     */
    readonly cloudWatchLogsGroupArn: pulumi.Output<string | undefined>;
    /**
     * Role for the CloudWatch Logs endpoint to assume to write to a user’s log group.
     */
    readonly cloudWatchLogsRoleArn: pulumi.Output<string | undefined>;
    /**
     * Whether log file integrity validation is enabled. Defaults to `false`.
     */
    readonly enableLogFileValidation: pulumi.Output<boolean | undefined>;
    /**
     * Enables logging for the trail. When set to `true`, logging is started by calling the [`StartLogging`](https://docs.aws.amazon.com/awscloudtrail/latest/APIReference/API_StartLogging.html) API. When set to `false`, logging is stopped by calling the [`StopLogging`](https://docs.aws.amazon.com/awscloudtrail/latest/APIReference/API_StopLogging.html) API. Defaults to `true`.
     */
    readonly enableLogging: pulumi.Output<boolean | undefined>;
    /**
     * Specifies an event selector for enabling data event logging. Fields documented below. Please note the [CloudTrail limits](https://docs.aws.amazon.com/awscloudtrail/latest/userguide/WhatIsCloudTrail-Limits.html) when configuring these. Conflicts with `advancedEventSelector`.
     */
    readonly eventSelectors: pulumi.Output<outputs.cloudtrail.TrailEventSelector[] | undefined>;
    /**
     * Region in which the trail was created.
     */
    readonly homeRegion: pulumi.Output<string>;
    /**
     * Whether the trail is publishing events from global services such as IAM to the log files. Defaults to `true`.
     */
    readonly includeGlobalServiceEvents: pulumi.Output<boolean | undefined>;
    /**
     * Configuration block for identifying unusual operational activity. See details below.
     */
    readonly insightSelectors: pulumi.Output<outputs.cloudtrail.TrailInsightSelector[] | undefined>;
    /**
     * Whether the trail is created in the current region or in all regions. Defaults to `false`.
     */
    readonly isMultiRegionTrail: pulumi.Output<boolean | undefined>;
    /**
     * Whether the trail is an AWS Organizations trail. Organization trails log events for the master account and all member accounts. Can only be created in the organization master account. Defaults to `false`.
     */
    readonly isOrganizationTrail: pulumi.Output<boolean | undefined>;
    /**
     * KMS key ARN to use to encrypt the logs delivered by CloudTrail.
     */
    readonly kmsKeyId: pulumi.Output<string | undefined>;
    /**
     * Name of the trail.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Name of the S3 bucket designated for publishing log files.
     *
     * The following arguments are optional:
     */
    readonly s3BucketName: pulumi.Output<string>;
    /**
     * S3 key prefix that follows the name of the bucket you have designated for log file delivery.
     */
    readonly s3KeyPrefix: pulumi.Output<string | undefined>;
    /**
     * ARN of the Amazon SNS topic that CloudTrail uses to send notifications when log files are delivered.
     */
    readonly snsTopicArn: pulumi.Output<string>;
    /**
     * Name of the Amazon SNS topic defined for notification of log file delivery. Specify the SNS topic ARN if it resides in another region.
     */
    readonly snsTopicName: pulumi.Output<string | undefined>;
    /**
     * Map of tags to assign to the trail. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Create a Trail resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: TrailArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Trail resources.
 */
export interface TrailState {
    /**
     * Specifies an advanced event selector for enabling data event logging. Fields documented below. Conflicts with `eventSelector`.
     */
    advancedEventSelectors?: pulumi.Input<pulumi.Input<inputs.cloudtrail.TrailAdvancedEventSelector>[]>;
    /**
     * ARN of the trail.
     */
    arn?: pulumi.Input<string>;
    /**
     * Log group name using an ARN that represents the log group to which CloudTrail logs will be delivered. Note that CloudTrail requires the Log Stream wildcard.
     */
    cloudWatchLogsGroupArn?: pulumi.Input<string>;
    /**
     * Role for the CloudWatch Logs endpoint to assume to write to a user’s log group.
     */
    cloudWatchLogsRoleArn?: pulumi.Input<string>;
    /**
     * Whether log file integrity validation is enabled. Defaults to `false`.
     */
    enableLogFileValidation?: pulumi.Input<boolean>;
    /**
     * Enables logging for the trail. When set to `true`, logging is started by calling the [`StartLogging`](https://docs.aws.amazon.com/awscloudtrail/latest/APIReference/API_StartLogging.html) API. When set to `false`, logging is stopped by calling the [`StopLogging`](https://docs.aws.amazon.com/awscloudtrail/latest/APIReference/API_StopLogging.html) API. Defaults to `true`.
     */
    enableLogging?: pulumi.Input<boolean>;
    /**
     * Specifies an event selector for enabling data event logging. Fields documented below. Please note the [CloudTrail limits](https://docs.aws.amazon.com/awscloudtrail/latest/userguide/WhatIsCloudTrail-Limits.html) when configuring these. Conflicts with `advancedEventSelector`.
     */
    eventSelectors?: pulumi.Input<pulumi.Input<inputs.cloudtrail.TrailEventSelector>[]>;
    /**
     * Region in which the trail was created.
     */
    homeRegion?: pulumi.Input<string>;
    /**
     * Whether the trail is publishing events from global services such as IAM to the log files. Defaults to `true`.
     */
    includeGlobalServiceEvents?: pulumi.Input<boolean>;
    /**
     * Configuration block for identifying unusual operational activity. See details below.
     */
    insightSelectors?: pulumi.Input<pulumi.Input<inputs.cloudtrail.TrailInsightSelector>[]>;
    /**
     * Whether the trail is created in the current region or in all regions. Defaults to `false`.
     */
    isMultiRegionTrail?: pulumi.Input<boolean>;
    /**
     * Whether the trail is an AWS Organizations trail. Organization trails log events for the master account and all member accounts. Can only be created in the organization master account. Defaults to `false`.
     */
    isOrganizationTrail?: pulumi.Input<boolean>;
    /**
     * KMS key ARN to use to encrypt the logs delivered by CloudTrail.
     */
    kmsKeyId?: pulumi.Input<string>;
    /**
     * Name of the trail.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Name of the S3 bucket designated for publishing log files.
     *
     * The following arguments are optional:
     */
    s3BucketName?: pulumi.Input<string>;
    /**
     * S3 key prefix that follows the name of the bucket you have designated for log file delivery.
     */
    s3KeyPrefix?: pulumi.Input<string>;
    /**
     * ARN of the Amazon SNS topic that CloudTrail uses to send notifications when log files are delivered.
     */
    snsTopicArn?: pulumi.Input<string>;
    /**
     * Name of the Amazon SNS topic defined for notification of log file delivery. Specify the SNS topic ARN if it resides in another region.
     */
    snsTopicName?: pulumi.Input<string>;
    /**
     * Map of tags to assign to the trail. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
 * The set of arguments for constructing a Trail resource.
 */
export interface TrailArgs {
    /**
     * Specifies an advanced event selector for enabling data event logging. Fields documented below. Conflicts with `eventSelector`.
     */
    advancedEventSelectors?: pulumi.Input<pulumi.Input<inputs.cloudtrail.TrailAdvancedEventSelector>[]>;
    /**
     * Log group name using an ARN that represents the log group to which CloudTrail logs will be delivered. Note that CloudTrail requires the Log Stream wildcard.
     */
    cloudWatchLogsGroupArn?: pulumi.Input<string>;
    /**
     * Role for the CloudWatch Logs endpoint to assume to write to a user’s log group.
     */
    cloudWatchLogsRoleArn?: pulumi.Input<string>;
    /**
     * Whether log file integrity validation is enabled. Defaults to `false`.
     */
    enableLogFileValidation?: pulumi.Input<boolean>;
    /**
     * Enables logging for the trail. When set to `true`, logging is started by calling the [`StartLogging`](https://docs.aws.amazon.com/awscloudtrail/latest/APIReference/API_StartLogging.html) API. When set to `false`, logging is stopped by calling the [`StopLogging`](https://docs.aws.amazon.com/awscloudtrail/latest/APIReference/API_StopLogging.html) API. Defaults to `true`.
     */
    enableLogging?: pulumi.Input<boolean>;
    /**
     * Specifies an event selector for enabling data event logging. Fields documented below. Please note the [CloudTrail limits](https://docs.aws.amazon.com/awscloudtrail/latest/userguide/WhatIsCloudTrail-Limits.html) when configuring these. Conflicts with `advancedEventSelector`.
     */
    eventSelectors?: pulumi.Input<pulumi.Input<inputs.cloudtrail.TrailEventSelector>[]>;
    /**
     * Whether the trail is publishing events from global services such as IAM to the log files. Defaults to `true`.
     */
    includeGlobalServiceEvents?: pulumi.Input<boolean>;
    /**
     * Configuration block for identifying unusual operational activity. See details below.
     */
    insightSelectors?: pulumi.Input<pulumi.Input<inputs.cloudtrail.TrailInsightSelector>[]>;
    /**
     * Whether the trail is created in the current region or in all regions. Defaults to `false`.
     */
    isMultiRegionTrail?: pulumi.Input<boolean>;
    /**
     * Whether the trail is an AWS Organizations trail. Organization trails log events for the master account and all member accounts. Can only be created in the organization master account. Defaults to `false`.
     */
    isOrganizationTrail?: pulumi.Input<boolean>;
    /**
     * KMS key ARN to use to encrypt the logs delivered by CloudTrail.
     */
    kmsKeyId?: pulumi.Input<string>;
    /**
     * Name of the trail.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Name of the S3 bucket designated for publishing log files.
     *
     * The following arguments are optional:
     */
    s3BucketName: pulumi.Input<string>;
    /**
     * S3 key prefix that follows the name of the bucket you have designated for log file delivery.
     */
    s3KeyPrefix?: pulumi.Input<string>;
    /**
     * Name of the Amazon SNS topic defined for notification of log file delivery. Specify the SNS topic ARN if it resides in another region.
     */
    snsTopicName?: pulumi.Input<string>;
    /**
     * Map of tags to assign to the trail. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
