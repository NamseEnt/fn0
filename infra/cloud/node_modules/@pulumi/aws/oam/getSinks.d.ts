import * as pulumi from "@pulumi/pulumi";
/**
 * Data source for managing an AWS CloudWatch Observability Access Manager Sinks.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.oam.getSinks({});
 * ```
 */
export declare function getSinks(args?: GetSinksArgs, opts?: pulumi.InvokeOptions): Promise<GetSinksResult>;
/**
 * A collection of arguments for invoking getSinks.
 */
export interface GetSinksArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
}
/**
 * A collection of values returned by getSinks.
 */
export interface GetSinksResult {
    /**
     * Set of ARN of the Sinks.
     */
    readonly arns: string[];
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly region: string;
}
/**
 * Data source for managing an AWS CloudWatch Observability Access Manager Sinks.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.oam.getSinks({});
 * ```
 */
export declare function getSinksOutput(args?: GetSinksOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetSinksResult>;
/**
 * A collection of arguments for invoking getSinks.
 */
export interface GetSinksOutputArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
