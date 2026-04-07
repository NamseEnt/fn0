import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * The "AMI from instance" resource allows the creation of an Amazon Machine
 * Image (AMI) modeled after an existing EBS-backed EC2 instance.
 *
 * The created AMI will refer to implicitly-created snapshots of the instance's
 * EBS volumes and mimick its assigned block device configuration at the time
 * the resource is created.
 *
 * This resource is best applied to an instance that is stopped when this instance
 * is created, so that the contents of the created image are predictable. When
 * applied to an instance that is running, *the instance will be stopped before taking
 * the snapshots and then started back up again*, resulting in a period of
 * downtime.
 *
 * Note that the source instance is inspected only at the initial creation of this
 * resource. Ongoing updates to the referenced instance will not be propagated into
 * the generated AMI. Users may taint or otherwise recreate the resource in order
 * to produce a fresh snapshot.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.ec2.AmiFromInstance("example", {
 *     name: "example",
 *     sourceInstanceId: "i-xxxxxxxx",
 * });
 * ```
 */
export declare class AmiFromInstance extends pulumi.CustomResource {
    /**
     * Get an existing AmiFromInstance resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: AmiFromInstanceState, opts?: pulumi.CustomResourceOptions): AmiFromInstance;
    /**
     * Returns true if the given object is an instance of AmiFromInstance.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is AmiFromInstance;
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
     * Nested block describing an EBS block device that should be
     * attached to created instances. The structure of this block is described below.
     */
    readonly ebsBlockDevices: pulumi.Output<outputs.ec2.AmiFromInstanceEbsBlockDevice[]>;
    /**
     * Whether enhanced networking with ENA is enabled. Defaults to `false`.
     */
    readonly enaSupport: pulumi.Output<boolean>;
    /**
     * Nested block describing an ephemeral block device that
     * should be attached to created instances. The structure of this block is described below.
     */
    readonly ephemeralBlockDevices: pulumi.Output<outputs.ec2.AmiFromInstanceEphemeralBlockDevice[]>;
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
     * Boolean that overrides the behavior of stopping
     * the instance before snapshotting. This is risky since it may cause a snapshot of an
     * inconsistent filesystem state, but can be used to avoid downtime if the user otherwise
     * guarantees that no filesystem writes will be underway at the time of snapshot.
     */
    readonly snapshotWithoutReboot: pulumi.Output<boolean | undefined>;
    /**
     * ID of the instance to use as the basis of the AMI.
     */
    readonly sourceInstanceId: pulumi.Output<string>;
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
     * Create a AmiFromInstance resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: AmiFromInstanceArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering AmiFromInstance resources.
 */
export interface AmiFromInstanceState {
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
    ebsBlockDevices?: pulumi.Input<pulumi.Input<inputs.ec2.AmiFromInstanceEbsBlockDevice>[]>;
    /**
     * Whether enhanced networking with ENA is enabled. Defaults to `false`.
     */
    enaSupport?: pulumi.Input<boolean>;
    /**
     * Nested block describing an ephemeral block device that
     * should be attached to created instances. The structure of this block is described below.
     */
    ephemeralBlockDevices?: pulumi.Input<pulumi.Input<inputs.ec2.AmiFromInstanceEphemeralBlockDevice>[]>;
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
     * Boolean that overrides the behavior of stopping
     * the instance before snapshotting. This is risky since it may cause a snapshot of an
     * inconsistent filesystem state, but can be used to avoid downtime if the user otherwise
     * guarantees that no filesystem writes will be underway at the time of snapshot.
     */
    snapshotWithoutReboot?: pulumi.Input<boolean>;
    /**
     * ID of the instance to use as the basis of the AMI.
     */
    sourceInstanceId?: pulumi.Input<string>;
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
 * The set of arguments for constructing a AmiFromInstance resource.
 */
export interface AmiFromInstanceArgs {
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
    ebsBlockDevices?: pulumi.Input<pulumi.Input<inputs.ec2.AmiFromInstanceEbsBlockDevice>[]>;
    /**
     * Nested block describing an ephemeral block device that
     * should be attached to created instances. The structure of this block is described below.
     */
    ephemeralBlockDevices?: pulumi.Input<pulumi.Input<inputs.ec2.AmiFromInstanceEphemeralBlockDevice>[]>;
    /**
     * Region-unique name for the AMI.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Boolean that overrides the behavior of stopping
     * the instance before snapshotting. This is risky since it may cause a snapshot of an
     * inconsistent filesystem state, but can be used to avoid downtime if the user otherwise
     * guarantees that no filesystem writes will be underway at the time of snapshot.
     */
    snapshotWithoutReboot?: pulumi.Input<boolean>;
    /**
     * ID of the instance to use as the basis of the AMI.
     */
    sourceInstanceId: pulumi.Input<string>;
    /**
     * Map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
