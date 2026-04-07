import * as pulumi from "@pulumi/pulumi";
/**
 * > There is only a single spot data feed subscription per account.
 *
 * Data source for accessing an AWS EC2 (Elastic Compute Cloud) spot data feed subscription.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const _default = aws.ec2.getSpotDatafeedSubscription({});
 * ```
 */
export declare function getSpotDatafeedSubscription(args?: GetSpotDatafeedSubscriptionArgs, opts?: pulumi.InvokeOptions): Promise<GetSpotDatafeedSubscriptionResult>;
/**
 * A collection of arguments for invoking getSpotDatafeedSubscription.
 */
export interface GetSpotDatafeedSubscriptionArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
}
/**
 * A collection of values returned by getSpotDatafeedSubscription.
 */
export interface GetSpotDatafeedSubscriptionResult {
    /**
     * The name of the Amazon S3 bucket where the spot instance data feed is located.
     */
    readonly bucket: string;
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    /**
     * The prefix for the data feed files.
     */
    readonly prefix: string;
    readonly region: string;
}
/**
 * > There is only a single spot data feed subscription per account.
 *
 * Data source for accessing an AWS EC2 (Elastic Compute Cloud) spot data feed subscription.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const _default = aws.ec2.getSpotDatafeedSubscription({});
 * ```
 */
export declare function getSpotDatafeedSubscriptionOutput(args?: GetSpotDatafeedSubscriptionOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetSpotDatafeedSubscriptionResult>;
/**
 * A collection of arguments for invoking getSpotDatafeedSubscription.
 */
export interface GetSpotDatafeedSubscriptionOutputArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
