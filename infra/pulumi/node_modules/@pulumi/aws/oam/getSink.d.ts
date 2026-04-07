import * as pulumi from "@pulumi/pulumi";
/**
 * Data source for managing an AWS CloudWatch Observability Access Manager Sink.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.oam.getSink({
 *     sinkIdentifier: "arn:aws:oam:us-west-1:111111111111:sink/abcd1234-a123-456a-a12b-a123b456c789",
 * });
 * ```
 */
export declare function getSink(args: GetSinkArgs, opts?: pulumi.InvokeOptions): Promise<GetSinkResult>;
/**
 * A collection of arguments for invoking getSink.
 */
export interface GetSinkArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
    /**
     * ARN of the sink.
     */
    sinkIdentifier: string;
    /**
     * Tags assigned to the sink.
     */
    tags?: {
        [key: string]: string;
    };
}
/**
 * A collection of values returned by getSink.
 */
export interface GetSinkResult {
    /**
     * ARN of the sink.
     */
    readonly arn: string;
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    /**
     * Name of the sink.
     */
    readonly name: string;
    readonly region: string;
    /**
     * Random ID string that AWS generated as part of the sink ARN.
     */
    readonly sinkId: string;
    readonly sinkIdentifier: string;
    /**
     * Tags assigned to the sink.
     */
    readonly tags: {
        [key: string]: string;
    };
}
/**
 * Data source for managing an AWS CloudWatch Observability Access Manager Sink.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.oam.getSink({
 *     sinkIdentifier: "arn:aws:oam:us-west-1:111111111111:sink/abcd1234-a123-456a-a12b-a123b456c789",
 * });
 * ```
 */
export declare function getSinkOutput(args: GetSinkOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetSinkResult>;
/**
 * A collection of arguments for invoking getSink.
 */
export interface GetSinkOutputArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ARN of the sink.
     */
    sinkIdentifier: pulumi.Input<string>;
    /**
     * Tags assigned to the sink.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
