import * as pulumi from "@pulumi/pulumi";
import * as outputs from "../types/output";
/**
 * Data source for managing an AWS EventBridge Event Buses.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.cloudwatch.getEventBuses({
 *     namePrefix: "test",
 * });
 * ```
 */
export declare function getEventBuses(args?: GetEventBusesArgs, opts?: pulumi.InvokeOptions): Promise<GetEventBusesResult>;
/**
 * A collection of arguments for invoking getEventBuses.
 */
export interface GetEventBusesArgs {
    /**
     * Specifying this limits the results to only those event buses with names that start with the specified prefix.
     */
    namePrefix?: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
}
/**
 * A collection of values returned by getEventBuses.
 */
export interface GetEventBusesResult {
    /**
     * This list of event buses.
     */
    readonly eventBuses: outputs.cloudwatch.GetEventBusesEventBus[];
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly namePrefix?: string;
    readonly region: string;
}
/**
 * Data source for managing an AWS EventBridge Event Buses.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.cloudwatch.getEventBuses({
 *     namePrefix: "test",
 * });
 * ```
 */
export declare function getEventBusesOutput(args?: GetEventBusesOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetEventBusesResult>;
/**
 * A collection of arguments for invoking getEventBuses.
 */
export interface GetEventBusesOutputArgs {
    /**
     * Specifying this limits the results to only those event buses with names that start with the specified prefix.
     */
    namePrefix?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
