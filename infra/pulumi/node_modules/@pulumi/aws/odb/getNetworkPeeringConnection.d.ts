import * as pulumi from "@pulumi/pulumi";
/**
 * Data source for managing oracle database network peering resource in AWS.
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
 * const example = aws.odb.getNetworkPeeringConnection({
 *     id: "example",
 * });
 * ```
 */
export declare function getNetworkPeeringConnection(args: GetNetworkPeeringConnectionArgs, opts?: pulumi.InvokeOptions): Promise<GetNetworkPeeringConnectionResult>;
/**
 * A collection of arguments for invoking getNetworkPeeringConnection.
 */
export interface GetNetworkPeeringConnectionArgs {
    /**
     * The unique identifier of the ODB network peering connection.
     *
     * The following arguments are optional:
     */
    id: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
}
/**
 * A collection of values returned by getNetworkPeeringConnection.
 */
export interface GetNetworkPeeringConnectionResult {
    /**
     * The Amazon Resource Name (ARN) for the  ODB network peering connection.
     */
    readonly arn: string;
    /**
     * Created time of the ODB network peering connection.
     */
    readonly createdAt: string;
    /**
     * Display name of the ODB network peering connection.
     */
    readonly displayName: string;
    readonly id: string;
    /**
     * ARN of the ODB network peering connection.
     */
    readonly odbNetworkArn: string;
    /**
     * Type of the ODB peering connection.
     */
    readonly odbPeeringConnectionType: string;
    /**
     * ARN of the peer network peering connection.
     */
    readonly peerNetworkArn: string;
    /**
     * Progress of the ODB network peering connection.
     */
    readonly percentProgress: number;
    readonly region: string;
    /**
     * Status of the ODB network peering connection.
     */
    readonly status: string;
    /**
     * Status of the ODB network peering connection.
     */
    readonly statusReason: string;
    /**
     * Tags applied to the resource.
     */
    readonly tags: {
        [key: string]: string;
    };
}
/**
 * Data source for managing oracle database network peering resource in AWS.
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
 * const example = aws.odb.getNetworkPeeringConnection({
 *     id: "example",
 * });
 * ```
 */
export declare function getNetworkPeeringConnectionOutput(args: GetNetworkPeeringConnectionOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetNetworkPeeringConnectionResult>;
/**
 * A collection of arguments for invoking getNetworkPeeringConnection.
 */
export interface GetNetworkPeeringConnectionOutputArgs {
    /**
     * The unique identifier of the ODB network peering connection.
     *
     * The following arguments are optional:
     */
    id: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
