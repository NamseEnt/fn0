import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides information for multiple EC2 Transit Gateway Route Table Propagations, such as their identifiers.
 *
 * ## Example Usage
 *
 * ### By Transit Gateway Identifier
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.ec2transitgateway.getRouteTablePropagations({
 *     transitGatewayRouteTableId: exampleAwsEc2TransitGatewayRouteTable.id,
 * });
 * ```
 */
export declare function getRouteTablePropagations(args: GetRouteTablePropagationsArgs, opts?: pulumi.InvokeOptions): Promise<GetRouteTablePropagationsResult>;
/**
 * A collection of arguments for invoking getRouteTablePropagations.
 */
export interface GetRouteTablePropagationsArgs {
    /**
     * Custom filter block as described below.
     *
     * More complex filters can be expressed using one or more `filter` sub-blocks,
     * which take the following arguments:
     */
    filters?: inputs.ec2transitgateway.GetRouteTablePropagationsFilter[];
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
    /**
     * Identifier of EC2 Transit Gateway Route Table.
     */
    transitGatewayRouteTableId: string;
}
/**
 * A collection of values returned by getRouteTablePropagations.
 */
export interface GetRouteTablePropagationsResult {
    readonly filters?: outputs.ec2transitgateway.GetRouteTablePropagationsFilter[];
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    /**
     * Set of Transit Gateway Route Table Association identifiers.
     */
    readonly ids: string[];
    readonly region: string;
    readonly transitGatewayRouteTableId: string;
}
/**
 * Provides information for multiple EC2 Transit Gateway Route Table Propagations, such as their identifiers.
 *
 * ## Example Usage
 *
 * ### By Transit Gateway Identifier
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.ec2transitgateway.getRouteTablePropagations({
 *     transitGatewayRouteTableId: exampleAwsEc2TransitGatewayRouteTable.id,
 * });
 * ```
 */
export declare function getRouteTablePropagationsOutput(args: GetRouteTablePropagationsOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetRouteTablePropagationsResult>;
/**
 * A collection of arguments for invoking getRouteTablePropagations.
 */
export interface GetRouteTablePropagationsOutputArgs {
    /**
     * Custom filter block as described below.
     *
     * More complex filters can be expressed using one or more `filter` sub-blocks,
     * which take the following arguments:
     */
    filters?: pulumi.Input<pulumi.Input<inputs.ec2transitgateway.GetRouteTablePropagationsFilterArgs>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Identifier of EC2 Transit Gateway Route Table.
     */
    transitGatewayRouteTableId: pulumi.Input<string>;
}
