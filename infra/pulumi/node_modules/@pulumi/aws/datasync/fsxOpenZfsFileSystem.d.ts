import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages an AWS DataSync FSx OpenZfs Location.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.datasync.FsxOpenZfsFileSystem("example", {
 *     fsxFilesystemArn: exampleAwsFsxOpenzfsFileSystem.arn,
 *     securityGroupArns: [exampleAwsSecurityGroup.arn],
 *     protocol: {
 *         nfs: {
 *             mountOptions: {
 *                 version: "AUTOMATIC",
 *             },
 *         },
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_datasync_location_fsx_openzfs_file_system` using the `DataSync-ARN#FSx-openzfs-ARN`. For example:
 *
 * ```sh
 * $ pulumi import aws:datasync/fsxOpenZfsFileSystem:FsxOpenZfsFileSystem example arn:aws:datasync:us-west-2:123456789012:location/loc-12345678901234567#arn:aws:fsx:us-west-2:123456789012:file-system/fs-08e04cd442c1bb94a
 * ```
 */
export declare class FsxOpenZfsFileSystem extends pulumi.CustomResource {
    /**
     * Get an existing FsxOpenZfsFileSystem resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: FsxOpenZfsFileSystemState, opts?: pulumi.CustomResourceOptions): FsxOpenZfsFileSystem;
    /**
     * Returns true if the given object is an instance of FsxOpenZfsFileSystem.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is FsxOpenZfsFileSystem;
    /**
     * Amazon Resource Name (ARN) of the DataSync Location.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The time that the FSx for openzfs location was created.
     */
    readonly creationTime: pulumi.Output<string>;
    /**
     * The Amazon Resource Name (ARN) for the FSx for OpenZfs file system.
     */
    readonly fsxFilesystemArn: pulumi.Output<string>;
    /**
     * The type of protocol that DataSync uses to access your file system. See below.
     */
    readonly protocol: pulumi.Output<outputs.datasync.FsxOpenZfsFileSystemProtocol>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The Amazon Resource Names (ARNs) of the security groups that are to use to configure the FSx for openzfs file system.
     */
    readonly securityGroupArns: pulumi.Output<string[]>;
    /**
     * Subdirectory to perform actions as source or destination. Must start with `/fsx`.
     */
    readonly subdirectory: pulumi.Output<string>;
    /**
     * Key-value pairs of resource tags to assign to the DataSync Location. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * The URL of the FSx for openzfs location that was described.
     */
    readonly uri: pulumi.Output<string>;
    /**
     * Create a FsxOpenZfsFileSystem resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: FsxOpenZfsFileSystemArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering FsxOpenZfsFileSystem resources.
 */
export interface FsxOpenZfsFileSystemState {
    /**
     * Amazon Resource Name (ARN) of the DataSync Location.
     */
    arn?: pulumi.Input<string>;
    /**
     * The time that the FSx for openzfs location was created.
     */
    creationTime?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) for the FSx for OpenZfs file system.
     */
    fsxFilesystemArn?: pulumi.Input<string>;
    /**
     * The type of protocol that DataSync uses to access your file system. See below.
     */
    protocol?: pulumi.Input<inputs.datasync.FsxOpenZfsFileSystemProtocol>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The Amazon Resource Names (ARNs) of the security groups that are to use to configure the FSx for openzfs file system.
     */
    securityGroupArns?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Subdirectory to perform actions as source or destination. Must start with `/fsx`.
     */
    subdirectory?: pulumi.Input<string>;
    /**
     * Key-value pairs of resource tags to assign to the DataSync Location. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * The URL of the FSx for openzfs location that was described.
     */
    uri?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a FsxOpenZfsFileSystem resource.
 */
export interface FsxOpenZfsFileSystemArgs {
    /**
     * The Amazon Resource Name (ARN) for the FSx for OpenZfs file system.
     */
    fsxFilesystemArn: pulumi.Input<string>;
    /**
     * The type of protocol that DataSync uses to access your file system. See below.
     */
    protocol: pulumi.Input<inputs.datasync.FsxOpenZfsFileSystemProtocol>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The Amazon Resource Names (ARNs) of the security groups that are to use to configure the FSx for openzfs file system.
     */
    securityGroupArns: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Subdirectory to perform actions as source or destination. Must start with `/fsx`.
     */
    subdirectory?: pulumi.Input<string>;
    /**
     * Key-value pairs of resource tags to assign to the DataSync Location. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
