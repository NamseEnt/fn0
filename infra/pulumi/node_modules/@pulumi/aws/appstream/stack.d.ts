import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides an AppStream stack.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.appstream.Stack("example", {
 *     name: "stack name",
 *     description: "stack description",
 *     displayName: "stack display name",
 *     feedbackUrl: "http://your-domain/feedback",
 *     redirectUrl: "http://your-domain/redirect",
 *     storageConnectors: [{
 *         connectorType: "HOMEFOLDERS",
 *     }],
 *     userSettings: [
 *         {
 *             action: "AUTO_TIME_ZONE_REDIRECTION",
 *             permission: "DISABLED",
 *         },
 *         {
 *             action: "CLIPBOARD_COPY_FROM_LOCAL_DEVICE",
 *             permission: "ENABLED",
 *         },
 *         {
 *             action: "CLIPBOARD_COPY_TO_LOCAL_DEVICE",
 *             permission: "ENABLED",
 *         },
 *         {
 *             action: "DOMAIN_PASSWORD_SIGNIN",
 *             permission: "ENABLED",
 *         },
 *         {
 *             action: "DOMAIN_SMART_CARD_SIGNIN",
 *             permission: "DISABLED",
 *         },
 *         {
 *             action: "FILE_DOWNLOAD",
 *             permission: "ENABLED",
 *         },
 *         {
 *             action: "FILE_UPLOAD",
 *             permission: "ENABLED",
 *         },
 *         {
 *             action: "PRINTING_TO_LOCAL_DEVICE",
 *             permission: "ENABLED",
 *         },
 *     ],
 *     applicationSettings: {
 *         enabled: true,
 *         settingsGroup: "SettingsGroup",
 *     },
 *     tags: {
 *         TagName: "TagValue",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_appstream_stack` using the id. For example:
 *
 * ```sh
 * $ pulumi import aws:appstream/stack:Stack example stackID
 * ```
 */
export declare class Stack extends pulumi.CustomResource {
    /**
     * Get an existing Stack resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: StackState, opts?: pulumi.CustomResourceOptions): Stack;
    /**
     * Returns true if the given object is an instance of Stack.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Stack;
    /**
     * Set of configuration blocks defining the interface VPC endpoints. Users of the stack can connect to AppStream 2.0 only through the specified endpoints.
     * See `accessEndpoints` below.
     */
    readonly accessEndpoints: pulumi.Output<outputs.appstream.StackAccessEndpoint[]>;
    /**
     * Settings for application settings persistence.
     * See `applicationSettings` below.
     */
    readonly applicationSettings: pulumi.Output<outputs.appstream.StackApplicationSettings>;
    /**
     * ARN of the appstream stack.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Date and time, in UTC and extended RFC 3339 format, when the stack was created.
     */
    readonly createdTime: pulumi.Output<string>;
    /**
     * Description for the AppStream stack.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * Stack name to display.
     */
    readonly displayName: pulumi.Output<string | undefined>;
    /**
     * Domains where AppStream 2.0 streaming sessions can be embedded in an iframe. You must approve the domains that you want to host embedded AppStream 2.0 streaming sessions.
     */
    readonly embedHostDomains: pulumi.Output<string[]>;
    /**
     * URL that users are redirected to after they click the Send Feedback link. If no URL is specified, no Send Feedback link is displayed. .
     */
    readonly feedbackUrl: pulumi.Output<string>;
    /**
     * Unique name for the AppStream stack.
     *
     * The following arguments are optional:
     */
    readonly name: pulumi.Output<string>;
    /**
     * URL that users are redirected to after their streaming session ends.
     */
    readonly redirectUrl: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Configuration block for the storage connectors to enable.
     * See `storageConnectors` below.
     */
    readonly storageConnectors: pulumi.Output<outputs.appstream.StackStorageConnector[]>;
    /**
     * The streaming protocol you want your stack to prefer. This can be UDP or TCP. Currently, UDP is only supported in the Windows native client.
     * See `streamingExperienceSettings` below.
     */
    readonly streamingExperienceSettings: pulumi.Output<outputs.appstream.StackStreamingExperienceSettings>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * Configuration block for the actions that are enabled or disabled for users during their streaming sessions. If not provided, these settings are configured automatically by AWS. If provided, the configuration should include a block for each configurable action.
     * See `userSettings` below.
     */
    readonly userSettings: pulumi.Output<outputs.appstream.StackUserSetting[]>;
    /**
     * Create a Stack resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: StackArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Stack resources.
 */
export interface StackState {
    /**
     * Set of configuration blocks defining the interface VPC endpoints. Users of the stack can connect to AppStream 2.0 only through the specified endpoints.
     * See `accessEndpoints` below.
     */
    accessEndpoints?: pulumi.Input<pulumi.Input<inputs.appstream.StackAccessEndpoint>[]>;
    /**
     * Settings for application settings persistence.
     * See `applicationSettings` below.
     */
    applicationSettings?: pulumi.Input<inputs.appstream.StackApplicationSettings>;
    /**
     * ARN of the appstream stack.
     */
    arn?: pulumi.Input<string>;
    /**
     * Date and time, in UTC and extended RFC 3339 format, when the stack was created.
     */
    createdTime?: pulumi.Input<string>;
    /**
     * Description for the AppStream stack.
     */
    description?: pulumi.Input<string>;
    /**
     * Stack name to display.
     */
    displayName?: pulumi.Input<string>;
    /**
     * Domains where AppStream 2.0 streaming sessions can be embedded in an iframe. You must approve the domains that you want to host embedded AppStream 2.0 streaming sessions.
     */
    embedHostDomains?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * URL that users are redirected to after they click the Send Feedback link. If no URL is specified, no Send Feedback link is displayed. .
     */
    feedbackUrl?: pulumi.Input<string>;
    /**
     * Unique name for the AppStream stack.
     *
     * The following arguments are optional:
     */
    name?: pulumi.Input<string>;
    /**
     * URL that users are redirected to after their streaming session ends.
     */
    redirectUrl?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Configuration block for the storage connectors to enable.
     * See `storageConnectors` below.
     */
    storageConnectors?: pulumi.Input<pulumi.Input<inputs.appstream.StackStorageConnector>[]>;
    /**
     * The streaming protocol you want your stack to prefer. This can be UDP or TCP. Currently, UDP is only supported in the Windows native client.
     * See `streamingExperienceSettings` below.
     */
    streamingExperienceSettings?: pulumi.Input<inputs.appstream.StackStreamingExperienceSettings>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Configuration block for the actions that are enabled or disabled for users during their streaming sessions. If not provided, these settings are configured automatically by AWS. If provided, the configuration should include a block for each configurable action.
     * See `userSettings` below.
     */
    userSettings?: pulumi.Input<pulumi.Input<inputs.appstream.StackUserSetting>[]>;
}
/**
 * The set of arguments for constructing a Stack resource.
 */
export interface StackArgs {
    /**
     * Set of configuration blocks defining the interface VPC endpoints. Users of the stack can connect to AppStream 2.0 only through the specified endpoints.
     * See `accessEndpoints` below.
     */
    accessEndpoints?: pulumi.Input<pulumi.Input<inputs.appstream.StackAccessEndpoint>[]>;
    /**
     * Settings for application settings persistence.
     * See `applicationSettings` below.
     */
    applicationSettings?: pulumi.Input<inputs.appstream.StackApplicationSettings>;
    /**
     * Description for the AppStream stack.
     */
    description?: pulumi.Input<string>;
    /**
     * Stack name to display.
     */
    displayName?: pulumi.Input<string>;
    /**
     * Domains where AppStream 2.0 streaming sessions can be embedded in an iframe. You must approve the domains that you want to host embedded AppStream 2.0 streaming sessions.
     */
    embedHostDomains?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * URL that users are redirected to after they click the Send Feedback link. If no URL is specified, no Send Feedback link is displayed. .
     */
    feedbackUrl?: pulumi.Input<string>;
    /**
     * Unique name for the AppStream stack.
     *
     * The following arguments are optional:
     */
    name?: pulumi.Input<string>;
    /**
     * URL that users are redirected to after their streaming session ends.
     */
    redirectUrl?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Configuration block for the storage connectors to enable.
     * See `storageConnectors` below.
     */
    storageConnectors?: pulumi.Input<pulumi.Input<inputs.appstream.StackStorageConnector>[]>;
    /**
     * The streaming protocol you want your stack to prefer. This can be UDP or TCP. Currently, UDP is only supported in the Windows native client.
     * See `streamingExperienceSettings` below.
     */
    streamingExperienceSettings?: pulumi.Input<inputs.appstream.StackStreamingExperienceSettings>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Configuration block for the actions that are enabled or disabled for users during their streaming sessions. If not provided, these settings are configured automatically by AWS. If provided, the configuration should include a block for each configurable action.
     * See `userSettings` below.
     */
    userSettings?: pulumi.Input<pulumi.Input<inputs.appstream.StackUserSetting>[]>;
}
