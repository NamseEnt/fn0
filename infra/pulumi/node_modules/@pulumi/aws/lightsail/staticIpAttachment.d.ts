import * as pulumi from "@pulumi/pulumi";
/**
 * Manages a static IP address attachment - relationship between a Lightsail static IP and Lightsail instance.
 *
 * Use this resource to attach a static IP address to a Lightsail instance to provide a consistent public IP address that persists across instance restarts.
 *
 * > **Note:** Lightsail is currently only supported in a limited number of AWS Regions, please see ["Regions and Availability Zones in Amazon Lightsail"](https://lightsail.aws.amazon.com/ls/docs/overview/article/understanding-regions-and-availability-zones-in-amazon-lightsail) for more details.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.lightsail.StaticIp("example", {name: "example"});
 * const exampleInstance = new aws.lightsail.Instance("example", {
 *     name: "example",
 *     availabilityZone: "us-east-1a",
 *     blueprintId: "ubuntu_20_04",
 *     bundleId: "nano_2_0",
 * });
 * const exampleStaticIpAttachment = new aws.lightsail.StaticIpAttachment("example", {
 *     staticIpName: example.id,
 *     instanceName: exampleInstance.id,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_lightsail_static_ip_attachment` using the static IP name. For example:
 *
 * ```sh
 * $ pulumi import aws:lightsail/staticIpAttachment:StaticIpAttachment example example-static-ip
 * ```
 */
export declare class StaticIpAttachment extends pulumi.CustomResource {
    /**
     * Get an existing StaticIpAttachment resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: StaticIpAttachmentState, opts?: pulumi.CustomResourceOptions): StaticIpAttachment;
    /**
     * Returns true if the given object is an instance of StaticIpAttachment.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is StaticIpAttachment;
    /**
     * Name of the Lightsail instance to attach the IP to.
     */
    readonly instanceName: pulumi.Output<string>;
    /**
     * Allocated static IP address.
     */
    readonly ipAddress: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Name of the allocated static IP.
     *
     * The following arguments are optional:
     */
    readonly staticIpName: pulumi.Output<string>;
    /**
     * Create a StaticIpAttachment resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: StaticIpAttachmentArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering StaticIpAttachment resources.
 */
export interface StaticIpAttachmentState {
    /**
     * Name of the Lightsail instance to attach the IP to.
     */
    instanceName?: pulumi.Input<string>;
    /**
     * Allocated static IP address.
     */
    ipAddress?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Name of the allocated static IP.
     *
     * The following arguments are optional:
     */
    staticIpName?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a StaticIpAttachment resource.
 */
export interface StaticIpAttachmentArgs {
    /**
     * Name of the Lightsail instance to attach the IP to.
     */
    instanceName: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Name of the allocated static IP.
     *
     * The following arguments are optional:
     */
    staticIpName: pulumi.Input<string>;
}
