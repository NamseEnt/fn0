import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Get information on EC2 Transit Gateway VPC Attachments.
 *
 * ## Example Usage
 *
 * ### By Filter
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const filtered = aws.ec2transitgateway.getVpcAttachments({
 *     filters: [{
 *         name: "state",
 *         values: ["pendingAcceptance"],
 *     }],
 * });
 * const unit = .map(__index => (aws.ec2transitgateway.getVpcAttachment({
 *     id: _arg0_.ids[__index],
 * })));
 * ```
 */
export declare function getVpcAttachments(args?: GetVpcAttachmentsArgs, opts?: pulumi.InvokeOptions): Promise<GetVpcAttachmentsResult>;
/**
 * A collection of arguments for invoking getVpcAttachments.
 */
export interface GetVpcAttachmentsArgs {
    /**
     * One or more configuration blocks containing name-values filters. Detailed below.
     */
    filters?: inputs.ec2transitgateway.GetVpcAttachmentsFilter[];
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
}
/**
 * A collection of values returned by getVpcAttachments.
 */
export interface GetVpcAttachmentsResult {
    readonly filters?: outputs.ec2transitgateway.GetVpcAttachmentsFilter[];
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    /**
     * A list of all attachments ids matching the filter. You can retrieve more information about the attachment using the [aws.ec2transitgateway.VpcAttachment][2] data source, searching by identifier.
     */
    readonly ids: string[];
    readonly region: string;
}
/**
 * Get information on EC2 Transit Gateway VPC Attachments.
 *
 * ## Example Usage
 *
 * ### By Filter
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const filtered = aws.ec2transitgateway.getVpcAttachments({
 *     filters: [{
 *         name: "state",
 *         values: ["pendingAcceptance"],
 *     }],
 * });
 * const unit = .map(__index => (aws.ec2transitgateway.getVpcAttachment({
 *     id: _arg0_.ids[__index],
 * })));
 * ```
 */
export declare function getVpcAttachmentsOutput(args?: GetVpcAttachmentsOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetVpcAttachmentsResult>;
/**
 * A collection of arguments for invoking getVpcAttachments.
 */
export interface GetVpcAttachmentsOutputArgs {
    /**
     * One or more configuration blocks containing name-values filters. Detailed below.
     */
    filters?: pulumi.Input<pulumi.Input<inputs.ec2transitgateway.GetVpcAttachmentsFilterArgs>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
