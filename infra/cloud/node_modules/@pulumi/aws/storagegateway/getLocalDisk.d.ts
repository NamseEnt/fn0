import * as pulumi from "@pulumi/pulumi";
/**
 * Retrieve information about a Storage Gateway local disk. The disk identifier is useful for adding the disk as a cache or upload buffer to a gateway.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = aws.storagegateway.getLocalDisk({
 *     diskPath: testAwsVolumeAttachment.deviceName,
 *     gatewayArn: testAwsStoragegatewayGateway.arn,
 * });
 * ```
 */
export declare function getLocalDisk(args: GetLocalDiskArgs, opts?: pulumi.InvokeOptions): Promise<GetLocalDiskResult>;
/**
 * A collection of arguments for invoking getLocalDisk.
 */
export interface GetLocalDiskArgs {
    /**
     * Device node of the local disk to retrieve. For example, `/dev/sdb`.
     */
    diskNode?: string;
    /**
     * Device path of the local disk to retrieve. For example, `/dev/xvdb` or `/dev/nvme1n1`.
     */
    diskPath?: string;
    /**
     * ARN of the gateway.
     */
    gatewayArn: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
}
/**
 * A collection of values returned by getLocalDisk.
 */
export interface GetLocalDiskResult {
    /**
     * Disk identifierE.g., `pci-0000:03:00.0-scsi-0:0:0:0`
     */
    readonly diskId: string;
    readonly diskNode: string;
    readonly diskPath: string;
    readonly gatewayArn: string;
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly region: string;
}
/**
 * Retrieve information about a Storage Gateway local disk. The disk identifier is useful for adding the disk as a cache or upload buffer to a gateway.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = aws.storagegateway.getLocalDisk({
 *     diskPath: testAwsVolumeAttachment.deviceName,
 *     gatewayArn: testAwsStoragegatewayGateway.arn,
 * });
 * ```
 */
export declare function getLocalDiskOutput(args: GetLocalDiskOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetLocalDiskResult>;
/**
 * A collection of arguments for invoking getLocalDisk.
 */
export interface GetLocalDiskOutputArgs {
    /**
     * Device node of the local disk to retrieve. For example, `/dev/sdb`.
     */
    diskNode?: pulumi.Input<string>;
    /**
     * Device path of the local disk to retrieve. For example, `/dev/xvdb` or `/dev/nvme1n1`.
     */
    diskPath?: pulumi.Input<string>;
    /**
     * ARN of the gateway.
     */
    gatewayArn: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
