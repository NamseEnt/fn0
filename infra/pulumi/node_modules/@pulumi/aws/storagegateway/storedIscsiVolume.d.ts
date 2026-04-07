import * as pulumi from "@pulumi/pulumi";
/**
 * Manages an AWS Storage Gateway stored iSCSI volume.
 *
 * > **NOTE:** The gateway must have a working storage added (e.g., via the `aws.storagegateway.WorkingStorage` resource) before the volume is operational to clients, however the Storage Gateway API will allow volume creation without error in that case and return volume status as `WORKING STORAGE NOT CONFIGURED`.
 *
 * ## Example Usage
 *
 * ### Create Empty Stored iSCSI Volume
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.storagegateway.StoredIscsiVolume("example", {
 *     gatewayArn: exampleAwsStoragegatewayCache.gatewayArn,
 *     networkInterfaceId: exampleAwsInstance.privateIp,
 *     targetName: "example",
 *     preserveExistingData: false,
 *     diskId: test.id,
 * });
 * ```
 *
 * ### Create Stored iSCSI Volume From Snapshot
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.storagegateway.StoredIscsiVolume("example", {
 *     gatewayArn: exampleAwsStoragegatewayCache.gatewayArn,
 *     networkInterfaceId: exampleAwsInstance.privateIp,
 *     snapshotId: exampleAwsEbsSnapshot.id,
 *     targetName: "example",
 *     preserveExistingData: false,
 *     diskId: test.id,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_storagegateway_stored_iscsi_volume` using the volume Amazon Resource Name (ARN). For example:
 *
 * ```sh
 * $ pulumi import aws:storagegateway/storedIscsiVolume:StoredIscsiVolume example arn:aws:storagegateway:us-east-1:123456789012:gateway/sgw-12345678/volume/vol-12345678
 * ```
 */
export declare class StoredIscsiVolume extends pulumi.CustomResource {
    /**
     * Get an existing StoredIscsiVolume resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: StoredIscsiVolumeState, opts?: pulumi.CustomResourceOptions): StoredIscsiVolume;
    /**
     * Returns true if the given object is an instance of StoredIscsiVolume.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is StoredIscsiVolume;
    /**
     * Volume Amazon Resource Name (ARN), e.g., `arn:aws:storagegateway:us-east-1:123456789012:gateway/sgw-12345678/volume/vol-12345678`.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Whether mutual CHAP is enabled for the iSCSI target.
     */
    readonly chapEnabled: pulumi.Output<boolean>;
    /**
     * The unique identifier for the gateway local disk that is configured as a stored volume.
     */
    readonly diskId: pulumi.Output<string>;
    /**
     * The Amazon Resource Name (ARN) of the gateway.
     */
    readonly gatewayArn: pulumi.Output<string>;
    /**
     * `true` to use Amazon S3 server side encryption with your own AWS KMS key, or `false` to use a key managed by Amazon S3. Optional.
     */
    readonly kmsEncrypted: pulumi.Output<boolean | undefined>;
    /**
     * The Amazon Resource Name (ARN) of the AWS KMS key used for Amazon S3 server side encryption. This value can only be set when `kmsEncrypted` is `true`.
     */
    readonly kmsKey: pulumi.Output<string | undefined>;
    /**
     * Logical disk number.
     */
    readonly lunNumber: pulumi.Output<number>;
    /**
     * The network interface of the gateway on which to expose the iSCSI target. Only IPv4 addresses are accepted.
     */
    readonly networkInterfaceId: pulumi.Output<string>;
    /**
     * The port used to communicate with iSCSI targets.
     */
    readonly networkInterfacePort: pulumi.Output<number>;
    /**
     * Specify this field as `true` if you want to preserve the data on the local disk. Otherwise, specifying this field as false creates an empty volume.
     */
    readonly preserveExistingData: pulumi.Output<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The snapshot ID of the snapshot to restore as the new stored volumeE.g., `snap-1122aabb`.
     */
    readonly snapshotId: pulumi.Output<string | undefined>;
    /**
     * Key-value mapping of resource tags. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * A map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * Target Amazon Resource Name (ARN), e.g., `arn:aws:storagegateway:us-east-1:123456789012:gateway/sgw-12345678/target/iqn.1997-05.com.amazon:TargetName`.
     */
    readonly targetArn: pulumi.Output<string>;
    /**
     * The name of the iSCSI target used by initiators to connect to the target and as a suffix for the target ARN. The target name must be unique across all volumes of a gateway.
     */
    readonly targetName: pulumi.Output<string>;
    /**
     * A value that indicates whether a storage volume is attached to, detached from, or is in the process of detaching from a gateway.
     */
    readonly volumeAttachmentStatus: pulumi.Output<string>;
    /**
     * Volume ID, e.g., `vol-12345678`.
     */
    readonly volumeId: pulumi.Output<string>;
    /**
     * The size of the data stored on the volume in bytes.
     */
    readonly volumeSizeInBytes: pulumi.Output<number>;
    /**
     * indicates the state of the storage volume.
     */
    readonly volumeStatus: pulumi.Output<string>;
    /**
     * indicates the type of the volume.
     */
    readonly volumeType: pulumi.Output<string>;
    /**
     * Create a StoredIscsiVolume resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: StoredIscsiVolumeArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering StoredIscsiVolume resources.
 */
export interface StoredIscsiVolumeState {
    /**
     * Volume Amazon Resource Name (ARN), e.g., `arn:aws:storagegateway:us-east-1:123456789012:gateway/sgw-12345678/volume/vol-12345678`.
     */
    arn?: pulumi.Input<string>;
    /**
     * Whether mutual CHAP is enabled for the iSCSI target.
     */
    chapEnabled?: pulumi.Input<boolean>;
    /**
     * The unique identifier for the gateway local disk that is configured as a stored volume.
     */
    diskId?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) of the gateway.
     */
    gatewayArn?: pulumi.Input<string>;
    /**
     * `true` to use Amazon S3 server side encryption with your own AWS KMS key, or `false` to use a key managed by Amazon S3. Optional.
     */
    kmsEncrypted?: pulumi.Input<boolean>;
    /**
     * The Amazon Resource Name (ARN) of the AWS KMS key used for Amazon S3 server side encryption. This value can only be set when `kmsEncrypted` is `true`.
     */
    kmsKey?: pulumi.Input<string>;
    /**
     * Logical disk number.
     */
    lunNumber?: pulumi.Input<number>;
    /**
     * The network interface of the gateway on which to expose the iSCSI target. Only IPv4 addresses are accepted.
     */
    networkInterfaceId?: pulumi.Input<string>;
    /**
     * The port used to communicate with iSCSI targets.
     */
    networkInterfacePort?: pulumi.Input<number>;
    /**
     * Specify this field as `true` if you want to preserve the data on the local disk. Otherwise, specifying this field as false creates an empty volume.
     */
    preserveExistingData?: pulumi.Input<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The snapshot ID of the snapshot to restore as the new stored volumeE.g., `snap-1122aabb`.
     */
    snapshotId?: pulumi.Input<string>;
    /**
     * Key-value mapping of resource tags. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * A map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Target Amazon Resource Name (ARN), e.g., `arn:aws:storagegateway:us-east-1:123456789012:gateway/sgw-12345678/target/iqn.1997-05.com.amazon:TargetName`.
     */
    targetArn?: pulumi.Input<string>;
    /**
     * The name of the iSCSI target used by initiators to connect to the target and as a suffix for the target ARN. The target name must be unique across all volumes of a gateway.
     */
    targetName?: pulumi.Input<string>;
    /**
     * A value that indicates whether a storage volume is attached to, detached from, or is in the process of detaching from a gateway.
     */
    volumeAttachmentStatus?: pulumi.Input<string>;
    /**
     * Volume ID, e.g., `vol-12345678`.
     */
    volumeId?: pulumi.Input<string>;
    /**
     * The size of the data stored on the volume in bytes.
     */
    volumeSizeInBytes?: pulumi.Input<number>;
    /**
     * indicates the state of the storage volume.
     */
    volumeStatus?: pulumi.Input<string>;
    /**
     * indicates the type of the volume.
     */
    volumeType?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a StoredIscsiVolume resource.
 */
export interface StoredIscsiVolumeArgs {
    /**
     * The unique identifier for the gateway local disk that is configured as a stored volume.
     */
    diskId: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) of the gateway.
     */
    gatewayArn: pulumi.Input<string>;
    /**
     * `true` to use Amazon S3 server side encryption with your own AWS KMS key, or `false` to use a key managed by Amazon S3. Optional.
     */
    kmsEncrypted?: pulumi.Input<boolean>;
    /**
     * The Amazon Resource Name (ARN) of the AWS KMS key used for Amazon S3 server side encryption. This value can only be set when `kmsEncrypted` is `true`.
     */
    kmsKey?: pulumi.Input<string>;
    /**
     * The network interface of the gateway on which to expose the iSCSI target. Only IPv4 addresses are accepted.
     */
    networkInterfaceId: pulumi.Input<string>;
    /**
     * Specify this field as `true` if you want to preserve the data on the local disk. Otherwise, specifying this field as false creates an empty volume.
     */
    preserveExistingData: pulumi.Input<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The snapshot ID of the snapshot to restore as the new stored volumeE.g., `snap-1122aabb`.
     */
    snapshotId?: pulumi.Input<string>;
    /**
     * Key-value mapping of resource tags. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The name of the iSCSI target used by initiators to connect to the target and as a suffix for the target ARN. The target name must be unique across all volumes of a gateway.
     */
    targetName: pulumi.Input<string>;
}
