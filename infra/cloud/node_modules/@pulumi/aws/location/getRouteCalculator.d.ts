import * as pulumi from "@pulumi/pulumi";
/**
 * Retrieve information about a Location Service Route Calculator.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.location.getRouteCalculator({
 *     calculatorName: "example",
 * });
 * ```
 */
export declare function getRouteCalculator(args: GetRouteCalculatorArgs, opts?: pulumi.InvokeOptions): Promise<GetRouteCalculatorResult>;
/**
 * A collection of arguments for invoking getRouteCalculator.
 */
export interface GetRouteCalculatorArgs {
    /**
     * Name of the route calculator resource.
     */
    calculatorName: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
    /**
     * Key-value map of resource tags for the route calculator.
     */
    tags?: {
        [key: string]: string;
    };
}
/**
 * A collection of values returned by getRouteCalculator.
 */
export interface GetRouteCalculatorResult {
    /**
     * ARN for the Route calculator resource. Use the ARN when you specify a resource across AWS.
     */
    readonly calculatorArn: string;
    readonly calculatorName: string;
    /**
     * Timestamp for when the route calculator resource was created in ISO 8601 format.
     */
    readonly createTime: string;
    /**
     * Data provider of traffic and road network data.
     */
    readonly dataSource: string;
    /**
     * Optional description of the route calculator resource.
     */
    readonly description: string;
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly region: string;
    /**
     * Key-value map of resource tags for the route calculator.
     */
    readonly tags: {
        [key: string]: string;
    };
    /**
     * Timestamp for when the route calculator resource was last updated in ISO 8601 format.
     */
    readonly updateTime: string;
}
/**
 * Retrieve information about a Location Service Route Calculator.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.location.getRouteCalculator({
 *     calculatorName: "example",
 * });
 * ```
 */
export declare function getRouteCalculatorOutput(args: GetRouteCalculatorOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetRouteCalculatorResult>;
/**
 * A collection of arguments for invoking getRouteCalculator.
 */
export interface GetRouteCalculatorOutputArgs {
    /**
     * Name of the route calculator resource.
     */
    calculatorName: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Key-value map of resource tags for the route calculator.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
