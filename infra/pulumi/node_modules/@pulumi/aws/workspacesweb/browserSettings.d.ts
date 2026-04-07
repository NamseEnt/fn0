import * as pulumi from "@pulumi/pulumi";
/**
 * Resource for managing an AWS WorkSpaces Web Browser Settings resource.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.workspacesweb.BrowserSettings("example", {browserPolicy: JSON.stringify({
 *     AdditionalSettings: {
 *         DownloadsSettings: {
 *             Behavior: "DISABLE",
 *         },
 *     },
 * })});
 * ```
 *
 * ### With All Arguments
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.kms.Key("example", {
 *     description: "KMS key for WorkSpaces Web Browser Settings",
 *     deletionWindowInDays: 7,
 * });
 * const exampleBrowserSettings = new aws.workspacesweb.BrowserSettings("example", {
 *     browserPolicy: JSON.stringify({
 *         chromePolicies: {
 *             DefaultDownloadDirectory: {
 *                 value: "/home/as2-streaming-user/MyFiles/TemporaryFiles1",
 *             },
 *         },
 *     }),
 *     customerManagedKey: example.arn,
 *     additionalEncryptionContext: {
 *         Environment: "Production",
 *     },
 *     tags: {
 *         Name: "example-browser-settings",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import WorkSpaces Web Browser Settings using the `browser_settings_arn`. For example:
 *
 * ```sh
 * $ pulumi import aws:workspacesweb/browserSettings:BrowserSettings example arn:aws:workspacesweb:us-west-2:123456789012:browsersettings/abcdef12345
 * ```
 */
export declare class BrowserSettings extends pulumi.CustomResource {
    /**
     * Get an existing BrowserSettings resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: BrowserSettingsState, opts?: pulumi.CustomResourceOptions): BrowserSettings;
    /**
     * Returns true if the given object is an instance of BrowserSettings.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is BrowserSettings;
    /**
     * Additional encryption context for the browser settings.
     */
    readonly additionalEncryptionContext: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * List of web portal ARNs to associate with the browser settings.
     */
    readonly associatedPortalArns: pulumi.Output<string[]>;
    /**
     * Browser policy for the browser settings. This is a JSON string that defines the browser settings policy.
     *
     * The following arguments are optional:
     */
    readonly browserPolicy: pulumi.Output<string>;
    /**
     * ARN of the browser settings resource.
     */
    readonly browserSettingsArn: pulumi.Output<string>;
    /**
     * ARN of the customer managed KMS key.
     */
    readonly customerManagedKey: pulumi.Output<string | undefined>;
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
     * Create a BrowserSettings resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: BrowserSettingsArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering BrowserSettings resources.
 */
export interface BrowserSettingsState {
    /**
     * Additional encryption context for the browser settings.
     */
    additionalEncryptionContext?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * List of web portal ARNs to associate with the browser settings.
     */
    associatedPortalArns?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Browser policy for the browser settings. This is a JSON string that defines the browser settings policy.
     *
     * The following arguments are optional:
     */
    browserPolicy?: pulumi.Input<string>;
    /**
     * ARN of the browser settings resource.
     */
    browserSettingsArn?: pulumi.Input<string>;
    /**
     * ARN of the customer managed KMS key.
     */
    customerManagedKey?: pulumi.Input<string>;
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
 * The set of arguments for constructing a BrowserSettings resource.
 */
export interface BrowserSettingsArgs {
    /**
     * Additional encryption context for the browser settings.
     */
    additionalEncryptionContext?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Browser policy for the browser settings. This is a JSON string that defines the browser settings policy.
     *
     * The following arguments are optional:
     */
    browserPolicy: pulumi.Input<string>;
    /**
     * ARN of the customer managed KMS key.
     */
    customerManagedKey?: pulumi.Input<string>;
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
