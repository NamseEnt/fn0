import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages an AWS DataSync EFS Location.
 *
 * > **NOTE:** The EFS File System must have a mounted EFS Mount Target before creating this resource.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.datasync.EfsLocation("example", {
 *     efsFileSystemArn: exampleAwsEfsMountTarget.fileSystemArn,
 *     ec2Config: {
 *         securityGroupArns: [exampleAwsSecurityGroup.arn],
 *         subnetArn: exampleAwsSubnet.arn,
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * ### Identity Schema
 *
 * #### Required
 *
 * - `arn` (String) Amazon Resource Name (ARN) of the DataSync EFS location.
 *
 * Using `pulumi import`, import `aws_datasync_location_efs` using the DataSync Task Amazon Resource Name (ARN). For example:
 *
 * console
 *
 * % pulumi import aws_datasync_location_efs.example arn:aws:datasync:us-east-1:123456789012:location/loc-12345678901234567
 */
export declare class EfsLocation extends pulumi.CustomResource {
    /**
     * Get an existing EfsLocation resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: EfsLocationState, opts?: pulumi.CustomResourceOptions): EfsLocation;
    /**
     * Returns true if the given object is an instance of EfsLocation.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is EfsLocation;
    /**
     * Specifies the Amazon Resource Name (ARN) of the access point that DataSync uses to access the Amazon EFS file system.
     */
    readonly accessPointArn: pulumi.Output<string | undefined>;
    /**
     * Amazon Resource Name (ARN) of the DataSync Location.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Configuration block containing EC2 configurations for connecting to the EFS File System.
     */
    readonly ec2Config: pulumi.Output<outputs.datasync.EfsLocationEc2Config>;
    /**
     * Amazon Resource Name (ARN) of EFS File System.
     */
    readonly efsFileSystemArn: pulumi.Output<string>;
    /**
     * Specifies an Identity and Access Management (IAM) role that DataSync assumes when mounting the Amazon EFS file system.
     */
    readonly fileSystemAccessRoleArn: pulumi.Output<string | undefined>;
    /**
     * Specifies whether you want DataSync to use TLS encryption when transferring data to or from your Amazon EFS file system. Valid values are `NONE` and `TLS1_2`.
     */
    readonly inTransitEncryption: pulumi.Output<string | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Subdirectory to perform actions as source or destination. Default `/`.
     */
    readonly subdirectory: pulumi.Output<string | undefined>;
    /**
     * Key-value pairs of resource tags to assign to the DataSync Location. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
    readonly uri: pulumi.Output<string>;
    /**
     * Create a EfsLocation resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: EfsLocationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering EfsLocation resources.
 */
export interface EfsLocationState {
    /**
     * Specifies the Amazon Resource Name (ARN) of the access point that DataSync uses to access the Amazon EFS file system.
     */
    accessPointArn?: pulumi.Input<string>;
    /**
     * Amazon Resource Name (ARN) of the DataSync Location.
     */
    arn?: pulumi.Input<string>;
    /**
     * Configuration block containing EC2 configurations for connecting to the EFS File System.
     */
    ec2Config?: pulumi.Input<inputs.datasync.EfsLocationEc2Config>;
    /**
     * Amazon Resource Name (ARN) of EFS File System.
     */
    efsFileSystemArn?: pulumi.Input<string>;
    /**
     * Specifies an Identity and Access Management (IAM) role that DataSync assumes when mounting the Amazon EFS file system.
     */
    fileSystemAccessRoleArn?: pulumi.Input<string>;
    /**
     * Specifies whether you want DataSync to use TLS encryption when transferring data to or from your Amazon EFS file system. Valid values are `NONE` and `TLS1_2`.
     */
    inTransitEncryption?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Subdirectory to perform actions as source or destination. Default `/`.
     */
    subdirectory?: pulumi.Input<string>;
    /**
     * Key-value pairs of resource tags to assign to the DataSync Location. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
    uri?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a EfsLocation resource.
 */
export interface EfsLocationArgs {
    /**
     * Specifies the Amazon Resource Name (ARN) of the access point that DataSync uses to access the Amazon EFS file system.
     */
    accessPointArn?: pulumi.Input<string>;
    /**
     * Configuration block containing EC2 configurations for connecting to the EFS File System.
     */
    ec2Config: pulumi.Input<inputs.datasync.EfsLocationEc2Config>;
    /**
     * Amazon Resource Name (ARN) of EFS File System.
     */
    efsFileSystemArn: pulumi.Input<string>;
    /**
     * Specifies an Identity and Access Management (IAM) role that DataSync assumes when mounting the Amazon EFS file system.
     */
    fileSystemAccessRoleArn?: pulumi.Input<string>;
    /**
     * Specifies whether you want DataSync to use TLS encryption when transferring data to or from your Amazon EFS file system. Valid values are `NONE` and `TLS1_2`.
     */
    inTransitEncryption?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Subdirectory to perform actions as source or destination. Default `/`.
     */
    subdirectory?: pulumi.Input<string>;
    /**
     * Key-value pairs of resource tags to assign to the DataSync Location. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
