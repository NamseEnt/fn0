import * as pulumi from "@pulumi/pulumi";
import * as outputs from "../types/output";
/**
 * Provides details about a specific Amazon Connect Hours of Operation.
 *
 * ## Example Usage
 *
 * By `name`
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = aws.connect.getHoursOfOperation({
 *     instanceId: "aaaaaaaa-bbbb-cccc-dddd-111111111111",
 *     name: "Test",
 * });
 * ```
 *
 * By `hoursOfOperationId`
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = aws.connect.getHoursOfOperation({
 *     instanceId: "aaaaaaaa-bbbb-cccc-dddd-111111111111",
 *     hoursOfOperationId: "cccccccc-bbbb-cccc-dddd-111111111111",
 * });
 * ```
 */
export declare function getHoursOfOperation(args: GetHoursOfOperationArgs, opts?: pulumi.InvokeOptions): Promise<GetHoursOfOperationResult>;
/**
 * A collection of arguments for invoking getHoursOfOperation.
 */
export interface GetHoursOfOperationArgs {
    /**
     * Returns information on a specific Hours of Operation by hours of operation id
     */
    hoursOfOperationId?: string;
    /**
     * Reference to the hosting Amazon Connect Instance
     */
    instanceId: string;
    /**
     * Returns information on a specific Hours of Operation by name
     *
     * > **NOTE:** `instanceId` and one of either `name` or `hoursOfOperationId` is required.
     */
    name?: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
    /**
     * Map of tags to assign to the Hours of Operation.
     */
    tags?: {
        [key: string]: string;
    };
}
/**
 * A collection of values returned by getHoursOfOperation.
 */
export interface GetHoursOfOperationResult {
    /**
     * ARN of the Hours of Operation.
     */
    readonly arn: string;
    /**
     * Configuration information for the hours of operation: day, start time, and end time . Config blocks are documented below. Config blocks are documented below.
     */
    readonly configs: outputs.connect.GetHoursOfOperationConfig[];
    /**
     * Description of the Hours of Operation.
     */
    readonly description: string;
    /**
     * The identifier for the hours of operation.
     */
    readonly hoursOfOperationId: string;
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    /**
     * Identifier of the hosting Amazon Connect Instance.
     */
    readonly instanceId: string;
    /**
     * Name of the Hours of Operation.
     */
    readonly name: string;
    readonly region: string;
    /**
     * Map of tags to assign to the Hours of Operation.
     */
    readonly tags: {
        [key: string]: string;
    };
    /**
     * Time zone of the Hours of Operation.
     */
    readonly timeZone: string;
}
/**
 * Provides details about a specific Amazon Connect Hours of Operation.
 *
 * ## Example Usage
 *
 * By `name`
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = aws.connect.getHoursOfOperation({
 *     instanceId: "aaaaaaaa-bbbb-cccc-dddd-111111111111",
 *     name: "Test",
 * });
 * ```
 *
 * By `hoursOfOperationId`
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = aws.connect.getHoursOfOperation({
 *     instanceId: "aaaaaaaa-bbbb-cccc-dddd-111111111111",
 *     hoursOfOperationId: "cccccccc-bbbb-cccc-dddd-111111111111",
 * });
 * ```
 */
export declare function getHoursOfOperationOutput(args: GetHoursOfOperationOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetHoursOfOperationResult>;
/**
 * A collection of arguments for invoking getHoursOfOperation.
 */
export interface GetHoursOfOperationOutputArgs {
    /**
     * Returns information on a specific Hours of Operation by hours of operation id
     */
    hoursOfOperationId?: pulumi.Input<string>;
    /**
     * Reference to the hosting Amazon Connect Instance
     */
    instanceId: pulumi.Input<string>;
    /**
     * Returns information on a specific Hours of Operation by name
     *
     * > **NOTE:** `instanceId` and one of either `name` or `hoursOfOperationId` is required.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Map of tags to assign to the Hours of Operation.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
