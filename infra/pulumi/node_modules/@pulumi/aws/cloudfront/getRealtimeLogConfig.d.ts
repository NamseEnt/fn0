import * as pulumi from "@pulumi/pulumi";
import * as outputs from "../types/output";
/**
 * Provides a CloudFront real-time log configuration resource.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.cloudfront.getRealtimeLogConfig({
 *     name: "example",
 * });
 * ```
 */
export declare function getRealtimeLogConfig(args: GetRealtimeLogConfigArgs, opts?: pulumi.InvokeOptions): Promise<GetRealtimeLogConfigResult>;
/**
 * A collection of arguments for invoking getRealtimeLogConfig.
 */
export interface GetRealtimeLogConfigArgs {
    /**
     * Unique name to identify this real-time log configuration.
     */
    name: string;
}
/**
 * A collection of values returned by getRealtimeLogConfig.
 */
export interface GetRealtimeLogConfigResult {
    /**
     * ARN (Amazon Resource Name) of the CloudFront real-time log configuration.
     */
    readonly arn: string;
    /**
     * (Required) Amazon Kinesis data streams where real-time log data is sent.
     */
    readonly endpoints: outputs.cloudfront.GetRealtimeLogConfigEndpoint[];
    /**
     * (Required) Fields that are included in each real-time log record. See the [AWS documentation](https://docs.aws.amazon.com/AmazonCloudFront/latest/DeveloperGuide/real-time-logs.html#understand-real-time-log-config-fields) for supported values.
     */
    readonly fields: string[];
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly name: string;
    /**
     * (Required) Sampling rate for this real-time log configuration. The sampling rate determines the percentage of viewer requests that are represented in the real-time log data. An integer between `1` and `100`, inclusive.
     */
    readonly samplingRate: number;
}
/**
 * Provides a CloudFront real-time log configuration resource.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.cloudfront.getRealtimeLogConfig({
 *     name: "example",
 * });
 * ```
 */
export declare function getRealtimeLogConfigOutput(args: GetRealtimeLogConfigOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetRealtimeLogConfigResult>;
/**
 * A collection of arguments for invoking getRealtimeLogConfig.
 */
export interface GetRealtimeLogConfigOutputArgs {
    /**
     * Unique name to identify this real-time log configuration.
     */
    name: pulumi.Input<string>;
}
