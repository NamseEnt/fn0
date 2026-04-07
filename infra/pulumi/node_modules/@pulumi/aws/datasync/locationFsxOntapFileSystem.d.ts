import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS DataSync Location FSx Ontap File System.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = new aws.datasync.LocationFsxOntapFileSystem("test", {
 *     fsxFilesystemArn: testAwsFsxOntapFileSystem.arn,
 *     securityGroupArns: [testAwsSecurityGroup.arn],
 *     storageVirtualMachineArn: testAwsFsxOntapStorageVirtualMachine.arn,
 *     protocol: {
 *         nfs: {
 *             mountOptions: {
 *                 version: "NFS3",
 *             },
 *         },
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_datasync_location_fsx_ontap_file_system` using the `DataSync-ARN#FSx-ontap-svm-ARN`. For example:
 *
 * ```sh
 * $ pulumi import aws:datasync/locationFsxOntapFileSystem:LocationFsxOntapFileSystem example arn:aws:datasync:us-west-2:123456789012:location/loc-12345678901234567#arn:aws:fsx:us-west-2:123456789012:storage-virtual-machine/svm-12345678abcdef123
 * ```
 */
export declare class LocationFsxOntapFileSystem extends pulumi.CustomResource {
    /**
     * Get an existing LocationFsxOntapFileSystem resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: LocationFsxOntapFileSystemState, opts?: pulumi.CustomResourceOptions): LocationFsxOntapFileSystem;
    /**
     * Returns true if the given object is an instance of LocationFsxOntapFileSystem.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is LocationFsxOntapFileSystem;
    /**
     * ARN of the DataSync Location for the FSx Ontap File System.
     */
    readonly arn: pulumi.Output<string>;
    readonly creationTime: pulumi.Output<string>;
    /**
     * ARN of the FSx Ontap File System.
     */
    readonly fsxFilesystemArn: pulumi.Output<string>;
    /**
     * The data transfer protocol that DataSync uses to access your Amazon FSx file system. See Protocol below.
     */
    readonly protocol: pulumi.Output<outputs.datasync.LocationFsxOntapFileSystemProtocol>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The security groups that provide access to your file system's preferred subnet. The security groups must allow outbbound traffic on the following ports (depending on the protocol you use):
     * * Network File System (NFS): TCP ports 111, 635, and 2049
     * * Server Message Block (SMB): TCP port 445
     */
    readonly securityGroupArns: pulumi.Output<string[]>;
    /**
     * The ARN of the SVM in your file system where you want to copy data to of from.
     *
     * The following arguments are optional:
     */
    readonly storageVirtualMachineArn: pulumi.Output<string>;
    /**
     * Path to the file share in the SVM where you'll copy your data. You can specify a junction path (also known as a mount point), qtree path (for NFS file shares), or share name (for SMB file shares) (e.g. `/vol1`, `/vol1/tree1`, `share1`).
     */
    readonly subdirectory: pulumi.Output<string>;
    /**
     * Key-value pairs of resource tags to assign to the DataSync Location. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * URI of the FSx ONTAP file system location
     */
    readonly uri: pulumi.Output<string>;
    /**
     * Create a LocationFsxOntapFileSystem resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: LocationFsxOntapFileSystemArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering LocationFsxOntapFileSystem resources.
 */
export interface LocationFsxOntapFileSystemState {
    /**
     * ARN of the DataSync Location for the FSx Ontap File System.
     */
    arn?: pulumi.Input<string>;
    creationTime?: pulumi.Input<string>;
    /**
     * ARN of the FSx Ontap File System.
     */
    fsxFilesystemArn?: pulumi.Input<string>;
    /**
     * The data transfer protocol that DataSync uses to access your Amazon FSx file system. See Protocol below.
     */
    protocol?: pulumi.Input<inputs.datasync.LocationFsxOntapFileSystemProtocol>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The security groups that provide access to your file system's preferred subnet. The security groups must allow outbbound traffic on the following ports (depending on the protocol you use):
     * * Network File System (NFS): TCP ports 111, 635, and 2049
     * * Server Message Block (SMB): TCP port 445
     */
    securityGroupArns?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The ARN of the SVM in your file system where you want to copy data to of from.
     *
     * The following arguments are optional:
     */
    storageVirtualMachineArn?: pulumi.Input<string>;
    /**
     * Path to the file share in the SVM where you'll copy your data. You can specify a junction path (also known as a mount point), qtree path (for NFS file shares), or share name (for SMB file shares) (e.g. `/vol1`, `/vol1/tree1`, `share1`).
     */
    subdirectory?: pulumi.Input<string>;
    /**
     * Key-value pairs of resource tags to assign to the DataSync Location. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * URI of the FSx ONTAP file system location
     */
    uri?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a LocationFsxOntapFileSystem resource.
 */
export interface LocationFsxOntapFileSystemArgs {
    /**
     * The data transfer protocol that DataSync uses to access your Amazon FSx file system. See Protocol below.
     */
    protocol: pulumi.Input<inputs.datasync.LocationFsxOntapFileSystemProtocol>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The security groups that provide access to your file system's preferred subnet. The security groups must allow outbbound traffic on the following ports (depending on the protocol you use):
     * * Network File System (NFS): TCP ports 111, 635, and 2049
     * * Server Message Block (SMB): TCP port 445
     */
    securityGroupArns: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The ARN of the SVM in your file system where you want to copy data to of from.
     *
     * The following arguments are optional:
     */
    storageVirtualMachineArn: pulumi.Input<string>;
    /**
     * Path to the file share in the SVM where you'll copy your data. You can specify a junction path (also known as a mount point), qtree path (for NFS file shares), or share name (for SMB file shares) (e.g. `/vol1`, `/vol1/tree1`, `share1`).
     */
    subdirectory?: pulumi.Input<string>;
    /**
     * Key-value pairs of resource tags to assign to the DataSync Location. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
