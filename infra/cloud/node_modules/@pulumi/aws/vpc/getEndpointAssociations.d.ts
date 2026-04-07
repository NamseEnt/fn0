import * as pulumi from "@pulumi/pulumi";
import * as outputs from "../types/output";
/**
 * Data source for managing an AWS EC2 (Elastic Compute Cloud) Vpc Endpoint Associations.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.vpc.getEndpointAssociations({
 *     vpcEndpointId: exampleAwsVpcEndpoint.id,
 * });
 * ```
 */
export declare function getEndpointAssociations(args: GetEndpointAssociationsArgs, opts?: pulumi.InvokeOptions): Promise<GetEndpointAssociationsResult>;
/**
 * A collection of arguments for invoking getEndpointAssociations.
 */
export interface GetEndpointAssociationsArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
    /**
     * ID of the specific VPC Endpoint to retrieve.
     */
    vpcEndpointId: string;
}
/**
 * A collection of values returned by getEndpointAssociations.
 */
export interface GetEndpointAssociationsResult {
    /**
     * Associations for the VPC Endpoint. Association blocks are documented below.
     */
    readonly associations: outputs.vpc.GetEndpointAssociationsAssociation[];
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly region: string;
    readonly vpcEndpointId: string;
}
/**
 * Data source for managing an AWS EC2 (Elastic Compute Cloud) Vpc Endpoint Associations.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.vpc.getEndpointAssociations({
 *     vpcEndpointId: exampleAwsVpcEndpoint.id,
 * });
 * ```
 */
export declare function getEndpointAssociationsOutput(args: GetEndpointAssociationsOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetEndpointAssociationsResult>;
/**
 * A collection of arguments for invoking getEndpointAssociations.
 */
export interface GetEndpointAssociationsOutputArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ID of the specific VPC Endpoint to retrieve.
     */
    vpcEndpointId: pulumi.Input<string>;
}
