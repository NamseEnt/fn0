import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages an NFS Location within AWS DataSync.
 *
 * > **NOTE:** The DataSync Agents must be available before creating this resource.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.datasync.NfsLocation("example", {
 *     serverHostname: "nfs.example.com",
 *     subdirectory: "/exported/path",
 *     onPremConfig: {
 *         agentArns: [exampleAwsDatasyncAgent.arn],
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * ### Identity Schema
 *
 * #### Required
 *
 * - `arn` (String) Amazon Resource Name (ARN) of the DataSync NFS location.
 *
 * Using `pulumi import`, import `aws_datasync_location_nfs` using the DataSync Task Amazon Resource Name (ARN). For example:
 *
 * console
 *
 * % pulumi import aws_datasync_location_nfs.example arn:aws:datasync:us-east-1:123456789012:location/loc-12345678901234567
 */
export declare class NfsLocation extends pulumi.CustomResource {
    /**
     * Get an existing NfsLocation resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: NfsLocationState, opts?: pulumi.CustomResourceOptions): NfsLocation;
    /**
     * Returns true if the given object is an instance of NfsLocation.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is NfsLocation;
    /**
     * Amazon Resource Name (ARN) of the DataSync Location.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Configuration block containing mount options used by DataSync to access the NFS Server.
     */
    readonly mountOptions: pulumi.Output<outputs.datasync.NfsLocationMountOptions | undefined>;
    /**
     * Configuration block containing information for connecting to the NFS File System.
     */
    readonly onPremConfig: pulumi.Output<outputs.datasync.NfsLocationOnPremConfig>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Specifies the IP address or DNS name of the NFS server. The DataSync Agent(s) use this to mount the NFS server.
     */
    readonly serverHostname: pulumi.Output<string>;
    /**
     * Subdirectory to perform actions as source or destination. Should be exported by the NFS server.
     */
    readonly subdirectory: pulumi.Output<string>;
    /**
     * Key-value pairs of resource tags to assign to the DataSync Location. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
    readonly uri: pulumi.Output<string>;
    /**
     * Create a NfsLocation resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: NfsLocationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering NfsLocation resources.
 */
export interface NfsLocationState {
    /**
     * Amazon Resource Name (ARN) of the DataSync Location.
     */
    arn?: pulumi.Input<string>;
    /**
     * Configuration block containing mount options used by DataSync to access the NFS Server.
     */
    mountOptions?: pulumi.Input<inputs.datasync.NfsLocationMountOptions>;
    /**
     * Configuration block containing information for connecting to the NFS File System.
     */
    onPremConfig?: pulumi.Input<inputs.datasync.NfsLocationOnPremConfig>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Specifies the IP address or DNS name of the NFS server. The DataSync Agent(s) use this to mount the NFS server.
     */
    serverHostname?: pulumi.Input<string>;
    /**
     * Subdirectory to perform actions as source or destination. Should be exported by the NFS server.
     */
    subdirectory?: pulumi.Input<string>;
    /**
     * Key-value pairs of resource tags to assign to the DataSync Location. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
    uri?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a NfsLocation resource.
 */
export interface NfsLocationArgs {
    /**
     * Configuration block containing mount options used by DataSync to access the NFS Server.
     */
    mountOptions?: pulumi.Input<inputs.datasync.NfsLocationMountOptions>;
    /**
     * Configuration block containing information for connecting to the NFS File System.
     */
    onPremConfig: pulumi.Input<inputs.datasync.NfsLocationOnPremConfig>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Specifies the IP address or DNS name of the NFS server. The DataSync Agent(s) use this to mount the NFS server.
     */
    serverHostname: pulumi.Input<string>;
    /**
     * Subdirectory to perform actions as source or destination. Should be exported by the NFS server.
     */
    subdirectory: pulumi.Input<string>;
    /**
     * Key-value pairs of resource tags to assign to the DataSync Location. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
