import * as pulumi from "@pulumi/pulumi";
/**
 * Provides access to all Environments for an AppConfig Application. This will allow you to pass Environment IDs to another
 * resource.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.appconfig.getEnvironments({
 *     applicationId: "a1d3rpe",
 * });
 * ```
 */
export declare function getEnvironments(args: GetEnvironmentsArgs, opts?: pulumi.InvokeOptions): Promise<GetEnvironmentsResult>;
/**
 * A collection of arguments for invoking getEnvironments.
 */
export interface GetEnvironmentsArgs {
    /**
     * ID of the AppConfig Application.
     */
    applicationId: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
}
/**
 * A collection of values returned by getEnvironments.
 */
export interface GetEnvironmentsResult {
    readonly applicationId: string;
    /**
     * Set of Environment IDs associated with this AppConfig Application.
     */
    readonly environmentIds: string[];
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly region: string;
}
/**
 * Provides access to all Environments for an AppConfig Application. This will allow you to pass Environment IDs to another
 * resource.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.appconfig.getEnvironments({
 *     applicationId: "a1d3rpe",
 * });
 * ```
 */
export declare function getEnvironmentsOutput(args: GetEnvironmentsOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetEnvironmentsResult>;
/**
 * A collection of arguments for invoking getEnvironments.
 */
export interface GetEnvironmentsOutputArgs {
    /**
     * ID of the AppConfig Application.
     */
    applicationId: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
