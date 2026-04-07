import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages an AWS Storage Gateway SMB File Share.
 *
 * ## Example Usage
 *
 * ### Active Directory Authentication
 *
 * > **NOTE:** The gateway must have already joined the Active Directory domain prior to SMB file share creationE.g., via "SMB Settings" in the AWS Storage Gateway console or `smbActiveDirectorySettings` in the `aws.storagegateway.Gateway` resource.
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.storagegateway.SmbFileShare("example", {
 *     authentication: "ActiveDirectory",
 *     gatewayArn: exampleAwsStoragegatewayGateway.arn,
 *     locationArn: exampleAwsS3Bucket.arn,
 *     roleArn: exampleAwsIamRole.arn,
 * });
 * ```
 *
 * ### Guest Authentication
 *
 * > **NOTE:** The gateway must have already had the SMB guest password set prior to SMB file share creationE.g., via "SMB Settings" in the AWS Storage Gateway console or `smbGuestPassword` in the `aws.storagegateway.Gateway` resource.
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.storagegateway.SmbFileShare("example", {
 *     authentication: "GuestAccess",
 *     gatewayArn: exampleAwsStoragegatewayGateway.arn,
 *     locationArn: exampleAwsS3Bucket.arn,
 *     roleArn: exampleAwsIamRole.arn,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_storagegateway_smb_file_share` using the SMB File Share Amazon Resource Name (ARN). For example:
 *
 * ```sh
 * $ pulumi import aws:storagegateway/smbFileShare:SmbFileShare example arn:aws:storagegateway:us-east-1:123456789012:share/share-12345678
 * ```
 */
export declare class SmbFileShare extends pulumi.CustomResource {
    /**
     * Get an existing SmbFileShare resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: SmbFileShareState, opts?: pulumi.CustomResourceOptions): SmbFileShare;
    /**
     * Returns true if the given object is an instance of SmbFileShare.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is SmbFileShare;
    /**
     * The files and folders on this share will only be visible to users with read access. Default value is `false`.
     */
    readonly accessBasedEnumeration: pulumi.Output<boolean | undefined>;
    /**
     * A list of users in the Active Directory that have admin access to the file share. Only valid if `authentication` is set to `ActiveDirectory`.
     */
    readonly adminUserLists: pulumi.Output<string[] | undefined>;
    /**
     * Amazon Resource Name (ARN) of the SMB File Share.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The Amazon Resource Name (ARN) of the CloudWatch Log Group used for the audit logs.
     */
    readonly auditDestinationArn: pulumi.Output<string | undefined>;
    /**
     * The authentication method that users use to access the file share. Defaults to `ActiveDirectory`. Valid values: `ActiveDirectory`, `GuestAccess`.
     */
    readonly authentication: pulumi.Output<string | undefined>;
    /**
     * The region of the S3 buck used by the file share. Required when specifying a `vpcEndpointDnsName`.
     */
    readonly bucketRegion: pulumi.Output<string | undefined>;
    /**
     * Refresh cache information. see `cacheAttributes` Block for more details.
     */
    readonly cacheAttributes: pulumi.Output<outputs.storagegateway.SmbFileShareCacheAttributes | undefined>;
    /**
     * The case of an object name in an Amazon S3 bucket. For `ClientSpecified`, the client determines the case sensitivity. For `CaseSensitive`, the gateway determines the case sensitivity. The default value is `ClientSpecified`.
     */
    readonly caseSensitivity: pulumi.Output<string | undefined>;
    /**
     * The default [storage class](https://docs.aws.amazon.com/storagegateway/latest/APIReference/API_CreateNFSFileShare.html#StorageGateway-CreateNFSFileShare-request-DefaultStorageClass) for objects put into an Amazon S3 bucket by the file gateway. Defaults to `S3_STANDARD`.
     */
    readonly defaultStorageClass: pulumi.Output<string | undefined>;
    /**
     * The name of the file share. Must be set if an S3 prefix name is set in `locationArn`.
     */
    readonly fileShareName: pulumi.Output<string>;
    /**
     * ID of the SMB File Share.
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
     * A list of users in the Active Directory that are not allowed to access the file share. Only valid if `authentication` is set to `ActiveDirectory`.
     */
    readonly invalidUserLists: pulumi.Output<string[] | undefined>;
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
     * The notification policy of the file share. For more information see the [AWS Documentation](https://docs.aws.amazon.com/storagegateway/latest/APIReference/API_CreateNFSFileShare.html#StorageGateway-CreateNFSFileShare-request-NotificationPolicy). Default value is `{}`.
     */
    readonly notificationPolicy: pulumi.Output<string | undefined>;
    /**
     * Access Control List permission for S3 objects. Defaults to `private`.
     */
    readonly objectAcl: pulumi.Output<string | undefined>;
    /**
     * Boolean to indicate Opportunistic lock (oplock) status. Defaults to `true`.
     */
    readonly oplocksEnabled: pulumi.Output<boolean>;
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
     * Set this value to `true` to enable ACL (access control list) on the SMB fileshare. Set it to `false` to map file and directory permissions to the POSIX permissions. This setting applies only to `ActiveDirectory` authentication type.
     */
    readonly smbAclEnabled: pulumi.Output<boolean | undefined>;
    /**
     * Key-value map of resource tags. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     *
     * **Note:** If you have previously included a `cacheAttributes` block in your configuration, removing it will not reset the refresh cache value and the previous value will remain. You must explicitly set a new value to change it.
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
     * A list of users in the Active Directory that are allowed to access the file share. If you need to specify an Active directory group, add '@' before the name of the group. It will be set on Allowed group in AWS console. Only valid if `authentication` is set to `ActiveDirectory`.
     */
    readonly validUserLists: pulumi.Output<string[] | undefined>;
    /**
     * The DNS name of the VPC endpoint for S3 private link.
     */
    readonly vpcEndpointDnsName: pulumi.Output<string | undefined>;
    /**
     * Create a SmbFileShare resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: SmbFileShareArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering SmbFileShare resources.
 */
export interface SmbFileShareState {
    /**
     * The files and folders on this share will only be visible to users with read access. Default value is `false`.
     */
    accessBasedEnumeration?: pulumi.Input<boolean>;
    /**
     * A list of users in the Active Directory that have admin access to the file share. Only valid if `authentication` is set to `ActiveDirectory`.
     */
    adminUserLists?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Amazon Resource Name (ARN) of the SMB File Share.
     */
    arn?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) of the CloudWatch Log Group used for the audit logs.
     */
    auditDestinationArn?: pulumi.Input<string>;
    /**
     * The authentication method that users use to access the file share. Defaults to `ActiveDirectory`. Valid values: `ActiveDirectory`, `GuestAccess`.
     */
    authentication?: pulumi.Input<string>;
    /**
     * The region of the S3 buck used by the file share. Required when specifying a `vpcEndpointDnsName`.
     */
    bucketRegion?: pulumi.Input<string>;
    /**
     * Refresh cache information. see `cacheAttributes` Block for more details.
     */
    cacheAttributes?: pulumi.Input<inputs.storagegateway.SmbFileShareCacheAttributes>;
    /**
     * The case of an object name in an Amazon S3 bucket. For `ClientSpecified`, the client determines the case sensitivity. For `CaseSensitive`, the gateway determines the case sensitivity. The default value is `ClientSpecified`.
     */
    caseSensitivity?: pulumi.Input<string>;
    /**
     * The default [storage class](https://docs.aws.amazon.com/storagegateway/latest/APIReference/API_CreateNFSFileShare.html#StorageGateway-CreateNFSFileShare-request-DefaultStorageClass) for objects put into an Amazon S3 bucket by the file gateway. Defaults to `S3_STANDARD`.
     */
    defaultStorageClass?: pulumi.Input<string>;
    /**
     * The name of the file share. Must be set if an S3 prefix name is set in `locationArn`.
     */
    fileShareName?: pulumi.Input<string>;
    /**
     * ID of the SMB File Share.
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
     * A list of users in the Active Directory that are not allowed to access the file share. Only valid if `authentication` is set to `ActiveDirectory`.
     */
    invalidUserLists?: pulumi.Input<pulumi.Input<string>[]>;
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
     * The notification policy of the file share. For more information see the [AWS Documentation](https://docs.aws.amazon.com/storagegateway/latest/APIReference/API_CreateNFSFileShare.html#StorageGateway-CreateNFSFileShare-request-NotificationPolicy). Default value is `{}`.
     */
    notificationPolicy?: pulumi.Input<string>;
    /**
     * Access Control List permission for S3 objects. Defaults to `private`.
     */
    objectAcl?: pulumi.Input<string>;
    /**
     * Boolean to indicate Opportunistic lock (oplock) status. Defaults to `true`.
     */
    oplocksEnabled?: pulumi.Input<boolean>;
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
     * Set this value to `true` to enable ACL (access control list) on the SMB fileshare. Set it to `false` to map file and directory permissions to the POSIX permissions. This setting applies only to `ActiveDirectory` authentication type.
     */
    smbAclEnabled?: pulumi.Input<boolean>;
    /**
     * Key-value map of resource tags. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     *
     * **Note:** If you have previously included a `cacheAttributes` block in your configuration, removing it will not reset the refresh cache value and the previous value will remain. You must explicitly set a new value to change it.
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
     * A list of users in the Active Directory that are allowed to access the file share. If you need to specify an Active directory group, add '@' before the name of the group. It will be set on Allowed group in AWS console. Only valid if `authentication` is set to `ActiveDirectory`.
     */
    validUserLists?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The DNS name of the VPC endpoint for S3 private link.
     */
    vpcEndpointDnsName?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a SmbFileShare resource.
 */
export interface SmbFileShareArgs {
    /**
     * The files and folders on this share will only be visible to users with read access. Default value is `false`.
     */
    accessBasedEnumeration?: pulumi.Input<boolean>;
    /**
     * A list of users in the Active Directory that have admin access to the file share. Only valid if `authentication` is set to `ActiveDirectory`.
     */
    adminUserLists?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The Amazon Resource Name (ARN) of the CloudWatch Log Group used for the audit logs.
     */
    auditDestinationArn?: pulumi.Input<string>;
    /**
     * The authentication method that users use to access the file share. Defaults to `ActiveDirectory`. Valid values: `ActiveDirectory`, `GuestAccess`.
     */
    authentication?: pulumi.Input<string>;
    /**
     * The region of the S3 buck used by the file share. Required when specifying a `vpcEndpointDnsName`.
     */
    bucketRegion?: pulumi.Input<string>;
    /**
     * Refresh cache information. see `cacheAttributes` Block for more details.
     */
    cacheAttributes?: pulumi.Input<inputs.storagegateway.SmbFileShareCacheAttributes>;
    /**
     * The case of an object name in an Amazon S3 bucket. For `ClientSpecified`, the client determines the case sensitivity. For `CaseSensitive`, the gateway determines the case sensitivity. The default value is `ClientSpecified`.
     */
    caseSensitivity?: pulumi.Input<string>;
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
     * A list of users in the Active Directory that are not allowed to access the file share. Only valid if `authentication` is set to `ActiveDirectory`.
     */
    invalidUserLists?: pulumi.Input<pulumi.Input<string>[]>;
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
     * The notification policy of the file share. For more information see the [AWS Documentation](https://docs.aws.amazon.com/storagegateway/latest/APIReference/API_CreateNFSFileShare.html#StorageGateway-CreateNFSFileShare-request-NotificationPolicy). Default value is `{}`.
     */
    notificationPolicy?: pulumi.Input<string>;
    /**
     * Access Control List permission for S3 objects. Defaults to `private`.
     */
    objectAcl?: pulumi.Input<string>;
    /**
     * Boolean to indicate Opportunistic lock (oplock) status. Defaults to `true`.
     */
    oplocksEnabled?: pulumi.Input<boolean>;
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
     * Set this value to `true` to enable ACL (access control list) on the SMB fileshare. Set it to `false` to map file and directory permissions to the POSIX permissions. This setting applies only to `ActiveDirectory` authentication type.
     */
    smbAclEnabled?: pulumi.Input<boolean>;
    /**
     * Key-value map of resource tags. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     *
     * **Note:** If you have previously included a `cacheAttributes` block in your configuration, removing it will not reset the refresh cache value and the previous value will remain. You must explicitly set a new value to change it.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * A list of users in the Active Directory that are allowed to access the file share. If you need to specify an Active directory group, add '@' before the name of the group. It will be set on Allowed group in AWS console. Only valid if `authentication` is set to `ActiveDirectory`.
     */
    validUserLists?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The DNS name of the VPC endpoint for S3 private link.
     */
    vpcEndpointDnsName?: pulumi.Input<string>;
}
