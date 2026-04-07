import * as pulumi from "@pulumi/pulumi";
/**
 * Manages a Lightsail disk attachment. Use this resource to attach additional storage disks to your Lightsail instances for expanded storage capacity.
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
 * const exampleInstance = new aws.lightsail.Instance("example", {
 *     name: "example-instance",
 *     availabilityZone: available.then(available => available.names?.[0]),
 *     blueprintId: "amazon_linux_2",
 *     bundleId: "nano_3_0",
 * });
 * const exampleDisk_attachment = new aws.lightsail.Disk_attachment("example", {
 *     diskName: example.name,
 *     instanceName: exampleInstance.name,
 *     diskPath: "/dev/xvdf",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_lightsail_disk_attachment` using the id attribute. For example:
 *
 * ```sh
 * $ pulumi import aws:lightsail/disk_attachment:Disk_attachment example example-disk,example-instance
 * ```
 */
export declare class Disk_attachment extends pulumi.CustomResource {
    /**
     * Get an existing Disk_attachment resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: Disk_attachmentState, opts?: pulumi.CustomResourceOptions): Disk_attachment;
    /**
     * Returns true if the given object is an instance of Disk_attachment.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Disk_attachment;
    /**
     * Name of the Lightsail disk.
     */
    readonly diskName: pulumi.Output<string>;
    /**
     * Disk path to expose to the instance.
     */
    readonly diskPath: pulumi.Output<string>;
    /**
     * Name of the Lightsail instance to attach to.
     */
    readonly instanceName: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Create a Disk_attachment resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: Disk_attachmentArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Disk_attachment resources.
 */
export interface Disk_attachmentState {
    /**
     * Name of the Lightsail disk.
     */
    diskName?: pulumi.Input<string>;
    /**
     * Disk path to expose to the instance.
     */
    diskPath?: pulumi.Input<string>;
    /**
     * Name of the Lightsail instance to attach to.
     */
    instanceName?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a Disk_attachment resource.
 */
export interface Disk_attachmentArgs {
    /**
     * Name of the Lightsail disk.
     */
    diskName: pulumi.Input<string>;
    /**
     * Disk path to expose to the instance.
     */
    diskPath: pulumi.Input<string>;
    /**
     * Name of the Lightsail instance to attach to.
     */
    instanceName: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
