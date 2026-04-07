import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Get information on an EC2 Transit Gateway's attachment to a Direct Connect Gateway.
 *
 * !> **Warning:** Using the `aws.ec2transitgateway.getDirectConnectGatewayAttachment` data source in combination with  `aws.ec2transitgateway.RouteTablePropagation` or `aws.ec2transitgateway.RouteTableAssociation` may result in lost connectivity due to unnecessary resource re-creation. To avoid this, use the `transitGatewayAttachmentId` attribute directly from the `aws.directconnect.GatewayAssociation` resource. For example, `transitGatewayAttachmentId  = aws_dx_gateway_association.example.transit_gateway_attachment_id`.
 *
 * ## Example Usage
 *
 * ### By Transit Gateway and Direct Connect Gateway Identifiers
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.ec2transitgateway.getDirectConnectGatewayAttachment({
 *     transitGatewayId: exampleAwsEc2TransitGateway.id,
 *     dxGatewayId: exampleAwsDxGateway.id,
 * });
 * ```
 */
export declare function getDirectConnectGatewayAttachment(args?: GetDirectConnectGatewayAttachmentArgs, opts?: pulumi.InvokeOptions): Promise<GetDirectConnectGatewayAttachmentResult>;
/**
 * A collection of arguments for invoking getDirectConnectGatewayAttachment.
 */
export interface GetDirectConnectGatewayAttachmentArgs {
    /**
     * Identifier of the Direct Connect Gateway.
     */
    dxGatewayId?: string;
    /**
     * Configuration block(s) for filtering. Detailed below.
     */
    filters?: inputs.ec2transitgateway.GetDirectConnectGatewayAttachmentFilter[];
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
    /**
     * Map of tags, each pair of which must exactly match a pair on the desired Transit Gateway Direct Connect Gateway Attachment.
     */
    tags?: {
        [key: string]: string;
    };
    /**
     * Identifier of the EC2 Transit Gateway.
     */
    transitGatewayId?: string;
}
/**
 * A collection of values returned by getDirectConnectGatewayAttachment.
 */
export interface GetDirectConnectGatewayAttachmentResult {
    /**
     * ARN of the attachment.
     */
    readonly arn: string;
    readonly dxGatewayId?: string;
    readonly filters?: outputs.ec2transitgateway.GetDirectConnectGatewayAttachmentFilter[];
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly region: string;
    /**
     * Key-value tags for the EC2 Transit Gateway Attachment.
     */
    readonly tags: {
        [key: string]: string;
    };
    readonly transitGatewayId?: string;
}
/**
 * Get information on an EC2 Transit Gateway's attachment to a Direct Connect Gateway.
 *
 * !> **Warning:** Using the `aws.ec2transitgateway.getDirectConnectGatewayAttachment` data source in combination with  `aws.ec2transitgateway.RouteTablePropagation` or `aws.ec2transitgateway.RouteTableAssociation` may result in lost connectivity due to unnecessary resource re-creation. To avoid this, use the `transitGatewayAttachmentId` attribute directly from the `aws.directconnect.GatewayAssociation` resource. For example, `transitGatewayAttachmentId  = aws_dx_gateway_association.example.transit_gateway_attachment_id`.
 *
 * ## Example Usage
 *
 * ### By Transit Gateway and Direct Connect Gateway Identifiers
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.ec2transitgateway.getDirectConnectGatewayAttachment({
 *     transitGatewayId: exampleAwsEc2TransitGateway.id,
 *     dxGatewayId: exampleAwsDxGateway.id,
 * });
 * ```
 */
export declare function getDirectConnectGatewayAttachmentOutput(args?: GetDirectConnectGatewayAttachmentOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetDirectConnectGatewayAttachmentResult>;
/**
 * A collection of arguments for invoking getDirectConnectGatewayAttachment.
 */
export interface GetDirectConnectGatewayAttachmentOutputArgs {
    /**
     * Identifier of the Direct Connect Gateway.
     */
    dxGatewayId?: pulumi.Input<string>;
    /**
     * Configuration block(s) for filtering. Detailed below.
     */
    filters?: pulumi.Input<pulumi.Input<inputs.ec2transitgateway.GetDirectConnectGatewayAttachmentFilterArgs>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Map of tags, each pair of which must exactly match a pair on the desired Transit Gateway Direct Connect Gateway Attachment.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Identifier of the EC2 Transit Gateway.
     */
    transitGatewayId?: pulumi.Input<string>;
}
