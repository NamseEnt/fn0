import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides information for multiple EC2 Transit Gateway Route Table Associations, such as their identifiers.
 *
 * ## Example Usage
 *
 * ### By Transit Gateway Identifier
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.ec2transitgateway.getRouteTableAssociations({
 *     transitGatewayRouteTableId: exampleAwsEc2TransitGatewayRouteTable.id,
 * });
 * ```
 */
export declare function getRouteTableAssociations(args: GetRouteTableAssociationsArgs, opts?: pulumi.InvokeOptions): Promise<GetRouteTableAssociationsResult>;
/**
 * A collection of arguments for invoking getRouteTableAssociations.
 */
export interface GetRouteTableAssociationsArgs {
    /**
     * Custom filter block as described below.
     *
     * More complex filters can be expressed using one or more `filter` sub-blocks,
     * which take the following arguments:
     */
    filters?: inputs.ec2transitgateway.GetRouteTableAssociationsFilter[];
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
    /**
     * Identifier of EC2 Transit Gateway Route Table.
     *
     * The following arguments are optional:
     */
    transitGatewayRouteTableId: string;
}
/**
 * A collection of values returned by getRouteTableAssociations.
 */
export interface GetRouteTableAssociationsResult {
    readonly filters?: outputs.ec2transitgateway.GetRouteTableAssociationsFilter[];
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
 * Provides information for multiple EC2 Transit Gateway Route Table Associations, such as their identifiers.
 *
 * ## Example Usage
 *
 * ### By Transit Gateway Identifier
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.ec2transitgateway.getRouteTableAssociations({
 *     transitGatewayRouteTableId: exampleAwsEc2TransitGatewayRouteTable.id,
 * });
 * ```
 */
export declare function getRouteTableAssociationsOutput(args: GetRouteTableAssociationsOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetRouteTableAssociationsResult>;
/**
 * A collection of arguments for invoking getRouteTableAssociations.
 */
export interface GetRouteTableAssociationsOutputArgs {
    /**
     * Custom filter block as described below.
     *
     * More complex filters can be expressed using one or more `filter` sub-blocks,
     * which take the following arguments:
     */
    filters?: pulumi.Input<pulumi.Input<inputs.ec2transitgateway.GetRouteTableAssociationsFilterArgs>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Identifier of EC2 Transit Gateway Route Table.
     *
     * The following arguments are optional:
     */
    transitGatewayRouteTableId: pulumi.Input<string>;
}
