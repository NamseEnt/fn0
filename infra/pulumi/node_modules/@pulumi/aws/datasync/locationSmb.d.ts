import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages a SMB Location within AWS DataSync.
 *
 * > **NOTE:** The DataSync Agents must be available before creating this resource.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.datasync.LocationSmb("example", {
 *     serverHostname: "smb.example.com",
 *     subdirectory: "/exported/path",
 *     user: "Guest",
 *     password: "ANotGreatPassword",
 *     agentArns: [exampleAwsDatasyncAgent.arn],
 * });
 * ```
 *
 * ## Import
 *
 * ### Identity Schema
 *
 * #### Required
 *
 * - `arn` (String) Amazon Resource Name (ARN) of the DataSync SMB location.
 *
 * Using `pulumi import`, import `aws_datasync_location_smb` using the Amazon Resource Name (ARN). For example:
 *
 * console
 *
 * % pulumi import aws_datasync_location_smb.example arn:aws:datasync:us-east-1:123456789012:location/loc-12345678901234567
 */
export declare class LocationSmb extends pulumi.CustomResource {
    /**
     * Get an existing LocationSmb resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: LocationSmbState, opts?: pulumi.CustomResourceOptions): LocationSmb;
    /**
     * Returns true if the given object is an instance of LocationSmb.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is LocationSmb;
    /**
     * A list of DataSync Agent ARNs with which this location will be associated.
     */
    readonly agentArns: pulumi.Output<string[]>;
    /**
     * Amazon Resource Name (ARN) of the DataSync Location.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The name of the Windows domain the SMB server belongs to.
     */
    readonly domain: pulumi.Output<string>;
    /**
     * Configuration block containing mount options used by DataSync to access the SMB Server. Can be `AUTOMATIC`, `SMB2`, or `SMB3`.
     */
    readonly mountOptions: pulumi.Output<outputs.datasync.LocationSmbMountOptions | undefined>;
    /**
     * The password of the user who can mount the share and has file permissions in the SMB.
     */
    readonly password: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Specifies the IP address or DNS name of the SMB server. The DataSync Agent(s) use this to mount the SMB share.
     */
    readonly serverHostname: pulumi.Output<string>;
    /**
     * Subdirectory to perform actions as source or destination. Should be exported by the NFS server.
     */
    readonly subdirectory: pulumi.Output<string>;
    /**
     * Key-value pairs of resource tags to assign to the DataSync Location. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * The user who can mount the share and has file and folder permissions in the SMB share.
     */
    readonly user: pulumi.Output<string>;
    /**
     * Create a LocationSmb resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: LocationSmbArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering LocationSmb resources.
 */
export interface LocationSmbState {
    /**
     * A list of DataSync Agent ARNs with which this location will be associated.
     */
    agentArns?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Amazon Resource Name (ARN) of the DataSync Location.
     */
    arn?: pulumi.Input<string>;
    /**
     * The name of the Windows domain the SMB server belongs to.
     */
    domain?: pulumi.Input<string>;
    /**
     * Configuration block containing mount options used by DataSync to access the SMB Server. Can be `AUTOMATIC`, `SMB2`, or `SMB3`.
     */
    mountOptions?: pulumi.Input<inputs.datasync.LocationSmbMountOptions>;
    /**
     * The password of the user who can mount the share and has file permissions in the SMB.
     */
    password?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Specifies the IP address or DNS name of the SMB server. The DataSync Agent(s) use this to mount the SMB share.
     */
    serverHostname?: pulumi.Input<string>;
    /**
     * Subdirectory to perform actions as source or destination. Should be exported by the NFS server.
     */
    subdirectory?: pulumi.Input<string>;
    /**
     * Key-value pairs of resource tags to assign to the DataSync Location. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
    /**
     * The user who can mount the share and has file and folder permissions in the SMB share.
     */
    user?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a LocationSmb resource.
 */
export interface LocationSmbArgs {
    /**
     * A list of DataSync Agent ARNs with which this location will be associated.
     */
    agentArns: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The name of the Windows domain the SMB server belongs to.
     */
    domain?: pulumi.Input<string>;
    /**
     * Configuration block containing mount options used by DataSync to access the SMB Server. Can be `AUTOMATIC`, `SMB2`, or `SMB3`.
     */
    mountOptions?: pulumi.Input<inputs.datasync.LocationSmbMountOptions>;
    /**
     * The password of the user who can mount the share and has file permissions in the SMB.
     */
    password: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Specifies the IP address or DNS name of the SMB server. The DataSync Agent(s) use this to mount the SMB share.
     */
    serverHostname: pulumi.Input<string>;
    /**
     * Subdirectory to perform actions as source or destination. Should be exported by the NFS server.
     */
    subdirectory: pulumi.Input<string>;
    /**
     * Key-value pairs of resource tags to assign to the DataSync Location. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The user who can mount the share and has file and folder permissions in the SMB share.
     */
    user: pulumi.Input<string>;
}
