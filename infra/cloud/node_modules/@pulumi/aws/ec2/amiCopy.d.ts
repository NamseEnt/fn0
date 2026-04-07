import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * The "AMI copy" resource allows duplication of an Amazon Machine Image (AMI),
 * including cross-region copies.
 *
 * If the source AMI has associated EBS snapshots, those will also be duplicated
 * along with the AMI.
 *
 * This is useful for taking a single AMI provisioned in one region and making
 * it available in another for a multi-region deployment.
 *
 * Copying an AMI can take several minutes. The creation of this resource will
 * block until the new AMI is available for use on new instances.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.ec2.AmiCopy("example", {
 *     name: "example",
 *     sourceAmiId: "ami-xxxxxxxx",
 *     sourceAmiRegion: "us-west-1",
 *     tags: {
 *         Name: "HelloWorld",
 *     },
 * });
 * ```
 */
export declare class AmiCopy extends pulumi.CustomResource {
    /**
     * Get an existing AmiCopy resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: AmiCopyState, opts?: pulumi.CustomResourceOptions): AmiCopy;
    /**
     * Returns true if the given object is an instance of AmiCopy.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is AmiCopy;
    /**
     * Machine architecture for created instances. Defaults to `x8664`.
     */
    readonly architecture: pulumi.Output<string>;
    /**
     * ARN of the AMI.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Boot mode of the AMI. For more information, see [Boot modes](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/ami-boot.html) in the Amazon Elastic Compute Cloud User Guide.
     */
    readonly bootMode: pulumi.Output<string>;
    /**
     * Date and time to deprecate the AMI. If you specified a value for seconds, Amazon EC2 rounds the seconds to the nearest minute. Valid values: [RFC3339 time string](https://tools.ietf.org/html/rfc3339#section-5.8) (`YYYY-MM-DDTHH:MM:SSZ`)
     */
    readonly deprecationTime: pulumi.Output<string | undefined>;
    /**
     * Longer, human-readable description for the AMI.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * ARN of the Outpost to which to copy the AMI.
     * Only specify this parameter when copying an AMI from an AWS Region to an Outpost. The AMI must be in the Region of the destination Outpost.
     */
    readonly destinationOutpostArn: pulumi.Output<string | undefined>;
    /**
     * Nested block describing an EBS block device that should be
     * attached to created instances. The structure of this block is described below.
     */
    readonly ebsBlockDevices: pulumi.Output<outputs.ec2.AmiCopyEbsBlockDevice[]>;
    /**
     * Whether enhanced networking with ENA is enabled. Defaults to `false`.
     */
    readonly enaSupport: pulumi.Output<boolean>;
    /**
     * Whether the destination snapshots of the copied image should be encrypted. Defaults to `false`
     */
    readonly encrypted: pulumi.Output<boolean | undefined>;
    /**
     * Nested block describing an ephemeral block device that
     * should be attached to created instances. The structure of this block is described below.
     */
    readonly ephemeralBlockDevices: pulumi.Output<outputs.ec2.AmiCopyEphemeralBlockDevice[]>;
    readonly hypervisor: pulumi.Output<string>;
    /**
     * Path to an S3 object containing an image manifest, e.g., created
     * by the `ec2-upload-bundle` command in the EC2 command line tools.
     */
    readonly imageLocation: pulumi.Output<string>;
    readonly imageOwnerAlias: pulumi.Output<string>;
    readonly imageType: pulumi.Output<string>;
    /**
     * If EC2 instances started from this image should require the use of the Instance Metadata Service V2 (IMDSv2), set this argument to `v2.0`. For more information, see [Configure instance metadata options for new instances](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/configuring-IMDS-new-instances.html#configure-IMDS-new-instances-ami-configuration).
     */
    readonly imdsSupport: pulumi.Output<string>;
    /**
     * ID of the kernel image (AKI) that will be used as the paravirtual
     * kernel in created instances.
     */
    readonly kernelId: pulumi.Output<string>;
    /**
     * Full ARN of the KMS Key to use when encrypting the snapshots of an image during a copy operation. If not specified, then the default AWS KMS Key will be used
     */
    readonly kmsKeyId: pulumi.Output<string>;
    /**
     * Date and time, in ISO 8601 date-time format , when the AMI was last used to launch an EC2 instance. When the AMI is used to launch an instance, there is a 24-hour delay before that usage is reported. For more information, see the following [AWS document](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/ami-last-launched-time.html).
     */
    readonly lastLaunchedTime: pulumi.Output<string>;
    readonly manageEbsSnapshots: pulumi.Output<boolean>;
    /**
     * Region-unique name for the AMI.
     */
    readonly name: pulumi.Output<string>;
    readonly ownerId: pulumi.Output<string>;
    readonly platform: pulumi.Output<string>;
    readonly platformDetails: pulumi.Output<string>;
    readonly public: pulumi.Output<boolean>;
    /**
     * ID of an initrd image (ARI) that will be used when booting the
     * created instances.
     */
    readonly ramdiskId: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Name of the root device (for example, `/dev/sda1`, or `/dev/xvda`).
     */
    readonly rootDeviceName: pulumi.Output<string>;
    readonly rootSnapshotId: pulumi.Output<string>;
    /**
     * Id of the AMI to copy. This id must be valid in the region
     * given by `sourceAmiRegion`.
     */
    readonly sourceAmiId: pulumi.Output<string>;
    /**
     * Region from which the AMI will be copied. This may be the
     * same as the AWS provider region in order to create a copy within the same region.
     */
    readonly sourceAmiRegion: pulumi.Output<string>;
    /**
     * When set to "simple" (the default), enables enhanced networking
     * for created instances. No other value is supported at this time.
     */
    readonly sriovNetSupport: pulumi.Output<string>;
    /**
     * Map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * If the image is configured for NitroTPM support, the value is `v2.0`. For more information, see [NitroTPM](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/nitrotpm.html) in the Amazon Elastic Compute Cloud User Guide.
     */
    readonly tpmSupport: pulumi.Output<string>;
    /**
     * Base64 representation of the non-volatile UEFI variable store.
     */
    readonly uefiData: pulumi.Output<string>;
    readonly usageOperation: pulumi.Output<string>;
    /**
     * Keyword to choose what virtualization mode created instances
     * will use. Can be either "paravirtual" (the default) or "hvm". The choice of virtualization type
     * changes the set of further arguments that are required, as described below.
     */
    readonly virtualizationType: pulumi.Output<string>;
    /**
     * Create a AmiCopy resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: AmiCopyArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering AmiCopy resources.
 */
export interface AmiCopyState {
    /**
     * Machine architecture for created instances. Defaults to `x8664`.
     */
    architecture?: pulumi.Input<string>;
    /**
     * ARN of the AMI.
     */
    arn?: pulumi.Input<string>;
    /**
     * Boot mode of the AMI. For more information, see [Boot modes](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/ami-boot.html) in the Amazon Elastic Compute Cloud User Guide.
     */
    bootMode?: pulumi.Input<string>;
    /**
     * Date and time to deprecate the AMI. If you specified a value for seconds, Amazon EC2 rounds the seconds to the nearest minute. Valid values: [RFC3339 time string](https://tools.ietf.org/html/rfc3339#section-5.8) (`YYYY-MM-DDTHH:MM:SSZ`)
     */
    deprecationTime?: pulumi.Input<string>;
    /**
     * Longer, human-readable description for the AMI.
     */
    description?: pulumi.Input<string>;
    /**
     * ARN of the Outpost to which to copy the AMI.
     * Only specify this parameter when copying an AMI from an AWS Region to an Outpost. The AMI must be in the Region of the destination Outpost.
     */
    destinationOutpostArn?: pulumi.Input<string>;
    /**
     * Nested block describing an EBS block device that should be
     * attached to created instances. The structure of this block is described below.
     */
    ebsBlockDevices?: pulumi.Input<pulumi.Input<inputs.ec2.AmiCopyEbsBlockDevice>[]>;
    /**
     * Whether enhanced networking with ENA is enabled. Defaults to `false`.
     */
    enaSupport?: pulumi.Input<boolean>;
    /**
     * Whether the destination snapshots of the copied image should be encrypted. Defaults to `false`
     */
    encrypted?: pulumi.Input<boolean>;
    /**
     * Nested block describing an ephemeral block device that
     * should be attached to created instances. The structure of this block is described below.
     */
    ephemeralBlockDevices?: pulumi.Input<pulumi.Input<inputs.ec2.AmiCopyEphemeralBlockDevice>[]>;
    hypervisor?: pulumi.Input<string>;
    /**
     * Path to an S3 object containing an image manifest, e.g., created
     * by the `ec2-upload-bundle` command in the EC2 command line tools.
     */
    imageLocation?: pulumi.Input<string>;
    imageOwnerAlias?: pulumi.Input<string>;
    imageType?: pulumi.Input<string>;
    /**
     * If EC2 instances started from this image should require the use of the Instance Metadata Service V2 (IMDSv2), set this argument to `v2.0`. For more information, see [Configure instance metadata options for new instances](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/configuring-IMDS-new-instances.html#configure-IMDS-new-instances-ami-configuration).
     */
    imdsSupport?: pulumi.Input<string>;
    /**
     * ID of the kernel image (AKI) that will be used as the paravirtual
     * kernel in created instances.
     */
    kernelId?: pulumi.Input<string>;
    /**
     * Full ARN of the KMS Key to use when encrypting the snapshots of an image during a copy operation. If not specified, then the default AWS KMS Key will be used
     */
    kmsKeyId?: pulumi.Input<string>;
    /**
     * Date and time, in ISO 8601 date-time format , when the AMI was last used to launch an EC2 instance. When the AMI is used to launch an instance, there is a 24-hour delay before that usage is reported. For more information, see the following [AWS document](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/ami-last-launched-time.html).
     */
    lastLaunchedTime?: pulumi.Input<string>;
    manageEbsSnapshots?: pulumi.Input<boolean>;
    /**
     * Region-unique name for the AMI.
     */
    name?: pulumi.Input<string>;
    ownerId?: pulumi.Input<string>;
    platform?: pulumi.Input<string>;
    platformDetails?: pulumi.Input<string>;
    public?: pulumi.Input<boolean>;
    /**
     * ID of an initrd image (ARI) that will be used when booting the
     * created instances.
     */
    ramdiskId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Name of the root device (for example, `/dev/sda1`, or `/dev/xvda`).
     */
    rootDeviceName?: pulumi.Input<string>;
    rootSnapshotId?: pulumi.Input<string>;
    /**
     * Id of the AMI to copy. This id must be valid in the region
     * given by `sourceAmiRegion`.
     */
    sourceAmiId?: pulumi.Input<string>;
    /**
     * Region from which the AMI will be copied. This may be the
     * same as the AWS provider region in order to create a copy within the same region.
     */
    sourceAmiRegion?: pulumi.Input<string>;
    /**
     * When set to "simple" (the default), enables enhanced networking
     * for created instances. No other value is supported at this time.
     */
    sriovNetSupport?: pulumi.Input<string>;
    /**
     * Map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * If the image is configured for NitroTPM support, the value is `v2.0`. For more information, see [NitroTPM](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/nitrotpm.html) in the Amazon Elastic Compute Cloud User Guide.
     */
    tpmSupport?: pulumi.Input<string>;
    /**
     * Base64 representation of the non-volatile UEFI variable store.
     */
    uefiData?: pulumi.Input<string>;
    usageOperation?: pulumi.Input<string>;
    /**
     * Keyword to choose what virtualization mode created instances
     * will use. Can be either "paravirtual" (the default) or "hvm". The choice of virtualization type
     * changes the set of further arguments that are required, as described below.
     */
    virtualizationType?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a AmiCopy resource.
 */
export interface AmiCopyArgs {
    /**
     * Date and time to deprecate the AMI. If you specified a value for seconds, Amazon EC2 rounds the seconds to the nearest minute. Valid values: [RFC3339 time string](https://tools.ietf.org/html/rfc3339#section-5.8) (`YYYY-MM-DDTHH:MM:SSZ`)
     */
    deprecationTime?: pulumi.Input<string>;
    /**
     * Longer, human-readable description for the AMI.
     */
    description?: pulumi.Input<string>;
    /**
     * ARN of the Outpost to which to copy the AMI.
     * Only specify this parameter when copying an AMI from an AWS Region to an Outpost. The AMI must be in the Region of the destination Outpost.
     */
    destinationOutpostArn?: pulumi.Input<string>;
    /**
     * Nested block describing an EBS block device that should be
     * attached to created instances. The structure of this block is described below.
     */
    ebsBlockDevices?: pulumi.Input<pulumi.Input<inputs.ec2.AmiCopyEbsBlockDevice>[]>;
    /**
     * Whether the destination snapshots of the copied image should be encrypted. Defaults to `false`
     */
    encrypted?: pulumi.Input<boolean>;
    /**
     * Nested block describing an ephemeral block device that
     * should be attached to created instances. The structure of this block is described below.
     */
    ephemeralBlockDevices?: pulumi.Input<pulumi.Input<inputs.ec2.AmiCopyEphemeralBlockDevice>[]>;
    /**
     * Full ARN of the KMS Key to use when encrypting the snapshots of an image during a copy operation. If not specified, then the default AWS KMS Key will be used
     */
    kmsKeyId?: pulumi.Input<string>;
    /**
     * Region-unique name for the AMI.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Id of the AMI to copy. This id must be valid in the region
     * given by `sourceAmiRegion`.
     */
    sourceAmiId: pulumi.Input<string>;
    /**
     * Region from which the AMI will be copied. This may be the
     * same as the AWS provider region in order to create a copy within the same region.
     */
    sourceAmiRegion: pulumi.Input<string>;
    /**
     * Map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
