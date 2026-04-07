import * as pulumi from "@pulumi/pulumi";
/**
 * Manages a static IP address.
 *
 * Use this resource to allocate a static IP address that can be attached to Lightsail instances to provide a consistent public IP address that persists across instance restarts.
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
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_lightsail_static_ip` using the name attribute. For example:
 *
 * ```sh
 * $ pulumi import aws:lightsail/staticIp:StaticIp example example
 * ```
 */
export declare class StaticIp extends pulumi.CustomResource {
    /**
     * Get an existing StaticIp resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: StaticIpState, opts?: pulumi.CustomResourceOptions): StaticIp;
    /**
     * Returns true if the given object is an instance of StaticIp.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is StaticIp;
    /**
     * ARN of the Lightsail static IP.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Allocated static IP address.
     */
    readonly ipAddress: pulumi.Output<string>;
    /**
     * Name for the allocated static IP.
     *
     * The following arguments are optional:
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Support code for the static IP. Include this code in your email to support when you have questions about a static IP in Lightsail. This code enables our support team to look up your Lightsail information more easily.
     */
    readonly supportCode: pulumi.Output<string>;
    /**
     * Create a StaticIp resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: StaticIpArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering StaticIp resources.
 */
export interface StaticIpState {
    /**
     * ARN of the Lightsail static IP.
     */
    arn?: pulumi.Input<string>;
    /**
     * Allocated static IP address.
     */
    ipAddress?: pulumi.Input<string>;
    /**
     * Name for the allocated static IP.
     *
     * The following arguments are optional:
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Support code for the static IP. Include this code in your email to support when you have questions about a static IP in Lightsail. This code enables our support team to look up your Lightsail information more easily.
     */
    supportCode?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a StaticIp resource.
 */
export interface StaticIpArgs {
    /**
     * Name for the allocated static IP.
     *
     * The following arguments are optional:
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
