import * as pulumi from "@pulumi/pulumi";
/**
 * Retrieve information about a Direct Connect Gateway.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.directconnect.getGateway({
 *     name: "example",
 * });
 * ```
 */
export declare function getGateway(args: GetGatewayArgs, opts?: pulumi.InvokeOptions): Promise<GetGatewayResult>;
/**
 * A collection of arguments for invoking getGateway.
 */
export interface GetGatewayArgs {
    /**
     * Name of the gateway to retrieve.
     */
    name: string;
}
/**
 * A collection of values returned by getGateway.
 */
export interface GetGatewayResult {
    /**
     * ASN on the Amazon side of the connection.
     */
    readonly amazonSideAsn: string;
    /**
     * ARN of the gateway.
     */
    readonly arn: string;
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly name: string;
    /**
     * AWS Account ID of the gateway.
     */
    readonly ownerAccountId: string;
}
/**
 * Retrieve information about a Direct Connect Gateway.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.directconnect.getGateway({
 *     name: "example",
 * });
 * ```
 */
export declare function getGatewayOutput(args: GetGatewayOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetGatewayResult>;
/**
 * A collection of arguments for invoking getGateway.
 */
export interface GetGatewayOutputArgs {
    /**
     * Name of the gateway to retrieve.
     */
    name: pulumi.Input<string>;
}
