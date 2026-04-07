import * as pulumi from "@pulumi/pulumi";
/**
 * Manages an AWS Storage Gateway upload buffer.
 *
 * > **NOTE:** The Storage Gateway API provides no method to remove an upload buffer disk. Destroying this resource does not perform any Storage Gateway actions.
 *
 * ## Example Usage
 *
 * ### Cached and VTL Gateway Type
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = aws.storagegateway.getLocalDisk({
 *     diskNode: testAwsVolumeAttachment.deviceName,
 *     gatewayArn: testAwsStoragegatewayGateway.arn,
 * });
 * const testUploadBuffer = new aws.storagegateway.UploadBuffer("test", {
 *     diskPath: test.then(test => test.diskPath),
 *     gatewayArn: testAwsStoragegatewayGateway.arn,
 * });
 * ```
 *
 * ### Stored Gateway Type
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = aws.storagegateway.getLocalDisk({
 *     diskNode: testAwsVolumeAttachment.deviceName,
 *     gatewayArn: testAwsStoragegatewayGateway.arn,
 * });
 * const example = new aws.storagegateway.UploadBuffer("example", {
 *     diskId: exampleAwsStoragegatewayLocalDisk.id,
 *     gatewayArn: exampleAwsStoragegatewayGateway.arn,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_storagegateway_upload_buffer` using the gateway Amazon Resource Name (ARN) and local disk identifier separated with a colon (`:`). For example:
 *
 * ```sh
 * $ pulumi import aws:storagegateway/uploadBuffer:UploadBuffer example arn:aws:storagegateway:us-east-1:123456789012:gateway/sgw-12345678:pci-0000:03:00.0-scsi-0:0:0:0
 * ```
 */
export declare class UploadBuffer extends pulumi.CustomResource {
    /**
     * Get an existing UploadBuffer resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: UploadBufferState, opts?: pulumi.CustomResourceOptions): UploadBuffer;
    /**
     * Returns true if the given object is an instance of UploadBuffer.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is UploadBuffer;
    /**
     * Local disk identifier. For example, `pci-0000:03:00.0-scsi-0:0:0:0`.
     */
    readonly diskId: pulumi.Output<string>;
    /**
     * Local disk path. For example, `/dev/nvme1n1`.
     */
    readonly diskPath: pulumi.Output<string>;
    /**
     * The Amazon Resource Name (ARN) of the gateway.
     */
    readonly gatewayArn: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Create a UploadBuffer resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: UploadBufferArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering UploadBuffer resources.
 */
export interface UploadBufferState {
    /**
     * Local disk identifier. For example, `pci-0000:03:00.0-scsi-0:0:0:0`.
     */
    diskId?: pulumi.Input<string>;
    /**
     * Local disk path. For example, `/dev/nvme1n1`.
     */
    diskPath?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) of the gateway.
     */
    gatewayArn?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a UploadBuffer resource.
 */
export interface UploadBufferArgs {
    /**
     * Local disk identifier. For example, `pci-0000:03:00.0-scsi-0:0:0:0`.
     */
    diskId?: pulumi.Input<string>;
    /**
     * Local disk path. For example, `/dev/nvme1n1`.
     */
    diskPath?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) of the gateway.
     */
    gatewayArn: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
