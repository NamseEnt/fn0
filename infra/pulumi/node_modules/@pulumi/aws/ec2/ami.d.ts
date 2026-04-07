import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * The AMI resource allows the creation and management of a completely-custom
 * *Amazon Machine Image* (AMI).
 *
 * If you just want to duplicate an existing AMI, possibly copying it to another
 * region, it's better to use `aws.ec2.AmiCopy` instead.
 *
 * If you just want to share an existing AMI with another AWS account,
 * it's better to use `aws.ec2.AmiLaunchPermission` instead.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * // Create an AMI that will start a machine whose root device is backed by
 * // an EBS volume populated from a snapshot. We assume that such a snapshot
 * // already exists with the id "snap-xxxxxxxx".
 * const example = new aws.ec2.Ami("example", {
 *     name: "example",
 *     virtualizationType: "hvm",
 *     rootDeviceName: "/dev/xvda",
 *     imdsSupport: "v2.0",
 *     ebsBlockDevices: [{
 *         deviceName: "/dev/xvda",
 *         snapshotId: "snap-xxxxxxxx",
 *         volumeSize: 8,
 *     }],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_ami` using the ID of the AMI. For example:
 *
 * ```sh
 * $ pulumi import aws:ec2/ami:Ami example ami-12345678
 * ```
 */
export declare class Ami extends pulumi.CustomResource {
    /**
     * Get an existing Ami resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: AmiState, opts?: pulumi.CustomResourceOptions): Ami;
    /**
     * Returns true if the given object is an instance of Ami.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Ami;
    /**
     * Machine architecture for created instances. Defaults to `x8664`.
     */
    readonly architecture: pulumi.Output<string | undefined>;
    /**
     * ARN of the AMI.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Boot mode of the AMI. For more information, see [Boot modes](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/ami-boot.html) in the Amazon Elastic Compute Cloud User Guide.
     */
    readonly bootMode: pulumi.Output<string | undefined>;
    /**
     * Date and time to deprecate the AMI. If you specified a value for seconds, Amazon EC2 rounds the seconds to the nearest minute. Valid values: [RFC3339 time string](https://tools.ietf.org/html/rfc3339#section-5.8) (`YYYY-MM-DDTHH:MM:SSZ`)
     */
    readonly deprecationTime: pulumi.Output<string | undefined>;
    /**
     * Longer, human-readable description for the AMI.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * Nested block describing an EBS block device that should be
     * attached to created instances. The structure of this block is described below.
     */
    readonly ebsBlockDevices: pulumi.Output<outputs.ec2.AmiEbsBlockDevice[]>;
    /**
     * Whether enhanced networking with ENA is enabled. Defaults to `false`.
     */
    readonly enaSupport: pulumi.Output<boolean | undefined>;
    /**
     * Nested block describing an ephemeral block device that
     * should be attached to created instances. The structure of this block is described below.
     */
    readonly ephemeralBlockDevices: pulumi.Output<outputs.ec2.AmiEphemeralBlockDevice[]>;
    /**
     * Hypervisor type of the image.
     */
    readonly hypervisor: pulumi.Output<string>;
    readonly imageLocation: pulumi.Output<string>;
    /**
     * AWS account alias (for example, amazon, self) or the AWS account ID of the AMI owner.
     */
    readonly imageOwnerAlias: pulumi.Output<string>;
    /**
     * Type of image.
     */
    readonly imageType: pulumi.Output<string>;
    /**
     * If EC2 instances started from this image should require the use of the Instance Metadata Service V2 (IMDSv2), set this argument to `v2.0`. For more information, see [Configure instance metadata options for new instances](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/configuring-IMDS-new-instances.html#configure-IMDS-new-instances-ami-configuration).
     */
    readonly imdsSupport: pulumi.Output<string | undefined>;
    readonly kernelId: pulumi.Output<string | undefined>;
    /**
     * Date and time, in ISO 8601 date-time format , when the AMI was last used to launch an EC2 instance. When the AMI is used to launch an instance, there is a 24-hour delay before that usage is reported. For more information, see the following [AWS document](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/ami-last-launched-time.html).
     */
    readonly lastLaunchedTime: pulumi.Output<string>;
    readonly manageEbsSnapshots: pulumi.Output<boolean>;
    /**
     * Region-unique name for the AMI.
     */
    readonly name: pulumi.Output<string>;
    /**
     * AWS account ID of the image owner.
     */
    readonly ownerId: pulumi.Output<string>;
    /**
     * This value is set to windows for Windows AMIs; otherwise, it is blank.
     */
    readonly platform: pulumi.Output<string>;
    /**
     * Platform details associated with the billing code of the AMI.
     */
    readonly platformDetails: pulumi.Output<string>;
    /**
     * Whether the image has public launch permissions.
     */
    readonly public: pulumi.Output<boolean>;
    readonly ramdiskId: pulumi.Output<string | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Name of the root device (for example, `/dev/sda1`, or `/dev/xvda`).
     */
    readonly rootDeviceName: pulumi.Output<string | undefined>;
    /**
     * Snapshot ID for the root volume (for EBS-backed AMIs)
     */
    readonly rootSnapshotId: pulumi.Output<string>;
    readonly sriovNetSupport: pulumi.Output<string | undefined>;
    /**
     * Map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * Map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * If the image is configured for NitroTPM support, the value is `v2.0`. For more information, see [NitroTPM](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/nitrotpm.html) in the Amazon Elastic Compute Cloud User Guide.
     */
    readonly tpmSupport: pulumi.Output<string | undefined>;
    /**
     * Base64 representation of the non-volatile UEFI variable store.
     */
    readonly uefiData: pulumi.Output<string | undefined>;
    /**
     * Operation of the Amazon EC2 instance and the billing code that is associated with the AMI.
     */
    readonly usageOperation: pulumi.Output<string>;
    /**
     * Keyword to choose what virtualization mode created instances
     * will use. Can be either "paravirtual" (the default) or "hvm". The choice of virtualization type
     * changes the set of further arguments that are required, as described below.
     */
    readonly virtualizationType: pulumi.Output<string | undefined>;
    /**
     * Create a Ami resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: AmiArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Ami resources.
 */
export interface AmiState {
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
     * Nested block describing an EBS block device that should be
     * attached to created instances. The structure of this block is described below.
     */
    ebsBlockDevices?: pulumi.Input<pulumi.Input<inputs.ec2.AmiEbsBlockDevice>[]>;
    /**
     * Whether enhanced networking with ENA is enabled. Defaults to `false`.
     */
    enaSupport?: pulumi.Input<boolean>;
    /**
     * Nested block describing an ephemeral block device that
     * should be attached to created instances. The structure of this block is described below.
     */
    ephemeralBlockDevices?: pulumi.Input<pulumi.Input<inputs.ec2.AmiEphemeralBlockDevice>[]>;
    /**
     * Hypervisor type of the image.
     */
    hypervisor?: pulumi.Input<string>;
    imageLocation?: pulumi.Input<string>;
    /**
     * AWS account alias (for example, amazon, self) or the AWS account ID of the AMI owner.
     */
    imageOwnerAlias?: pulumi.Input<string>;
    /**
     * Type of image.
     */
    imageType?: pulumi.Input<string>;
    /**
     * If EC2 instances started from this image should require the use of the Instance Metadata Service V2 (IMDSv2), set this argument to `v2.0`. For more information, see [Configure instance metadata options for new instances](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/configuring-IMDS-new-instances.html#configure-IMDS-new-instances-ami-configuration).
     */
    imdsSupport?: pulumi.Input<string>;
    kernelId?: pulumi.Input<string>;
    /**
     * Date and time, in ISO 8601 date-time format , when the AMI was last used to launch an EC2 instance. When the AMI is used to launch an instance, there is a 24-hour delay before that usage is reported. For more information, see the following [AWS document](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/ami-last-launched-time.html).
     */
    lastLaunchedTime?: pulumi.Input<string>;
    manageEbsSnapshots?: pulumi.Input<boolean>;
    /**
     * Region-unique name for the AMI.
     */
    name?: pulumi.Input<string>;
    /**
     * AWS account ID of the image owner.
     */
    ownerId?: pulumi.Input<string>;
    /**
     * This value is set to windows for Windows AMIs; otherwise, it is blank.
     */
    platform?: pulumi.Input<string>;
    /**
     * Platform details associated with the billing code of the AMI.
     */
    platformDetails?: pulumi.Input<string>;
    /**
     * Whether the image has public launch permissions.
     */
    public?: pulumi.Input<boolean>;
    ramdiskId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Name of the root device (for example, `/dev/sda1`, or `/dev/xvda`).
     */
    rootDeviceName?: pulumi.Input<string>;
    /**
     * Snapshot ID for the root volume (for EBS-backed AMIs)
     */
    rootSnapshotId?: pulumi.Input<string>;
    sriovNetSupport?: pulumi.Input<string>;
    /**
     * Map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
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
    /**
     * Operation of the Amazon EC2 instance and the billing code that is associated with the AMI.
     */
    usageOperation?: pulumi.Input<string>;
    /**
     * Keyword to choose what virtualization mode created instances
     * will use. Can be either "paravirtual" (the default) or "hvm". The choice of virtualization type
     * changes the set of further arguments that are required, as described below.
     */
    virtualizationType?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a Ami resource.
 */
export interface AmiArgs {
    /**
     * Machine architecture for created instances. Defaults to `x8664`.
     */
    architecture?: pulumi.Input<string>;
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
     * Nested block describing an EBS block device that should be
     * attached to created instances. The structure of this block is described below.
     */
    ebsBlockDevices?: pulumi.Input<pulumi.Input<inputs.ec2.AmiEbsBlockDevice>[]>;
    /**
     * Whether enhanced networking with ENA is enabled. Defaults to `false`.
     */
    enaSupport?: pulumi.Input<boolean>;
    /**
     * Nested block describing an ephemeral block device that
     * should be attached to created instances. The structure of this block is described below.
     */
    ephemeralBlockDevices?: pulumi.Input<pulumi.Input<inputs.ec2.AmiEphemeralBlockDevice>[]>;
    imageLocation?: pulumi.Input<string>;
    /**
     * If EC2 instances started from this image should require the use of the Instance Metadata Service V2 (IMDSv2), set this argument to `v2.0`. For more information, see [Configure instance metadata options for new instances](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/configuring-IMDS-new-instances.html#configure-IMDS-new-instances-ami-configuration).
     */
    imdsSupport?: pulumi.Input<string>;
    kernelId?: pulumi.Input<string>;
    /**
     * Region-unique name for the AMI.
     */
    name?: pulumi.Input<string>;
    ramdiskId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Name of the root device (for example, `/dev/sda1`, or `/dev/xvda`).
     */
    rootDeviceName?: pulumi.Input<string>;
    sriovNetSupport?: pulumi.Input<string>;
    /**
     * Map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
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
    /**
     * Keyword to choose what virtualization mode created instances
     * will use. Can be either "paravirtual" (the default) or "hvm". The choice of virtualization type
     * changes the set of further arguments that are required, as described below.
     */
    virtualizationType?: pulumi.Input<string>;
}
