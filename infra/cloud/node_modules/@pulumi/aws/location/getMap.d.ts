import * as pulumi from "@pulumi/pulumi";
import * as outputs from "../types/output";
/**
 * Retrieve information about a Location Service Map.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.location.getMap({
 *     mapName: "example",
 * });
 * ```
 */
export declare function getMap(args: GetMapArgs, opts?: pulumi.InvokeOptions): Promise<GetMapResult>;
/**
 * A collection of arguments for invoking getMap.
 */
export interface GetMapArgs {
    /**
     * Name of the map resource.
     */
    mapName: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
    /**
     * Key-value map of resource tags for the map.
     */
    tags?: {
        [key: string]: string;
    };
}
/**
 * A collection of values returned by getMap.
 */
export interface GetMapResult {
    /**
     * List of configurations that specify the map tile style selected from a partner data provider.
     */
    readonly configurations: outputs.location.GetMapConfiguration[];
    /**
     * Timestamp for when the map resource was created in ISO 8601 format.
     */
    readonly createTime: string;
    /**
     * Optional description for the map resource.
     */
    readonly description: string;
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    /**
     * ARN for the map resource.
     */
    readonly mapArn: string;
    readonly mapName: string;
    readonly region: string;
    /**
     * Key-value map of resource tags for the map.
     */
    readonly tags: {
        [key: string]: string;
    };
    /**
     * Timestamp for when the map resource was last updated in ISO 8601 format.
     */
    readonly updateTime: string;
}
/**
 * Retrieve information about a Location Service Map.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.location.getMap({
 *     mapName: "example",
 * });
 * ```
 */
export declare function getMapOutput(args: GetMapOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetMapResult>;
/**
 * A collection of arguments for invoking getMap.
 */
export interface GetMapOutputArgs {
    /**
     * Name of the map resource.
     */
    mapName: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Key-value map of resource tags for the map.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
