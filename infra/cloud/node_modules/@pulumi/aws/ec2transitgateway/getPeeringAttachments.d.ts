import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Get information on EC2 Transit Gateway Peering Attachments.
 *
 * ## Example Usage
 *
 * ### All Resources
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = aws.ec2transitgateway.getPeeringAttachments({});
 * ```
 *
 * ### By Filter
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const filtered = aws.ec2transitgateway.getPeeringAttachments({
 *     filters: [{
 *         name: "state",
 *         values: ["pendingAcceptance"],
 *     }],
 * });
 * const unit = .map(__index => (aws.ec2transitgateway.getPeeringAttachment({
 *     id: _arg0_.ids[__index],
 * })));
 * ```
 */
export declare function getPeeringAttachments(args?: GetPeeringAttachmentsArgs, opts?: pulumi.InvokeOptions): Promise<GetPeeringAttachmentsResult>;
/**
 * A collection of arguments for invoking getPeeringAttachments.
 */
export interface GetPeeringAttachmentsArgs {
    /**
     * One or more configuration blocks containing name-values filters. Detailed below.
     */
    filters?: inputs.ec2transitgateway.GetPeeringAttachmentsFilter[];
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
}
/**
 * A collection of values returned by getPeeringAttachments.
 */
export interface GetPeeringAttachmentsResult {
    readonly filters?: outputs.ec2transitgateway.GetPeeringAttachmentsFilter[];
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    /**
     * A list of all attachments ids matching the filter. You can retrieve more information about the attachment using the [aws.ec2transitgateway.PeeringAttachment][2] data source, searching by identifier.
     */
    readonly ids: string[];
    readonly region: string;
}
/**
 * Get information on EC2 Transit Gateway Peering Attachments.
 *
 * ## Example Usage
 *
 * ### All Resources
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = aws.ec2transitgateway.getPeeringAttachments({});
 * ```
 *
 * ### By Filter
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const filtered = aws.ec2transitgateway.getPeeringAttachments({
 *     filters: [{
 *         name: "state",
 *         values: ["pendingAcceptance"],
 *     }],
 * });
 * const unit = .map(__index => (aws.ec2transitgateway.getPeeringAttachment({
 *     id: _arg0_.ids[__index],
 * })));
 * ```
 */
export declare function getPeeringAttachmentsOutput(args?: GetPeeringAttachmentsOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetPeeringAttachmentsResult>;
/**
 * A collection of arguments for invoking getPeeringAttachments.
 */
export interface GetPeeringAttachmentsOutputArgs {
    /**
     * One or more configuration blocks containing name-values filters. Detailed below.
     */
    filters?: pulumi.Input<pulumi.Input<inputs.ec2transitgateway.GetPeeringAttachmentsFilterArgs>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
