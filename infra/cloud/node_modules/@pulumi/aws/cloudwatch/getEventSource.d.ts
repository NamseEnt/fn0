import * as pulumi from "@pulumi/pulumi";
/**
 * Use this data source to get information about an EventBridge Partner Event Source. This data source will only return one partner event source. An error will be returned if multiple sources match the same name prefix.
 *
 * > **Note:** EventBridge was formerly known as CloudWatch Events. The functionality is identical.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const examplepartner = aws.cloudwatch.getEventSource({
 *     namePrefix: "aws.partner/examplepartner.com",
 * });
 * ```
 */
export declare function getEventSource(args?: GetEventSourceArgs, opts?: pulumi.InvokeOptions): Promise<GetEventSourceResult>;
/**
 * A collection of arguments for invoking getEventSource.
 */
export interface GetEventSourceArgs {
    /**
     * Specifying this limits the results to only those partner event sources with names that start with the specified prefix
     */
    namePrefix?: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
}
/**
 * A collection of values returned by getEventSource.
 */
export interface GetEventSourceResult {
    /**
     * ARN of the partner event source
     */
    readonly arn: string;
    /**
     * Name of the SaaS partner that created the event source
     */
    readonly createdBy: string;
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    /**
     * Name of the event source
     */
    readonly name: string;
    readonly namePrefix?: string;
    readonly region: string;
    /**
     * State of the event source (`ACTIVE` or `PENDING`)
     */
    readonly state: string;
}
/**
 * Use this data source to get information about an EventBridge Partner Event Source. This data source will only return one partner event source. An error will be returned if multiple sources match the same name prefix.
 *
 * > **Note:** EventBridge was formerly known as CloudWatch Events. The functionality is identical.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const examplepartner = aws.cloudwatch.getEventSource({
 *     namePrefix: "aws.partner/examplepartner.com",
 * });
 * ```
 */
export declare function getEventSourceOutput(args?: GetEventSourceOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetEventSourceResult>;
/**
 * A collection of arguments for invoking getEventSource.
 */
export interface GetEventSourceOutputArgs {
    /**
     * Specifying this limits the results to only those partner event sources with names that start with the specified prefix
     */
    namePrefix?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
