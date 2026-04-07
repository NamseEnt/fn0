import * as pulumi from "@pulumi/pulumi";
/**
 * Manages an AWS Storage Gateway cached iSCSI volume.
 *
 * > **NOTE:** The gateway must have cache added (e.g., via the `aws.storagegateway.Cache` resource) before creating volumes otherwise the Storage Gateway API will return an error.
 *
 * > **NOTE:** The gateway must have an upload buffer added (e.g., via the `aws.storagegateway.UploadBuffer` resource) before the volume is operational to clients, however the Storage Gateway API will allow volume creation without error in that case and return volume status as `UPLOAD BUFFER NOT CONFIGURED`.
 *
 * ## Example Usage
 *
 * > **NOTE:** These examples are referencing the `aws.storagegateway.Cache` resource `gatewayArn` attribute to ensure this provider properly adds cache before creating the volume. If you are not using this method, you may need to declare an expicit dependency (e.g. via `dependsOn = [aws_storagegateway_cache.example]`) to ensure proper ordering.
 *
 * ### Create Empty Cached iSCSI Volume
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.storagegateway.CachesIscsiVolume("example", {
 *     gatewayArn: exampleAwsStoragegatewayCache.gatewayArn,
 *     networkInterfaceId: exampleAwsInstance.privateIp,
 *     targetName: "example",
 *     volumeSizeInBytes: 5368709120,
 * });
 * ```
 *
 * ### Create Cached iSCSI Volume From Snapshot
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.storagegateway.CachesIscsiVolume("example", {
 *     gatewayArn: exampleAwsStoragegatewayCache.gatewayArn,
 *     networkInterfaceId: exampleAwsInstance.privateIp,
 *     snapshotId: exampleAwsEbsSnapshot.id,
 *     targetName: "example",
 *     volumeSizeInBytes: exampleAwsEbsSnapshot.volumeSize * 1024 * 1024 * 1024,
 * });
 * ```
 *
 * ### Create Cached iSCSI Volume From Source Volume
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.storagegateway.CachesIscsiVolume("example", {
 *     gatewayArn: exampleAwsStoragegatewayCache.gatewayArn,
 *     networkInterfaceId: exampleAwsInstance.privateIp,
 *     sourceVolumeArn: existing.arn,
 *     targetName: "example",
 *     volumeSizeInBytes: existing.volumeSizeInBytes,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_storagegateway_cached_iscsi_volume` using the volume Amazon Resource Name (ARN). For example:
 *
 * ```sh
 * $ pulumi import aws:storagegateway/cachesIscsiVolume:CachesIscsiVolume example arn:aws:storagegateway:us-east-1:123456789012:gateway/sgw-12345678/volume/vol-12345678
 * ```
 */
export declare class CachesIscsiVolume extends pulumi.CustomResource {
    /**
     * Get an existing CachesIscsiVolume resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: CachesIscsiVolumeState, opts?: pulumi.CustomResourceOptions): CachesIscsiVolume;
    /**
     * Returns true if the given object is an instance of CachesIscsiVolume.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is CachesIscsiVolume;
    /**
     * Volume Amazon Resource Name (ARN), e.g., `arn:aws:storagegateway:us-east-1:123456789012:gateway/sgw-12345678/volume/vol-12345678`.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Whether mutual CHAP is enabled for the iSCSI target.
     */
    readonly chapEnabled: pulumi.Output<boolean>;
    /**
     * The Amazon Resource Name (ARN) of the gateway.
     */
    readonly gatewayArn: pulumi.Output<string>;
    /**
     * Set to `true` to use Amazon S3 server side encryption with your own AWS KMS key, or `false` to use a key managed by Amazon S3.
     */
    readonly kmsEncrypted: pulumi.Output<boolean | undefined>;
    /**
     * The Amazon Resource Name (ARN) of the AWS KMS key used for Amazon S3 server side encryption. Is required when `kmsEncrypted` is set.
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
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The snapshot ID of the snapshot to restore as the new cached volumeE.g., `snap-1122aabb`.
     */
    readonly snapshotId: pulumi.Output<string | undefined>;
    /**
     * The ARN for an existing volume. Specifying this ARN makes the new volume into an exact copy of the specified existing volume's latest recovery point. The `volumeSizeInBytes` value for this new volume must be equal to or larger than the size of the existing volume, in bytes.
     */
    readonly sourceVolumeArn: pulumi.Output<string | undefined>;
    /**
     * Key-value map of resource tags. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Volume Amazon Resource Name (ARN), e.g., `arn:aws:storagegateway:us-east-1:123456789012:gateway/sgw-12345678/volume/vol-12345678`.
     */
    readonly volumeArn: pulumi.Output<string>;
    /**
     * Volume ID, e.g., `vol-12345678`.
     */
    readonly volumeId: pulumi.Output<string>;
    /**
     * The size of the volume in bytes.
     */
    readonly volumeSizeInBytes: pulumi.Output<number>;
    /**
     * Create a CachesIscsiVolume resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: CachesIscsiVolumeArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering CachesIscsiVolume resources.
 */
export interface CachesIscsiVolumeState {
    /**
     * Volume Amazon Resource Name (ARN), e.g., `arn:aws:storagegateway:us-east-1:123456789012:gateway/sgw-12345678/volume/vol-12345678`.
     */
    arn?: pulumi.Input<string>;
    /**
     * Whether mutual CHAP is enabled for the iSCSI target.
     */
    chapEnabled?: pulumi.Input<boolean>;
    /**
     * The Amazon Resource Name (ARN) of the gateway.
     */
    gatewayArn?: pulumi.Input<string>;
    /**
     * Set to `true` to use Amazon S3 server side encryption with your own AWS KMS key, or `false` to use a key managed by Amazon S3.
     */
    kmsEncrypted?: pulumi.Input<boolean>;
    /**
     * The Amazon Resource Name (ARN) of the AWS KMS key used for Amazon S3 server side encryption. Is required when `kmsEncrypted` is set.
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
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The snapshot ID of the snapshot to restore as the new cached volumeE.g., `snap-1122aabb`.
     */
    snapshotId?: pulumi.Input<string>;
    /**
     * The ARN for an existing volume. Specifying this ARN makes the new volume into an exact copy of the specified existing volume's latest recovery point. The `volumeSizeInBytes` value for this new volume must be equal to or larger than the size of the existing volume, in bytes.
     */
    sourceVolumeArn?: pulumi.Input<string>;
    /**
     * Key-value map of resource tags. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Volume Amazon Resource Name (ARN), e.g., `arn:aws:storagegateway:us-east-1:123456789012:gateway/sgw-12345678/volume/vol-12345678`.
     */
    volumeArn?: pulumi.Input<string>;
    /**
     * Volume ID, e.g., `vol-12345678`.
     */
    volumeId?: pulumi.Input<string>;
    /**
     * The size of the volume in bytes.
     */
    volumeSizeInBytes?: pulumi.Input<number>;
}
/**
 * The set of arguments for constructing a CachesIscsiVolume resource.
 */
export interface CachesIscsiVolumeArgs {
    /**
     * The Amazon Resource Name (ARN) of the gateway.
     */
    gatewayArn: pulumi.Input<string>;
    /**
     * Set to `true` to use Amazon S3 server side encryption with your own AWS KMS key, or `false` to use a key managed by Amazon S3.
     */
    kmsEncrypted?: pulumi.Input<boolean>;
    /**
     * The Amazon Resource Name (ARN) of the AWS KMS key used for Amazon S3 server side encryption. Is required when `kmsEncrypted` is set.
     */
    kmsKey?: pulumi.Input<string>;
    /**
     * The network interface of the gateway on which to expose the iSCSI target. Only IPv4 addresses are accepted.
     */
    networkInterfaceId: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The snapshot ID of the snapshot to restore as the new cached volumeE.g., `snap-1122aabb`.
     */
    snapshotId?: pulumi.Input<string>;
    /**
     * The ARN for an existing volume. Specifying this ARN makes the new volume into an exact copy of the specified existing volume's latest recovery point. The `volumeSizeInBytes` value for this new volume must be equal to or larger than the size of the existing volume, in bytes.
     */
    sourceVolumeArn?: pulumi.Input<string>;
    /**
     * Key-value map of resource tags. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The name of the iSCSI target used by initiators to connect to the target and as a suffix for the target ARN. The target name must be unique across all volumes of a gateway.
     */
    targetName: pulumi.Input<string>;
    /**
     * The size of the volume in bytes.
     */
    volumeSizeInBytes: pulumi.Input<number>;
}
