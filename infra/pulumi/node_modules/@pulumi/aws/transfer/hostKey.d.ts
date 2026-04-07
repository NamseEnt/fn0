import * as pulumi from "@pulumi/pulumi";
/**
 * Manages a host key for a server. This is an [_additional server host key_](https://docs.aws.amazon.com/transfer/latest/userguide/server-host-key-add.html).
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.transfer.HostKey("example", {
 *     serverId: exampleAwsTransferServer.id,
 *     description: "example additional host key",
 *     hostKeyBodyWo: "# Private key PEM.\n",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import host keys using the `server_id` and `host_key_id` separated by `,`. For example:
 *
 * ```sh
 * $ pulumi import aws:transfer/hostKey:HostKey example s-12345678,key-12345
 * ```
 */
export declare class HostKey extends pulumi.CustomResource {
    /**
     * Get an existing HostKey resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: HostKeyState, opts?: pulumi.CustomResourceOptions): HostKey;
    /**
     * Returns true if the given object is an instance of HostKey.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is HostKey;
    /**
     * Amazon Resource Name (ARN) of host key.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Text description.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * Private key portion of an SSH key pair.
     */
    readonly hostKeyBody: pulumi.Output<string | undefined>;
    /**
     * **NOTE:** This field is write-only and its value will not be updated in state as part of read operations.
     * Write-only private key portion of an SSH key pair, guaranteed not to be written to plan or state artifacts. One of `hostKeyBody` or `hostKeyBodyWo` must be configured.
     */
    readonly hostKeyBodyWo: pulumi.Output<string | undefined>;
    /**
     * Public key fingerprint.
     */
    readonly hostKeyFingerprint: pulumi.Output<string>;
    /**
     * ID of the host key.
     */
    readonly hostKeyId: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Server ID.
     */
    readonly serverId: pulumi.Output<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
    /**
     * Create a HostKey resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: HostKeyArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering HostKey resources.
 */
export interface HostKeyState {
    /**
     * Amazon Resource Name (ARN) of host key.
     */
    arn?: pulumi.Input<string>;
    /**
     * Text description.
     */
    description?: pulumi.Input<string>;
    /**
     * Private key portion of an SSH key pair.
     */
    hostKeyBody?: pulumi.Input<string>;
    /**
     * **NOTE:** This field is write-only and its value will not be updated in state as part of read operations.
     * Write-only private key portion of an SSH key pair, guaranteed not to be written to plan or state artifacts. One of `hostKeyBody` or `hostKeyBodyWo` must be configured.
     */
    hostKeyBodyWo?: pulumi.Input<string>;
    /**
     * Public key fingerprint.
     */
    hostKeyFingerprint?: pulumi.Input<string>;
    /**
     * ID of the host key.
     */
    hostKeyId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Server ID.
     */
    serverId?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
}
/**
 * The set of arguments for constructing a HostKey resource.
 */
export interface HostKeyArgs {
    /**
     * Text description.
     */
    description?: pulumi.Input<string>;
    /**
     * Private key portion of an SSH key pair.
     */
    hostKeyBody?: pulumi.Input<string>;
    /**
     * **NOTE:** This field is write-only and its value will not be updated in state as part of read operations.
     * Write-only private key portion of an SSH key pair, guaranteed not to be written to plan or state artifacts. One of `hostKeyBody` or `hostKeyBodyWo` must be configured.
     */
    hostKeyBodyWo?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Server ID.
     */
    serverId: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
