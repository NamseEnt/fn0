import * as pulumi from "@pulumi/pulumi";
/**
 * Data source for managing an AWS Glue Registry.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.glue.getRegistry({
 *     name: "example",
 * });
 * ```
 */
export declare function getRegistry(args: GetRegistryArgs, opts?: pulumi.InvokeOptions): Promise<GetRegistryResult>;
/**
 * A collection of arguments for invoking getRegistry.
 */
export interface GetRegistryArgs {
    /**
     * Name of the Glue Registry.
     */
    name: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
}
/**
 * A collection of values returned by getRegistry.
 */
export interface GetRegistryResult {
    /**
     * Amazon Resource Name (ARN) of Glue Registry.
     */
    readonly arn: string;
    /**
     * A description of the registry.
     */
    readonly description: string;
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly name: string;
    readonly region: string;
}
/**
 * Data source for managing an AWS Glue Registry.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.glue.getRegistry({
 *     name: "example",
 * });
 * ```
 */
export declare function getRegistryOutput(args: GetRegistryOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetRegistryResult>;
/**
 * A collection of arguments for invoking getRegistry.
 */
export interface GetRegistryOutputArgs {
    /**
     * Name of the Glue Registry.
     */
    name: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
