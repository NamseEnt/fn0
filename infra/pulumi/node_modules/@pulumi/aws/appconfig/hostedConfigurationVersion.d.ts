import * as pulumi from "@pulumi/pulumi";
/**
 * Provides an AppConfig Hosted Configuration Version resource.
 *
 * ## Example Usage
 *
 * ### Freeform
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.appconfig.HostedConfigurationVersion("example", {
 *     applicationId: exampleAwsAppconfigApplication.id,
 *     configurationProfileId: exampleAwsAppconfigConfigurationProfile.configurationProfileId,
 *     description: "Example Freeform Hosted Configuration Version",
 *     contentType: "application/json",
 *     content: JSON.stringify({
 *         foo: "bar",
 *         fruit: [
 *             "apple",
 *             "pear",
 *             "orange",
 *         ],
 *         isThingEnabled: true,
 *     }),
 * });
 * ```
 *
 * ### Feature Flags
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.appconfig.HostedConfigurationVersion("example", {
 *     applicationId: exampleAwsAppconfigApplication.id,
 *     configurationProfileId: exampleAwsAppconfigConfigurationProfile.configurationProfileId,
 *     description: "Example Feature Flag Configuration Version",
 *     contentType: "application/json",
 *     content: JSON.stringify({
 *         flags: {
 *             foo: {
 *                 name: "foo",
 *                 _deprecation: {
 *                     status: "planned",
 *                 },
 *             },
 *             bar: {
 *                 name: "bar",
 *                 attributes: {
 *                     someAttribute: {
 *                         constraints: {
 *                             type: "string",
 *                             required: true,
 *                         },
 *                     },
 *                     someOtherAttribute: {
 *                         constraints: {
 *                             type: "number",
 *                             required: true,
 *                         },
 *                     },
 *                 },
 *             },
 *         },
 *         values: {
 *             foo: {
 *                 enabled: "true",
 *             },
 *             bar: {
 *                 enabled: "true",
 *                 someAttribute: "Hello World",
 *                 someOtherAttribute: 123,
 *             },
 *         },
 *         version: "1",
 *     }),
 * });
 * ```
 *
 * ### Multi-variant Feature Flags
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 * import * as std from "@pulumi/std";
 *
 * const example = new aws.appconfig.HostedConfigurationVersion("example", {
 *     applicationId: exampleAwsAppconfigApplication.id,
 *     configurationProfileId: exampleAwsAppconfigConfigurationProfile.configurationProfileId,
 *     description: "Example Multi-variant Feature Flag Configuration Version",
 *     contentType: "application/json",
 *     content: JSON.stringify({
 *         flags: {
 *             loggingenabled: {
 *                 name: "loggingEnabled",
 *             },
 *         },
 *         values: {
 *             loggingenabled: {
 *                 _variants: std.concat({
 *                     input: [
 *                         .map(userId => ({
 *                             enabled: true,
 *                             name: `usersWithLoggingEnabled_${userId}`,
 *                             rule: `(or (eq $userId "${userId}"))`,
 *                         })),
 *                         [{
 *                             enabled: false,
 *                             name: "Default",
 *                         }],
 *                     ],
 *                 }).then(invoke => invoke.result),
 *             },
 *         },
 *         version: "1",
 *     }),
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import AppConfig Hosted Configuration Versions using the application ID, configuration profile ID, and version number separated by a slash (`/`). For example:
 *
 * ```sh
 * $ pulumi import aws:appconfig/hostedConfigurationVersion:HostedConfigurationVersion example 71abcde/11xxxxx/2
 * ```
 */
export declare class HostedConfigurationVersion extends pulumi.CustomResource {
    /**
     * Get an existing HostedConfigurationVersion resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: HostedConfigurationVersionState, opts?: pulumi.CustomResourceOptions): HostedConfigurationVersion;
    /**
     * Returns true if the given object is an instance of HostedConfigurationVersion.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is HostedConfigurationVersion;
    /**
     * Application ID.
     */
    readonly applicationId: pulumi.Output<string>;
    /**
     * ARN of the AppConfig  hosted configuration version.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Configuration profile ID.
     */
    readonly configurationProfileId: pulumi.Output<string>;
    /**
     * Content of the configuration or the configuration data.
     */
    readonly content: pulumi.Output<string>;
    /**
     * Standard MIME type describing the format of the configuration content. For more information, see [Content-Type](https://www.w3.org/Protocols/rfc2616/rfc2616-sec14.html#sec14.17).
     */
    readonly contentType: pulumi.Output<string>;
    /**
     * Description of the configuration.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Version number of the hosted configuration.
     */
    readonly versionNumber: pulumi.Output<number>;
    /**
     * Create a HostedConfigurationVersion resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: HostedConfigurationVersionArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering HostedConfigurationVersion resources.
 */
export interface HostedConfigurationVersionState {
    /**
     * Application ID.
     */
    applicationId?: pulumi.Input<string>;
    /**
     * ARN of the AppConfig  hosted configuration version.
     */
    arn?: pulumi.Input<string>;
    /**
     * Configuration profile ID.
     */
    configurationProfileId?: pulumi.Input<string>;
    /**
     * Content of the configuration or the configuration data.
     */
    content?: pulumi.Input<string>;
    /**
     * Standard MIME type describing the format of the configuration content. For more information, see [Content-Type](https://www.w3.org/Protocols/rfc2616/rfc2616-sec14.html#sec14.17).
     */
    contentType?: pulumi.Input<string>;
    /**
     * Description of the configuration.
     */
    description?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Version number of the hosted configuration.
     */
    versionNumber?: pulumi.Input<number>;
}
/**
 * The set of arguments for constructing a HostedConfigurationVersion resource.
 */
export interface HostedConfigurationVersionArgs {
    /**
     * Application ID.
     */
    applicationId: pulumi.Input<string>;
    /**
     * Configuration profile ID.
     */
    configurationProfileId: pulumi.Input<string>;
    /**
     * Content of the configuration or the configuration data.
     */
    content: pulumi.Input<string>;
    /**
     * Standard MIME type describing the format of the configuration content. For more information, see [Content-Type](https://www.w3.org/Protocols/rfc2616/rfc2616-sec14.html#sec14.17).
     */
    contentType: pulumi.Input<string>;
    /**
     * Description of the configuration.
     */
    description?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
