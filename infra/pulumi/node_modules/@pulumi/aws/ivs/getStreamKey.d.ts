import * as pulumi from "@pulumi/pulumi";
/**
 * Data source for managing an AWS IVS (Interactive Video) Stream Key.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.ivs.getStreamKey({
 *     channelArn: "arn:aws:ivs:us-west-2:326937407773:channel/0Y1lcs4U7jk5",
 * });
 * ```
 */
export declare function getStreamKey(args: GetStreamKeyArgs, opts?: pulumi.InvokeOptions): Promise<GetStreamKeyResult>;
/**
 * A collection of arguments for invoking getStreamKey.
 */
export interface GetStreamKeyArgs {
    /**
     * ARN of the Channel.
     */
    channelArn: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
    /**
     * Map of tags assigned to the resource.
     */
    tags?: {
        [key: string]: string;
    };
}
/**
 * A collection of values returned by getStreamKey.
 */
export interface GetStreamKeyResult {
    /**
     * ARN of the Stream Key.
     */
    readonly arn: string;
    readonly channelArn: string;
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly region: string;
    /**
     * Map of tags assigned to the resource.
     */
    readonly tags: {
        [key: string]: string;
    };
    /**
     * Stream Key value.
     */
    readonly value: string;
}
/**
 * Data source for managing an AWS IVS (Interactive Video) Stream Key.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.ivs.getStreamKey({
 *     channelArn: "arn:aws:ivs:us-west-2:326937407773:channel/0Y1lcs4U7jk5",
 * });
 * ```
 */
export declare function getStreamKeyOutput(args: GetStreamKeyOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetStreamKeyResult>;
/**
 * A collection of arguments for invoking getStreamKey.
 */
export interface GetStreamKeyOutputArgs {
    /**
     * ARN of the Channel.
     */
    channelArn: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Map of tags assigned to the resource.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
