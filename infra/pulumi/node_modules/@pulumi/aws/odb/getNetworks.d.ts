import * as pulumi from "@pulumi/pulumi";
import * as outputs from "../types/output";
/**
 * Data source for to retrieve networks from AWS for Oracle Database@AWS.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.odb.getNetworks({});
 * ```
 */
export declare function getNetworks(args?: GetNetworksArgs, opts?: pulumi.InvokeOptions): Promise<GetNetworksResult>;
/**
 * A collection of arguments for invoking getNetworks.
 */
export interface GetNetworksArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
}
/**
 * A collection of values returned by getNetworks.
 */
export interface GetNetworksResult {
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    /**
     * List of odb networks returns basic information about odb networks.
     */
    readonly odbNetworks: outputs.odb.GetNetworksOdbNetwork[];
    readonly region: string;
}
/**
 * Data source for to retrieve networks from AWS for Oracle Database@AWS.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.odb.getNetworks({});
 * ```
 */
export declare function getNetworksOutput(args?: GetNetworksOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetNetworksResult>;
/**
 * A collection of arguments for invoking getNetworks.
 */
export interface GetNetworksOutputArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
