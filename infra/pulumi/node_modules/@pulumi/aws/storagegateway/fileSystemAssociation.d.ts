import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Associate an Amazon FSx file system with the FSx File Gateway. After the association process is complete, the file shares on the Amazon FSx file system are available for access through the gateway. This operation only supports the FSx File Gateway type.
 *
 * [FSx File Gateway requirements](https://docs.aws.amazon.com/filegateway/latest/filefsxw/Requirements.html).
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.storagegateway.FileSystemAssociation("example", {
 *     gatewayArn: exampleAwsStoragegatewayGateway.arn,
 *     locationArn: exampleAwsFsxWindowsFileSystem.arn,
 *     username: "Admin",
 *     password: "avoid-plaintext-passwords",
 *     auditDestinationArn: exampleAwsS3Bucket.arn,
 * });
 * ```
 *
 * ## Required Services Example
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const awsServiceStoragegatewayAmiFILES3Latest = aws.ssm.getParameter({
 *     name: "/aws/service/storagegateway/ami/FILE_S3/latest",
 * });
 * const test = new aws.ec2.Instance("test", {
 *     ami: awsServiceStoragegatewayAmiFILES3Latest.then(awsServiceStoragegatewayAmiFILES3Latest => awsServiceStoragegatewayAmiFILES3Latest.value),
 *     associatePublicIpAddress: true,
 *     instanceType: aws.ec2.InstanceType[available.instanceType],
 *     vpcSecurityGroupIds: [testAwsSecurityGroup.id],
 *     subnetId: testAwsSubnet[0].id,
 * }, {
 *     dependsOn: [
 *         testAwsRoute,
 *         testAwsVpcDhcpOptionsAssociation,
 *     ],
 * });
 * const testGateway = new aws.storagegateway.Gateway("test", {
 *     gatewayIpAddress: test.publicIp,
 *     gatewayName: "test-sgw",
 *     gatewayTimezone: "GMT",
 *     gatewayType: "FILE_FSX_SMB",
 *     smbActiveDirectorySettings: {
 *         domainName: testAwsDirectoryServiceDirectory.name,
 *         password: testAwsDirectoryServiceDirectory.password,
 *         username: "Admin",
 *     },
 * });
 * const testWindowsFileSystem = new aws.fsx.WindowsFileSystem("test", {
 *     activeDirectoryId: testAwsDirectoryServiceDirectory.id,
 *     securityGroupIds: [testAwsSecurityGroup.id],
 *     skipFinalBackup: true,
 *     storageCapacity: 32,
 *     subnetIds: [testAwsSubnet[0].id],
 *     throughputCapacity: 8,
 * });
 * const fsx = new aws.storagegateway.FileSystemAssociation("fsx", {
 *     gatewayArn: testGateway.arn,
 *     locationArn: testWindowsFileSystem.arn,
 *     username: "Admin",
 *     password: testAwsDirectoryServiceDirectory.password,
 *     cacheAttributes: {
 *         cacheStaleTimeoutInSeconds: 400,
 *     },
 *     auditDestinationArn: testAwsCloudwatchLogGroup.arn,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_storagegateway_file_system_association` using the FSx file system association Amazon Resource Name (ARN). For example:
 *
 * ```sh
 * $ pulumi import aws:storagegateway/fileSystemAssociation:FileSystemAssociation example arn:aws:storagegateway:us-east-1:123456789012:fs-association/fsa-0DA347732FDB40125
 * ```
 */
export declare class FileSystemAssociation extends pulumi.CustomResource {
    /**
     * Get an existing FileSystemAssociation resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: FileSystemAssociationState, opts?: pulumi.CustomResourceOptions): FileSystemAssociation;
    /**
     * Returns true if the given object is an instance of FileSystemAssociation.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is FileSystemAssociation;
    /**
     * Amazon Resource Name (ARN) of the newly created file system association.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The Amazon Resource Name (ARN) of the storage used for the audit logs.
     */
    readonly auditDestinationArn: pulumi.Output<string | undefined>;
    /**
     * Refresh cache information. see Cache Attributes for more details.
     */
    readonly cacheAttributes: pulumi.Output<outputs.storagegateway.FileSystemAssociationCacheAttributes | undefined>;
    /**
     * The Amazon Resource Name (ARN) of the gateway.
     */
    readonly gatewayArn: pulumi.Output<string>;
    /**
     * The Amazon Resource Name (ARN) of the Amazon FSx file system to associate with the FSx File Gateway.
     */
    readonly locationArn: pulumi.Output<string>;
    /**
     * The password of the user credential.
     */
    readonly password: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * The user name of the user credential that has permission to access the root share of the Amazon FSx file system. The user account must belong to the Amazon FSx delegated admin user group.
     */
    readonly username: pulumi.Output<string>;
    /**
     * Create a FileSystemAssociation resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: FileSystemAssociationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering FileSystemAssociation resources.
 */
export interface FileSystemAssociationState {
    /**
     * Amazon Resource Name (ARN) of the newly created file system association.
     */
    arn?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) of the storage used for the audit logs.
     */
    auditDestinationArn?: pulumi.Input<string>;
    /**
     * Refresh cache information. see Cache Attributes for more details.
     */
    cacheAttributes?: pulumi.Input<inputs.storagegateway.FileSystemAssociationCacheAttributes>;
    /**
     * The Amazon Resource Name (ARN) of the gateway.
     */
    gatewayArn?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) of the Amazon FSx file system to associate with the FSx File Gateway.
     */
    locationArn?: pulumi.Input<string>;
    /**
     * The password of the user credential.
     */
    password?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * The user name of the user credential that has permission to access the root share of the Amazon FSx file system. The user account must belong to the Amazon FSx delegated admin user group.
     */
    username?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a FileSystemAssociation resource.
 */
export interface FileSystemAssociationArgs {
    /**
     * The Amazon Resource Name (ARN) of the storage used for the audit logs.
     */
    auditDestinationArn?: pulumi.Input<string>;
    /**
     * Refresh cache information. see Cache Attributes for more details.
     */
    cacheAttributes?: pulumi.Input<inputs.storagegateway.FileSystemAssociationCacheAttributes>;
    /**
     * The Amazon Resource Name (ARN) of the gateway.
     */
    gatewayArn: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) of the Amazon FSx file system to associate with the FSx File Gateway.
     */
    locationArn: pulumi.Input<string>;
    /**
     * The password of the user credential.
     */
    password: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The user name of the user credential that has permission to access the root share of the Amazon FSx file system. The user account must belong to the Amazon FSx delegated admin user group.
     */
    username: pulumi.Input<string>;
}
