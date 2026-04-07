import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Generates a CloudWatch Log Group Data Protection Policy document in JSON format for use with the `aws.cloudwatch.LogDataProtectionPolicy` resource.
 *
 * > For more information about data protection policies, see the [Help protect sensitive log data with masking](https://docs.aws.amazon.com/AmazonCloudWatch/latest/logs/mask-sensitive-log-data.html).
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.cloudwatch.getLogDataProtectionPolicyDocument({
 *     name: "Example",
 *     statements: [
 *         {
 *             sid: "Audit",
 *             dataIdentifiers: [
 *                 "arn:aws:dataprotection::aws:data-identifier/EmailAddress",
 *                 "arn:aws:dataprotection::aws:data-identifier/DriversLicense-US",
 *             ],
 *             operation: {
 *                 audit: {
 *                     findingsDestination: {
 *                         cloudwatchLogs: {
 *                             logGroup: audit.name,
 *                         },
 *                         firehose: {
 *                             deliveryStream: auditAwsKinesisFirehoseDeliveryStream.name,
 *                         },
 *                         s3: {
 *                             bucket: auditAwsS3Bucket.bucket,
 *                         },
 *                     },
 *                 },
 *             },
 *         },
 *         {
 *             sid: "Deidentify",
 *             dataIdentifiers: [
 *                 "arn:aws:dataprotection::aws:data-identifier/EmailAddress",
 *                 "arn:aws:dataprotection::aws:data-identifier/DriversLicense-US",
 *             ],
 *             operation: {
 *                 deidentify: {
 *                     maskConfig: {},
 *                 },
 *             },
 *         },
 *     ],
 * });
 * const exampleLogDataProtectionPolicy = new aws.cloudwatch.LogDataProtectionPolicy("example", {
 *     logGroupName: exampleAwsCloudwatchLogGroup.name,
 *     policyDocument: example.then(example => example.json),
 * });
 * ```
 */
export declare function getLogDataProtectionPolicyDocument(args: GetLogDataProtectionPolicyDocumentArgs, opts?: pulumi.InvokeOptions): Promise<GetLogDataProtectionPolicyDocumentResult>;
/**
 * A collection of arguments for invoking getLogDataProtectionPolicyDocument.
 */
export interface GetLogDataProtectionPolicyDocumentArgs {
    configuration?: inputs.cloudwatch.GetLogDataProtectionPolicyDocumentConfiguration;
    description?: string;
    /**
     * The name of the data protection policy document.
     */
    name: string;
    /**
     * Configures the data protection policy.
     *
     * > There must be exactly two statements: the first with an `audit` operation, and the second with a `deidentify` operation.
     *
     * The following arguments are optional:
     */
    statements: inputs.cloudwatch.GetLogDataProtectionPolicyDocumentStatement[];
    version?: string;
}
/**
 * A collection of values returned by getLogDataProtectionPolicyDocument.
 */
export interface GetLogDataProtectionPolicyDocumentResult {
    readonly configuration?: outputs.cloudwatch.GetLogDataProtectionPolicyDocumentConfiguration;
    readonly description?: string;
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    /**
     * Standard JSON policy document rendered based on the arguments above.
     */
    readonly json: string;
    readonly name: string;
    readonly statements: outputs.cloudwatch.GetLogDataProtectionPolicyDocumentStatement[];
    readonly version?: string;
}
/**
 * Generates a CloudWatch Log Group Data Protection Policy document in JSON format for use with the `aws.cloudwatch.LogDataProtectionPolicy` resource.
 *
 * > For more information about data protection policies, see the [Help protect sensitive log data with masking](https://docs.aws.amazon.com/AmazonCloudWatch/latest/logs/mask-sensitive-log-data.html).
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.cloudwatch.getLogDataProtectionPolicyDocument({
 *     name: "Example",
 *     statements: [
 *         {
 *             sid: "Audit",
 *             dataIdentifiers: [
 *                 "arn:aws:dataprotection::aws:data-identifier/EmailAddress",
 *                 "arn:aws:dataprotection::aws:data-identifier/DriversLicense-US",
 *             ],
 *             operation: {
 *                 audit: {
 *                     findingsDestination: {
 *                         cloudwatchLogs: {
 *                             logGroup: audit.name,
 *                         },
 *                         firehose: {
 *                             deliveryStream: auditAwsKinesisFirehoseDeliveryStream.name,
 *                         },
 *                         s3: {
 *                             bucket: auditAwsS3Bucket.bucket,
 *                         },
 *                     },
 *                 },
 *             },
 *         },
 *         {
 *             sid: "Deidentify",
 *             dataIdentifiers: [
 *                 "arn:aws:dataprotection::aws:data-identifier/EmailAddress",
 *                 "arn:aws:dataprotection::aws:data-identifier/DriversLicense-US",
 *             ],
 *             operation: {
 *                 deidentify: {
 *                     maskConfig: {},
 *                 },
 *             },
 *         },
 *     ],
 * });
 * const exampleLogDataProtectionPolicy = new aws.cloudwatch.LogDataProtectionPolicy("example", {
 *     logGroupName: exampleAwsCloudwatchLogGroup.name,
 *     policyDocument: example.then(example => example.json),
 * });
 * ```
 */
export declare function getLogDataProtectionPolicyDocumentOutput(args: GetLogDataProtectionPolicyDocumentOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetLogDataProtectionPolicyDocumentResult>;
/**
 * A collection of arguments for invoking getLogDataProtectionPolicyDocument.
 */
export interface GetLogDataProtectionPolicyDocumentOutputArgs {
    configuration?: pulumi.Input<inputs.cloudwatch.GetLogDataProtectionPolicyDocumentConfigurationArgs>;
    description?: pulumi.Input<string>;
    /**
     * The name of the data protection policy document.
     */
    name: pulumi.Input<string>;
    /**
     * Configures the data protection policy.
     *
     * > There must be exactly two statements: the first with an `audit` operation, and the second with a `deidentify` operation.
     *
     * The following arguments are optional:
     */
    statements: pulumi.Input<pulumi.Input<inputs.cloudwatch.GetLogDataProtectionPolicyDocumentStatementArgs>[]>;
    version?: pulumi.Input<string>;
}
