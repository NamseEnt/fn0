import * as pulumi from "@pulumi/pulumi";
/**
 * Provides an EC2 instance state resource. This allows managing an instance power state.
 *
 * > **NOTE on Instance State Management:** AWS does not currently have an EC2 API operation to determine an instance has finished processing user data. As a result, this resource can interfere with user data processing. For example, this resource may stop an instance while the user data script is in mid run.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const ubuntu = aws.ec2.getAmi({
 *     mostRecent: true,
 *     filters: [
 *         {
 *             name: "name",
 *             values: ["ubuntu/images/hvm-ssd/ubuntu-focal-20.04-amd64-server-*"],
 *         },
 *         {
 *             name: "virtualization-type",
 *             values: ["hvm"],
 *         },
 *     ],
 *     owners: ["099720109477"],
 * });
 * const test = new aws.ec2.Instance("test", {
 *     ami: ubuntu.then(ubuntu => ubuntu.id),
 *     instanceType: aws.ec2.InstanceType.T3_Micro,
 *     tags: {
 *         Name: "HelloWorld",
 *     },
 * });
 * const testInstanceState = new aws.ec2transitgateway.InstanceState("test", {
 *     instanceId: test.id,
 *     state: "stopped",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_ec2_instance_state` using the `instance_id` attribute. For example:
 *
 * ```sh
 * $ pulumi import aws:ec2transitgateway/instanceState:InstanceState test i-02cae6557dfcf2f96
 * ```
 */
export declare class InstanceState extends pulumi.CustomResource {
    /**
     * Get an existing InstanceState resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: InstanceStateState, opts?: pulumi.CustomResourceOptions): InstanceState;
    /**
     * Returns true if the given object is an instance of InstanceState.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is InstanceState;
    /**
     * Whether to request a forced stop when `state` is `stopped`. Otherwise (_i.e._, `state` is `running`), ignored. When an instance is forced to stop, it does not flush file system caches or file system metadata, and you must subsequently perform file system check and repair. Not recommended for Windows instances. Defaults to `false`.
     */
    readonly force: pulumi.Output<boolean | undefined>;
    /**
     * ID of the instance.
     */
    readonly instanceId: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * State of the instance. Valid values are `stopped`, `running`.
     *
     * The following arguments are optional:
     */
    readonly state: pulumi.Output<string>;
    /**
     * Create a InstanceState resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: InstanceStateArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering InstanceState resources.
 */
export interface InstanceStateState {
    /**
     * Whether to request a forced stop when `state` is `stopped`. Otherwise (_i.e._, `state` is `running`), ignored. When an instance is forced to stop, it does not flush file system caches or file system metadata, and you must subsequently perform file system check and repair. Not recommended for Windows instances. Defaults to `false`.
     */
    force?: pulumi.Input<boolean>;
    /**
     * ID of the instance.
     */
    instanceId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * State of the instance. Valid values are `stopped`, `running`.
     *
     * The following arguments are optional:
     */
    state?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a InstanceState resource.
 */
export interface InstanceStateArgs {
    /**
     * Whether to request a forced stop when `state` is `stopped`. Otherwise (_i.e._, `state` is `running`), ignored. When an instance is forced to stop, it does not flush file system caches or file system metadata, and you must subsequently perform file system check and repair. Not recommended for Windows instances. Defaults to `false`.
     */
    force?: pulumi.Input<boolean>;
    /**
     * ID of the instance.
     */
    instanceId: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * State of the instance. Valid values are `stopped`, `running`.
     *
     * The following arguments are optional:
     */
    state: pulumi.Input<string>;
}
