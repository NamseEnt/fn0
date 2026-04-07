import * as pulumi from "@pulumi/pulumi";
/**
 * Attach an Elastic network interface (ENI) resource with EC2 instance.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = new aws.ec2.NetworkInterfaceAttachment("test", {
 *     instanceId: testAwsInstance.id,
 *     networkInterfaceId: testAwsNetworkInterface.id,
 *     deviceIndex: 0,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Elastic network interface (ENI) Attachments using its Attachment ID. For example:
 *
 * ```sh
 * $ pulumi import aws:ec2/networkInterfaceAttachment:NetworkInterfaceAttachment secondary_nic eni-attach-0a33842b4ec347c4c
 * ```
 */
export declare class NetworkInterfaceAttachment extends pulumi.CustomResource {
    /**
     * Get an existing NetworkInterfaceAttachment resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: NetworkInterfaceAttachmentState, opts?: pulumi.CustomResourceOptions): NetworkInterfaceAttachment;
    /**
     * Returns true if the given object is an instance of NetworkInterfaceAttachment.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is NetworkInterfaceAttachment;
    /**
     * The ENI Attachment ID.
     */
    readonly attachmentId: pulumi.Output<string>;
    /**
     * Network interface index (int).
     */
    readonly deviceIndex: pulumi.Output<number>;
    /**
     * Instance ID to attach.
     */
    readonly instanceId: pulumi.Output<string>;
    /**
     * Index of the network card. Specify a value greater than 0 when using multiple network cards, which are supported by [some instance types](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/using-eni.html#network-cards). The default is 0.
     */
    readonly networkCardIndex: pulumi.Output<number>;
    /**
     * ENI ID to attach.
     */
    readonly networkInterfaceId: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The status of the Network Interface Attachment.
     */
    readonly status: pulumi.Output<string>;
    /**
     * Create a NetworkInterfaceAttachment resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: NetworkInterfaceAttachmentArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering NetworkInterfaceAttachment resources.
 */
export interface NetworkInterfaceAttachmentState {
    /**
     * The ENI Attachment ID.
     */
    attachmentId?: pulumi.Input<string>;
    /**
     * Network interface index (int).
     */
    deviceIndex?: pulumi.Input<number>;
    /**
     * Instance ID to attach.
     */
    instanceId?: pulumi.Input<string>;
    /**
     * Index of the network card. Specify a value greater than 0 when using multiple network cards, which are supported by [some instance types](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/using-eni.html#network-cards). The default is 0.
     */
    networkCardIndex?: pulumi.Input<number>;
    /**
     * ENI ID to attach.
     */
    networkInterfaceId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The status of the Network Interface Attachment.
     */
    status?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a NetworkInterfaceAttachment resource.
 */
export interface NetworkInterfaceAttachmentArgs {
    /**
     * Network interface index (int).
     */
    deviceIndex: pulumi.Input<number>;
    /**
     * Instance ID to attach.
     */
    instanceId: pulumi.Input<string>;
    /**
     * Index of the network card. Specify a value greater than 0 when using multiple network cards, which are supported by [some instance types](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/using-eni.html#network-cards). The default is 0.
     */
    networkCardIndex?: pulumi.Input<number>;
    /**
     * ENI ID to attach.
     */
    networkInterfaceId: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
