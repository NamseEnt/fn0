import * as pulumi from "@pulumi/pulumi";
/**
 * Manages a Lightsail disk. Use this resource to create additional block storage that can be attached to Lightsail instances for extra storage capacity.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const available = aws.getAvailabilityZones({
 *     state: "available",
 *     filters: [{
 *         name: "opt-in-status",
 *         values: ["opt-in-not-required"],
 *     }],
 * });
 * const example = new aws.lightsail.Disk("example", {
 *     name: "example-disk",
 *     sizeInGb: 8,
 *     availabilityZone: available.then(available => available.names?.[0]),
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_lightsail_disk` using the name attribute. For example:
 *
 * ```sh
 * $ pulumi import aws:lightsail/disk:Disk example example-disk
 * ```
 */
export declare class Disk extends pulumi.CustomResource {
    /**
     * Get an existing Disk resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: DiskState, opts?: pulumi.CustomResourceOptions): Disk;
    /**
     * Returns true if the given object is an instance of Disk.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Disk;
    /**
     * ARN of the disk.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Availability Zone in which to create the disk.
     */
    readonly availabilityZone: pulumi.Output<string>;
    /**
     * Date and time when the disk was created.
     */
    readonly createdAt: pulumi.Output<string>;
    /**
     * Name of the disk. Must begin with an alphabetic character and contain only alphanumeric characters, underscores, hyphens, and dots.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Size of the disk in GB.
     *
     * The following arguments are optional:
     */
    readonly sizeInGb: pulumi.Output<number>;
    /**
     * Support code for the disk. Include this code in your email to support when you have questions about a disk in Lightsail.
     */
    readonly supportCode: pulumi.Output<string>;
    /**
     * Map of tags to assign to the resource. To create a key-only tag, use an empty string as the value. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Create a Disk resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: DiskArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Disk resources.
 */
export interface DiskState {
    /**
     * ARN of the disk.
     */
    arn?: pulumi.Input<string>;
    /**
     * Availability Zone in which to create the disk.
     */
    availabilityZone?: pulumi.Input<string>;
    /**
     * Date and time when the disk was created.
     */
    createdAt?: pulumi.Input<string>;
    /**
     * Name of the disk. Must begin with an alphabetic character and contain only alphanumeric characters, underscores, hyphens, and dots.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Size of the disk in GB.
     *
     * The following arguments are optional:
     */
    sizeInGb?: pulumi.Input<number>;
    /**
     * Support code for the disk. Include this code in your email to support when you have questions about a disk in Lightsail.
     */
    supportCode?: pulumi.Input<string>;
    /**
     * Map of tags to assign to the resource. To create a key-only tag, use an empty string as the value. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
}
/**
 * The set of arguments for constructing a Disk resource.
 */
export interface DiskArgs {
    /**
     * Availability Zone in which to create the disk.
     */
    availabilityZone: pulumi.Input<string>;
    /**
     * Name of the disk. Must begin with an alphabetic character and contain only alphanumeric characters, underscores, hyphens, and dots.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Size of the disk in GB.
     *
     * The following arguments are optional:
     */
    sizeInGb: pulumi.Input<number>;
    /**
     * Map of tags to assign to the resource. To create a key-only tag, use an empty string as the value. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
