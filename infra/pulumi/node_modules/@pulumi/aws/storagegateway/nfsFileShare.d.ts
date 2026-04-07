import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages an AWS Storage Gateway NFS File Share.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.storagegateway.NfsFileShare("example", {
 *     clientLists: ["0.0.0.0/0"],
 *     gatewayArn: exampleAwsStoragegatewayGateway.arn,
 *     locationArn: exampleAwsS3Bucket.arn,
 *     roleArn: exampleAwsIamRole.arn,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_storagegateway_nfs_file_share` using the NFS File Share Amazon Resource Name (ARN). For example:
 *
 * ```sh
 * $ pulumi import aws:storagegateway/nfsFileShare:NfsFileShare example arn:aws:storagegateway:us-east-1:123456789012:share/share-12345678
 * ```
 */
export declare class NfsFileShare extends pulumi.CustomResource {
    /**
     * Get an existing NfsFileShare resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: NfsFileShareState, opts?: pulumi.CustomResourceOptions): NfsFileShare;
    /**
     * Returns true if the given object is an instance of NfsFileShare.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is NfsFileShare;
    /**
     * Amazon Resource Name (ARN) of the NFS File Share.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The Amazon Resource Name (ARN) of the storage used for audit logs.
     */
    readonly auditDestinationArn: pulumi.Output<string | undefined>;
    /**
     * The region of the S3 bucket used by the file share. Required when specifying `vpcEndpointDnsName`.
     */
    readonly bucketRegion: pulumi.Output<string | undefined>;
    /**
     * Refresh cache information. see Cache Attributes for more details.
     */
    readonly cacheAttributes: pulumi.Output<outputs.storagegateway.NfsFileShareCacheAttributes | undefined>;
    /**
     * The list of clients that are allowed to access the file gateway. The list must contain either valid IP addresses or valid CIDR blocks. Set to `["0.0.0.0/0"]` to not limit access. Minimum 1 item. Maximum 100 items.
     */
    readonly clientLists: pulumi.Output<string[]>;
    /**
     * The default [storage class](https://docs.aws.amazon.com/storagegateway/latest/APIReference/API_CreateNFSFileShare.html#StorageGateway-CreateNFSFileShare-request-DefaultStorageClass) for objects put into an Amazon S3 bucket by the file gateway. Defaults to `S3_STANDARD`.
     */
    readonly defaultStorageClass: pulumi.Output<string | undefined>;
    /**
     * The name of the file share. Must be set if an S3 prefix name is set in `locationArn`.
     */
    readonly fileShareName: pulumi.Output<string>;
    /**
     * ID of the NFS File Share.
     */
    readonly fileshareId: pulumi.Output<string>;
    /**
     * Amazon Resource Name (ARN) of the file gateway.
     */
    readonly gatewayArn: pulumi.Output<string>;
    /**
     * Boolean value that enables guessing of the MIME type for uploaded objects based on file extensions. Defaults to `true`.
     */
    readonly guessMimeTypeEnabled: pulumi.Output<boolean | undefined>;
    /**
     * Boolean value if `true` to use Amazon S3 server side encryption with your own AWS KMS key, or `false` to use a key managed by Amazon S3. Defaults to `false`.
     */
    readonly kmsEncrypted: pulumi.Output<boolean | undefined>;
    /**
     * Amazon Resource Name (ARN) for KMS key used for Amazon S3 server side encryption. This value can only be set when `kmsEncrypted` is true.
     */
    readonly kmsKeyArn: pulumi.Output<string | undefined>;
    /**
     * The ARN of the backed storage used for storing file data.
     */
    readonly locationArn: pulumi.Output<string>;
    /**
     * Nested argument with file share default values. More information below. see NFS File Share Defaults for more details.
     */
    readonly nfsFileShareDefaults: pulumi.Output<outputs.storagegateway.NfsFileShareNfsFileShareDefaults | undefined>;
    /**
     * The notification policy of the file share. For more information see the [AWS Documentation](https://docs.aws.amazon.com/storagegateway/latest/APIReference/API_CreateNFSFileShare.html#StorageGateway-CreateNFSFileShare-request-NotificationPolicy). Default value is `{}`.
     */
    readonly notificationPolicy: pulumi.Output<string | undefined>;
    /**
     * Access Control List permission for S3 objects. Defaults to `private`.
     */
    readonly objectAcl: pulumi.Output<string | undefined>;
    /**
     * File share path used by the NFS client to identify the mount point.
     */
    readonly path: pulumi.Output<string>;
    /**
     * Boolean to indicate write status of file share. File share does not accept writes if `true`. Defaults to `false`.
     */
    readonly readOnly: pulumi.Output<boolean | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Boolean who pays the cost of the request and the data download from the Amazon S3 bucket. Set this value to `true` if you want the requester to pay instead of the bucket owner. Defaults to `false`.
     */
    readonly requesterPays: pulumi.Output<boolean | undefined>;
    /**
     * The ARN of the AWS Identity and Access Management (IAM) role that a file gateway assumes when it accesses the underlying storage.
     */
    readonly roleArn: pulumi.Output<string>;
    /**
     * Maps a user to anonymous user. Defaults to `RootSquash`. Valid values: `RootSquash` (only root is mapped to anonymous user), `NoSquash` (no one is mapped to anonymous user), `AllSquash` (everyone is mapped to anonymous user)
     */
    readonly squash: pulumi.Output<string | undefined>;
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
     * The DNS name of the VPC endpoint for S3 PrivateLink.
     */
    readonly vpcEndpointDnsName: pulumi.Output<string | undefined>;
    /**
     * Create a NfsFileShare resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: NfsFileShareArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering NfsFileShare resources.
 */
export interface NfsFileShareState {
    /**
     * Amazon Resource Name (ARN) of the NFS File Share.
     */
    arn?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) of the storage used for audit logs.
     */
    auditDestinationArn?: pulumi.Input<string>;
    /**
     * The region of the S3 bucket used by the file share. Required when specifying `vpcEndpointDnsName`.
     */
    bucketRegion?: pulumi.Input<string>;
    /**
     * Refresh cache information. see Cache Attributes for more details.
     */
    cacheAttributes?: pulumi.Input<inputs.storagegateway.NfsFileShareCacheAttributes>;
    /**
     * The list of clients that are allowed to access the file gateway. The list must contain either valid IP addresses or valid CIDR blocks. Set to `["0.0.0.0/0"]` to not limit access. Minimum 1 item. Maximum 100 items.
     */
    clientLists?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The default [storage class](https://docs.aws.amazon.com/storagegateway/latest/APIReference/API_CreateNFSFileShare.html#StorageGateway-CreateNFSFileShare-request-DefaultStorageClass) for objects put into an Amazon S3 bucket by the file gateway. Defaults to `S3_STANDARD`.
     */
    defaultStorageClass?: pulumi.Input<string>;
    /**
     * The name of the file share. Must be set if an S3 prefix name is set in `locationArn`.
     */
    fileShareName?: pulumi.Input<string>;
    /**
     * ID of the NFS File Share.
     */
    fileshareId?: pulumi.Input<string>;
    /**
     * Amazon Resource Name (ARN) of the file gateway.
     */
    gatewayArn?: pulumi.Input<string>;
    /**
     * Boolean value that enables guessing of the MIME type for uploaded objects based on file extensions. Defaults to `true`.
     */
    guessMimeTypeEnabled?: pulumi.Input<boolean>;
    /**
     * Boolean value if `true` to use Amazon S3 server side encryption with your own AWS KMS key, or `false` to use a key managed by Amazon S3. Defaults to `false`.
     */
    kmsEncrypted?: pulumi.Input<boolean>;
    /**
     * Amazon Resource Name (ARN) for KMS key used for Amazon S3 server side encryption. This value can only be set when `kmsEncrypted` is true.
     */
    kmsKeyArn?: pulumi.Input<string>;
    /**
     * The ARN of the backed storage used for storing file data.
     */
    locationArn?: pulumi.Input<string>;
    /**
     * Nested argument with file share default values. More information below. see NFS File Share Defaults for more details.
     */
    nfsFileShareDefaults?: pulumi.Input<inputs.storagegateway.NfsFileShareNfsFileShareDefaults>;
    /**
     * The notification policy of the file share. For more information see the [AWS Documentation](https://docs.aws.amazon.com/storagegateway/latest/APIReference/API_CreateNFSFileShare.html#StorageGateway-CreateNFSFileShare-request-NotificationPolicy). Default value is `{}`.
     */
    notificationPolicy?: pulumi.Input<string>;
    /**
     * Access Control List permission for S3 objects. Defaults to `private`.
     */
    objectAcl?: pulumi.Input<string>;
    /**
     * File share path used by the NFS client to identify the mount point.
     */
    path?: pulumi.Input<string>;
    /**
     * Boolean to indicate write status of file share. File share does not accept writes if `true`. Defaults to `false`.
     */
    readOnly?: pulumi.Input<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Boolean who pays the cost of the request and the data download from the Amazon S3 bucket. Set this value to `true` if you want the requester to pay instead of the bucket owner. Defaults to `false`.
     */
    requesterPays?: pulumi.Input<boolean>;
    /**
     * The ARN of the AWS Identity and Access Management (IAM) role that a file gateway assumes when it accesses the underlying storage.
     */
    roleArn?: pulumi.Input<string>;
    /**
     * Maps a user to anonymous user. Defaults to `RootSquash`. Valid values: `RootSquash` (only root is mapped to anonymous user), `NoSquash` (no one is mapped to anonymous user), `AllSquash` (everyone is mapped to anonymous user)
     */
    squash?: pulumi.Input<string>;
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
     * The DNS name of the VPC endpoint for S3 PrivateLink.
     */
    vpcEndpointDnsName?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a NfsFileShare resource.
 */
export interface NfsFileShareArgs {
    /**
     * The Amazon Resource Name (ARN) of the storage used for audit logs.
     */
    auditDestinationArn?: pulumi.Input<string>;
    /**
     * The region of the S3 bucket used by the file share. Required when specifying `vpcEndpointDnsName`.
     */
    bucketRegion?: pulumi.Input<string>;
    /**
     * Refresh cache information. see Cache Attributes for more details.
     */
    cacheAttributes?: pulumi.Input<inputs.storagegateway.NfsFileShareCacheAttributes>;
    /**
     * The list of clients that are allowed to access the file gateway. The list must contain either valid IP addresses or valid CIDR blocks. Set to `["0.0.0.0/0"]` to not limit access. Minimum 1 item. Maximum 100 items.
     */
    clientLists: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The default [storage class](https://docs.aws.amazon.com/storagegateway/latest/APIReference/API_CreateNFSFileShare.html#StorageGateway-CreateNFSFileShare-request-DefaultStorageClass) for objects put into an Amazon S3 bucket by the file gateway. Defaults to `S3_STANDARD`.
     */
    defaultStorageClass?: pulumi.Input<string>;
    /**
     * The name of the file share. Must be set if an S3 prefix name is set in `locationArn`.
     */
    fileShareName?: pulumi.Input<string>;
    /**
     * Amazon Resource Name (ARN) of the file gateway.
     */
    gatewayArn: pulumi.Input<string>;
    /**
     * Boolean value that enables guessing of the MIME type for uploaded objects based on file extensions. Defaults to `true`.
     */
    guessMimeTypeEnabled?: pulumi.Input<boolean>;
    /**
     * Boolean value if `true` to use Amazon S3 server side encryption with your own AWS KMS key, or `false` to use a key managed by Amazon S3. Defaults to `false`.
     */
    kmsEncrypted?: pulumi.Input<boolean>;
    /**
     * Amazon Resource Name (ARN) for KMS key used for Amazon S3 server side encryption. This value can only be set when `kmsEncrypted` is true.
     */
    kmsKeyArn?: pulumi.Input<string>;
    /**
     * The ARN of the backed storage used for storing file data.
     */
    locationArn: pulumi.Input<string>;
    /**
     * Nested argument with file share default values. More information below. see NFS File Share Defaults for more details.
     */
    nfsFileShareDefaults?: pulumi.Input<inputs.storagegateway.NfsFileShareNfsFileShareDefaults>;
    /**
     * The notification policy of the file share. For more information see the [AWS Documentation](https://docs.aws.amazon.com/storagegateway/latest/APIReference/API_CreateNFSFileShare.html#StorageGateway-CreateNFSFileShare-request-NotificationPolicy). Default value is `{}`.
     */
    notificationPolicy?: pulumi.Input<string>;
    /**
     * Access Control List permission for S3 objects. Defaults to `private`.
     */
    objectAcl?: pulumi.Input<string>;
    /**
     * Boolean to indicate write status of file share. File share does not accept writes if `true`. Defaults to `false`.
     */
    readOnly?: pulumi.Input<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Boolean who pays the cost of the request and the data download from the Amazon S3 bucket. Set this value to `true` if you want the requester to pay instead of the bucket owner. Defaults to `false`.
     */
    requesterPays?: pulumi.Input<boolean>;
    /**
     * The ARN of the AWS Identity and Access Management (IAM) role that a file gateway assumes when it accesses the underlying storage.
     */
    roleArn: pulumi.Input<string>;
    /**
     * Maps a user to anonymous user. Defaults to `RootSquash`. Valid values: `RootSquash` (only root is mapped to anonymous user), `NoSquash` (no one is mapped to anonymous user), `AllSquash` (everyone is mapped to anonymous user)
     */
    squash?: pulumi.Input<string>;
    /**
     * Key-value map of resource tags. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The DNS name of the VPC endpoint for S3 PrivateLink.
     */
    vpcEndpointDnsName?: pulumi.Input<string>;
}
