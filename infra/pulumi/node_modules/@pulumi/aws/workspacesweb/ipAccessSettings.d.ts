import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS WorkSpaces Web IP Access Settings resource. Once associated with a web portal, IP access settings control which IP addresses users can connect from.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.workspacesweb.IpAccessSettings("example", {
 *     displayName: "example",
 *     ipRules: [{
 *         ipRange: "10.0.0.0/16",
 *     }],
 * });
 * ```
 *
 * ### With Multiple IP Rules
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.workspacesweb.IpAccessSettings("example", {
 *     displayName: "example",
 *     description: "Example IP access settings",
 *     ipRules: [
 *         {
 *             ipRange: "10.0.0.0/16",
 *             description: "Main office",
 *         },
 *         {
 *             ipRange: "192.168.0.0/24",
 *             description: "Branch office",
 *         },
 *     ],
 * });
 * ```
 *
 * ### With All Arguments
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.kms.Key("example", {
 *     description: "KMS key for WorkSpaces Web IP Access Settings",
 *     deletionWindowInDays: 7,
 * });
 * const exampleIpAccessSettings = new aws.workspacesweb.IpAccessSettings("example", {
 *     displayName: "example",
 *     description: "Example IP access settings",
 *     customerManagedKey: example.arn,
 *     additionalEncryptionContext: {
 *         Environment: "Production",
 *     },
 *     ipRules: [
 *         {
 *             ipRange: "10.0.0.0/16",
 *             description: "Main office",
 *         },
 *         {
 *             ipRange: "192.168.0.0/24",
 *             description: "Branch office",
 *         },
 *     ],
 *     tags: {
 *         Name: "example-ip-access-settings",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import WorkSpaces Web IP Access Settings using the `ip_access_settings_arn`. For example:
 *
 * ```sh
 * $ pulumi import aws:workspacesweb/ipAccessSettings:IpAccessSettings example arn:aws:workspaces-web:us-west-2:123456789012:ipAccessSettings/abcdef12345
 * ```
 */
export declare class IpAccessSettings extends pulumi.CustomResource {
    /**
     * Get an existing IpAccessSettings resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: IpAccessSettingsState, opts?: pulumi.CustomResourceOptions): IpAccessSettings;
    /**
     * Returns true if the given object is an instance of IpAccessSettings.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is IpAccessSettings;
    /**
     * Additional encryption context for the IP access settings.
     */
    readonly additionalEncryptionContext: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * List of web portal ARNs that this IP access settings resource is associated with.
     */
    readonly associatedPortalArns: pulumi.Output<string[]>;
    /**
     * ARN of the customer managed KMS key.
     */
    readonly customerManagedKey: pulumi.Output<string | undefined>;
    /**
     * The description of the IP access settings.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * The display name of the IP access settings.
     */
    readonly displayName: pulumi.Output<string>;
    /**
     * ARN of the IP access settings resource.
     */
    readonly ipAccessSettingsArn: pulumi.Output<string>;
    /**
     * The IP rules of the IP access settings. See IP Rule below.
     *
     * The following arguments are optional:
     */
    readonly ipRules: pulumi.Output<outputs.workspacesweb.IpAccessSettingsIpRule[] | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Map of tags assigned to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Create a IpAccessSettings resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: IpAccessSettingsArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering IpAccessSettings resources.
 */
export interface IpAccessSettingsState {
    /**
     * Additional encryption context for the IP access settings.
     */
    additionalEncryptionContext?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * List of web portal ARNs that this IP access settings resource is associated with.
     */
    associatedPortalArns?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * ARN of the customer managed KMS key.
     */
    customerManagedKey?: pulumi.Input<string>;
    /**
     * The description of the IP access settings.
     */
    description?: pulumi.Input<string>;
    /**
     * The display name of the IP access settings.
     */
    displayName?: pulumi.Input<string>;
    /**
     * ARN of the IP access settings resource.
     */
    ipAccessSettingsArn?: pulumi.Input<string>;
    /**
     * The IP rules of the IP access settings. See IP Rule below.
     *
     * The following arguments are optional:
     */
    ipRules?: pulumi.Input<pulumi.Input<inputs.workspacesweb.IpAccessSettingsIpRule>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Map of tags assigned to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
 * The set of arguments for constructing a IpAccessSettings resource.
 */
export interface IpAccessSettingsArgs {
    /**
     * Additional encryption context for the IP access settings.
     */
    additionalEncryptionContext?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * ARN of the customer managed KMS key.
     */
    customerManagedKey?: pulumi.Input<string>;
    /**
     * The description of the IP access settings.
     */
    description?: pulumi.Input<string>;
    /**
     * The display name of the IP access settings.
     */
    displayName: pulumi.Input<string>;
    /**
     * The IP rules of the IP access settings. See IP Rule below.
     *
     * The following arguments are optional:
     */
    ipRules?: pulumi.Input<pulumi.Input<inputs.workspacesweb.IpAccessSettingsIpRule>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Map of tags assigned to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
