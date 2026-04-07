import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Data source for retrieving all oracle database network peering resource in Oracle Database@AWS.
 *
 * You can find out more about Oracle Database@AWS from [User Guide](https://docs.aws.amazon.com/odb/latest/UserGuide/what-is-odb.html).
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.odb.getNetworkPeeringConnections({});
 * ```
 */
export declare function getNetworkPeeringConnections(args?: GetNetworkPeeringConnectionsArgs, opts?: pulumi.InvokeOptions): Promise<GetNetworkPeeringConnectionsResult>;
/**
 * A collection of arguments for invoking getNetworkPeeringConnections.
 */
export interface GetNetworkPeeringConnectionsArgs {
    /**
     * The list of ODB peering connections. A summary of an ODB peering connection.
     */
    odbPeeringConnections?: inputs.odb.GetNetworkPeeringConnectionsOdbPeeringConnection[];
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
}
/**
 * A collection of values returned by getNetworkPeeringConnections.
 */
export interface GetNetworkPeeringConnectionsResult {
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    /**
     * The list of ODB peering connections. A summary of an ODB peering connection.
     */
    readonly odbPeeringConnections?: outputs.odb.GetNetworkPeeringConnectionsOdbPeeringConnection[];
    readonly region: string;
}
/**
 * Data source for retrieving all oracle database network peering resource in Oracle Database@AWS.
 *
 * You can find out more about Oracle Database@AWS from [User Guide](https://docs.aws.amazon.com/odb/latest/UserGuide/what-is-odb.html).
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.odb.getNetworkPeeringConnections({});
 * ```
 */
export declare function getNetworkPeeringConnectionsOutput(args?: GetNetworkPeeringConnectionsOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetNetworkPeeringConnectionsResult>;
/**
 * A collection of arguments for invoking getNetworkPeeringConnections.
 */
export interface GetNetworkPeeringConnectionsOutputArgs {
    /**
     * The list of ODB peering connections. A summary of an ODB peering connection.
     */
    odbPeeringConnections?: pulumi.Input<pulumi.Input<inputs.odb.GetNetworkPeeringConnectionsOdbPeeringConnectionArgs>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
