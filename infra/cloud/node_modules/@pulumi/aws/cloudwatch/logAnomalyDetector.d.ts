import * as pulumi from "@pulumi/pulumi";
/**
 * Resource for managing an AWS CloudWatch Logs Log Anomaly Detector.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test: aws.cloudwatch.LogGroup[] = [];
 * for (const range = {value: 0}; range.value < 2; range.value++) {
 *     test.push(new aws.cloudwatch.LogGroup(`test-${range.value}`, {name: `testing-${range.value}`}));
 * }
 * const testLogAnomalyDetector = new aws.cloudwatch.LogAnomalyDetector("test", {
 *     detectorName: "testing",
 *     logGroupArnLists: [test[0].arn],
 *     anomalyVisibilityTime: 7,
 *     evaluationFrequency: "TEN_MIN",
 *     enabled: false,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import CloudWatch Log Anomaly Detector using the `arn`. For example:
 *
 * ```sh
 * $ pulumi import aws:cloudwatch/logAnomalyDetector:LogAnomalyDetector example log_anomaly_detector-arn-12345678
 * ```
 */
export declare class LogAnomalyDetector extends pulumi.CustomResource {
    /**
     * Get an existing LogAnomalyDetector resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: LogAnomalyDetectorState, opts?: pulumi.CustomResourceOptions): LogAnomalyDetector;
    /**
     * Returns true if the given object is an instance of LogAnomalyDetector.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is LogAnomalyDetector;
    /**
     * Number of days to have visibility on an anomaly. After this time period has elapsed for an anomaly, it will be automatically baselined and the anomaly detector will treat new occurrences of a similar anomaly as normal. Therefore, if you do not correct the cause of an anomaly during the time period specified in `anomalyVisibilityTime`, it will be considered normal going forward and will not be detected as an anomaly. Valid Range: Minimum value of 7. Maximum value of 90.
     */
    readonly anomalyVisibilityTime: pulumi.Output<number>;
    /**
     * ARN of the log anomaly detector that you just created.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Name for this anomaly detector.
     */
    readonly detectorName: pulumi.Output<string | undefined>;
    readonly enabled: pulumi.Output<boolean>;
    /**
     * Specifies how often the anomaly detector is to run and look for anomalies. Set this value according to the frequency that the log group receives new logs. For example, if the log group receives new log events every 10 minutes, then 15 minutes might be a good setting for `evaluationFrequency`. Valid Values: `ONE_MIN | FIVE_MIN | TEN_MIN | FIFTEEN_MIN | THIRTY_MIN | ONE_HOUR`.
     */
    readonly evaluationFrequency: pulumi.Output<string | undefined>;
    /**
     * You can use this parameter to limit the anomaly detection model to examine only log events that match the pattern you specify here. For more information, see [Filter and Pattern Syntax](https://docs.aws.amazon.com/AmazonCloudWatch/latest/logs/FilterAndPatternSyntax.html).
     */
    readonly filterPattern: pulumi.Output<string | undefined>;
    /**
     * Optionally assigns a AWS KMS key to secure this anomaly detector and its findings. If a key is assigned, the anomalies found and the model used by this detector are encrypted at rest with the key. If a key is assigned to an anomaly detector, a user must have permissions for both this key and for the anomaly detector to retrieve information about the anomalies that it finds.
     */
    readonly kmsKeyId: pulumi.Output<string | undefined>;
    /**
     * Array containing the ARN of the log group that this anomaly detector will watch. You can specify only one log group ARN.
     *
     * The following arguments are optional:
     */
    readonly logGroupArnLists: pulumi.Output<string[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * Create a LogAnomalyDetector resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: LogAnomalyDetectorArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering LogAnomalyDetector resources.
 */
export interface LogAnomalyDetectorState {
    /**
     * Number of days to have visibility on an anomaly. After this time period has elapsed for an anomaly, it will be automatically baselined and the anomaly detector will treat new occurrences of a similar anomaly as normal. Therefore, if you do not correct the cause of an anomaly during the time period specified in `anomalyVisibilityTime`, it will be considered normal going forward and will not be detected as an anomaly. Valid Range: Minimum value of 7. Maximum value of 90.
     */
    anomalyVisibilityTime?: pulumi.Input<number>;
    /**
     * ARN of the log anomaly detector that you just created.
     */
    arn?: pulumi.Input<string>;
    /**
     * Name for this anomaly detector.
     */
    detectorName?: pulumi.Input<string>;
    enabled?: pulumi.Input<boolean>;
    /**
     * Specifies how often the anomaly detector is to run and look for anomalies. Set this value according to the frequency that the log group receives new logs. For example, if the log group receives new log events every 10 minutes, then 15 minutes might be a good setting for `evaluationFrequency`. Valid Values: `ONE_MIN | FIVE_MIN | TEN_MIN | FIFTEEN_MIN | THIRTY_MIN | ONE_HOUR`.
     */
    evaluationFrequency?: pulumi.Input<string>;
    /**
     * You can use this parameter to limit the anomaly detection model to examine only log events that match the pattern you specify here. For more information, see [Filter and Pattern Syntax](https://docs.aws.amazon.com/AmazonCloudWatch/latest/logs/FilterAndPatternSyntax.html).
     */
    filterPattern?: pulumi.Input<string>;
    /**
     * Optionally assigns a AWS KMS key to secure this anomaly detector and its findings. If a key is assigned, the anomalies found and the model used by this detector are encrypted at rest with the key. If a key is assigned to an anomaly detector, a user must have permissions for both this key and for the anomaly detector to retrieve information about the anomalies that it finds.
     */
    kmsKeyId?: pulumi.Input<string>;
    /**
     * Array containing the ARN of the log group that this anomaly detector will watch. You can specify only one log group ARN.
     *
     * The following arguments are optional:
     */
    logGroupArnLists?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
/**
 * The set of arguments for constructing a LogAnomalyDetector resource.
 */
export interface LogAnomalyDetectorArgs {
    /**
     * Number of days to have visibility on an anomaly. After this time period has elapsed for an anomaly, it will be automatically baselined and the anomaly detector will treat new occurrences of a similar anomaly as normal. Therefore, if you do not correct the cause of an anomaly during the time period specified in `anomalyVisibilityTime`, it will be considered normal going forward and will not be detected as an anomaly. Valid Range: Minimum value of 7. Maximum value of 90.
     */
    anomalyVisibilityTime?: pulumi.Input<number>;
    /**
     * Name for this anomaly detector.
     */
    detectorName?: pulumi.Input<string>;
    enabled: pulumi.Input<boolean>;
    /**
     * Specifies how often the anomaly detector is to run and look for anomalies. Set this value according to the frequency that the log group receives new logs. For example, if the log group receives new log events every 10 minutes, then 15 minutes might be a good setting for `evaluationFrequency`. Valid Values: `ONE_MIN | FIVE_MIN | TEN_MIN | FIFTEEN_MIN | THIRTY_MIN | ONE_HOUR`.
     */
    evaluationFrequency?: pulumi.Input<string>;
    /**
     * You can use this parameter to limit the anomaly detection model to examine only log events that match the pattern you specify here. For more information, see [Filter and Pattern Syntax](https://docs.aws.amazon.com/AmazonCloudWatch/latest/logs/FilterAndPatternSyntax.html).
     */
    filterPattern?: pulumi.Input<string>;
    /**
     * Optionally assigns a AWS KMS key to secure this anomaly detector and its findings. If a key is assigned, the anomalies found and the model used by this detector are encrypted at rest with the key. If a key is assigned to an anomaly detector, a user must have permissions for both this key and for the anomaly detector to retrieve information about the anomalies that it finds.
     */
    kmsKeyId?: pulumi.Input<string>;
    /**
     * Array containing the ARN of the log group that this anomaly detector will watch. You can specify only one log group ARN.
     *
     * The following arguments are optional:
     */
    logGroupArnLists: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
