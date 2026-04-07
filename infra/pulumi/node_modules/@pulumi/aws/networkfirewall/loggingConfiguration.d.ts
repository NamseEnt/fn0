import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides an AWS Network Firewall Logging Configuration Resource
 *
 * ## Example Usage
 *
 * ### Logging to S3
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.networkfirewall.LoggingConfiguration("example", {
 *     firewallArn: exampleAwsNetworkfirewallFirewall.arn,
 *     loggingConfiguration: {
 *         logDestinationConfigs: [{
 *             logDestination: {
 *                 bucketName: exampleAwsS3Bucket.bucket,
 *                 prefix: "example",
 *             },
 *             logDestinationType: "S3",
 *             logType: "FLOW",
 *         }],
 *     },
 * });
 * ```
 *
 * ### Logging to CloudWatch
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.networkfirewall.LoggingConfiguration("example", {
 *     firewallArn: exampleAwsNetworkfirewallFirewall.arn,
 *     loggingConfiguration: {
 *         logDestinationConfigs: [{
 *             logDestination: {
 *                 logGroup: exampleAwsCloudwatchLogGroup.name,
 *             },
 *             logDestinationType: "CloudWatchLogs",
 *             logType: "ALERT",
 *         }],
 *     },
 * });
 * ```
 *
 * ### Logging to Kinesis Data Firehose
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.networkfirewall.LoggingConfiguration("example", {
 *     firewallArn: exampleAwsNetworkfirewallFirewall.arn,
 *     loggingConfiguration: {
 *         logDestinationConfigs: [{
 *             logDestination: {
 *                 deliveryStream: exampleAwsKinesisFirehoseDeliveryStream.name,
 *             },
 *             logDestinationType: "KinesisDataFirehose",
 *             logType: "TLS",
 *         }],
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Network Firewall Logging Configurations using the `firewall_arn`. For example:
 *
 * ```sh
 * $ pulumi import aws:networkfirewall/loggingConfiguration:LoggingConfiguration example arn:aws:network-firewall:us-west-1:123456789012:firewall/example
 * ```
 */
export declare class LoggingConfiguration extends pulumi.CustomResource {
    /**
     * Get an existing LoggingConfiguration resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: LoggingConfigurationState, opts?: pulumi.CustomResourceOptions): LoggingConfiguration;
    /**
     * Returns true if the given object is an instance of LoggingConfiguration.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is LoggingConfiguration;
    /**
     * Whether to enable the detailed firewall monitoring dashboard on the firewall. Defaults to `false`.
     */
    readonly enableMonitoringDashboard: pulumi.Output<boolean>;
    /**
     * The Amazon Resource Name (ARN) of the Network Firewall firewall.
     */
    readonly firewallArn: pulumi.Output<string>;
    /**
     * A configuration block describing how AWS Network Firewall performs logging for a firewall. See Logging Configuration below for details.
     */
    readonly loggingConfiguration: pulumi.Output<outputs.networkfirewall.LoggingConfigurationLoggingConfiguration>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Create a LoggingConfiguration resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: LoggingConfigurationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering LoggingConfiguration resources.
 */
export interface LoggingConfigurationState {
    /**
     * Whether to enable the detailed firewall monitoring dashboard on the firewall. Defaults to `false`.
     */
    enableMonitoringDashboard?: pulumi.Input<boolean>;
    /**
     * The Amazon Resource Name (ARN) of the Network Firewall firewall.
     */
    firewallArn?: pulumi.Input<string>;
    /**
     * A configuration block describing how AWS Network Firewall performs logging for a firewall. See Logging Configuration below for details.
     */
    loggingConfiguration?: pulumi.Input<inputs.networkfirewall.LoggingConfigurationLoggingConfiguration>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a LoggingConfiguration resource.
 */
export interface LoggingConfigurationArgs {
    /**
     * Whether to enable the detailed firewall monitoring dashboard on the firewall. Defaults to `false`.
     */
    enableMonitoringDashboard?: pulumi.Input<boolean>;
    /**
     * The Amazon Resource Name (ARN) of the Network Firewall firewall.
     */
    firewallArn: pulumi.Input<string>;
    /**
     * A configuration block describing how AWS Network Firewall performs logging for a firewall. See Logging Configuration below for details.
     */
    loggingConfiguration: pulumi.Input<inputs.networkfirewall.LoggingConfigurationLoggingConfiguration>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
