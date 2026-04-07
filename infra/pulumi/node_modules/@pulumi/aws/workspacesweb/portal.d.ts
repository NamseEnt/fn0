import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS WorkSpaces Web Portal.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.workspacesweb.Portal("example", {
 *     displayName: "example-portal",
 *     instanceType: "standard.regular",
 * });
 * ```
 *
 * ### Complete Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.kms.Key("example", {
 *     description: "KMS key for WorkSpaces Web Portal",
 *     deletionWindowInDays: 7,
 * });
 * const examplePortal = new aws.workspacesweb.Portal("example", {
 *     displayName: "example-portal",
 *     instanceType: "standard.large",
 *     authenticationType: "IAM_Identity_Center",
 *     customerManagedKey: example.arn,
 *     maxConcurrentSessions: 10,
 *     additionalEncryptionContext: {
 *         Environment: "Production",
 *     },
 *     tags: {
 *         Name: "example-portal",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import WorkSpaces Web Portal using the `portal_arn`. For example:
 *
 * ```sh
 * $ pulumi import aws:workspacesweb/portal:Portal example arn:aws:workspaces-web:us-west-2:123456789012:portal/abcdef12345678
 * ```
 */
export declare class Portal extends pulumi.CustomResource {
    /**
     * Get an existing Portal resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: PortalState, opts?: pulumi.CustomResourceOptions): Portal;
    /**
     * Returns true if the given object is an instance of Portal.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Portal;
    /**
     * Additional encryption context for the customer managed key. Forces replacement if changed.
     */
    readonly additionalEncryptionContext: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * Authentication type for the portal. Valid values: `Standard`, `IAM_Identity_Center`.
     */
    readonly authenticationType: pulumi.Output<string>;
    /**
     * ARN of the browser settings to use for the portal.
     */
    readonly browserSettingsArn: pulumi.Output<string>;
    /**
     * Browser type of the portal.
     */
    readonly browserType: pulumi.Output<string>;
    /**
     * Creation date of the portal.
     */
    readonly creationDate: pulumi.Output<string>;
    /**
     * ARN of the customer managed key. Forces replacement if changed.
     */
    readonly customerManagedKey: pulumi.Output<string | undefined>;
    /**
     * ARN of the data protection settings associated with the portal.
     */
    readonly dataProtectionSettingsArn: pulumi.Output<string>;
    /**
     * Display name of the portal.
     */
    readonly displayName: pulumi.Output<string>;
    /**
     * Instance type for the portal. Valid values: `standard.regular`, `standard.large`.
     */
    readonly instanceType: pulumi.Output<string>;
    /**
     * ARN of the IP access settings associated with the portal.
     */
    readonly ipAccessSettingsArn: pulumi.Output<string>;
    /**
     * Maximum number of concurrent sessions for the portal.
     */
    readonly maxConcurrentSessions: pulumi.Output<number>;
    /**
     * ARN of the network settings associated with the portal.
     */
    readonly networkSettingsArn: pulumi.Output<string>;
    /**
     * ARN of the portal.
     */
    readonly portalArn: pulumi.Output<string>;
    /**
     * Endpoint URL of the portal.
     */
    readonly portalEndpoint: pulumi.Output<string>;
    /**
     * Status of the portal.
     */
    readonly portalStatus: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Renderer type of the portal.
     */
    readonly rendererType: pulumi.Output<string>;
    /**
     * ARN of the session logger associated with the portal.
     */
    readonly sessionLoggerArn: pulumi.Output<string>;
    /**
     * Reason for the current status of the portal.
     */
    readonly statusReason: pulumi.Output<string>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
    readonly timeouts: pulumi.Output<outputs.workspacesweb.PortalTimeouts | undefined>;
    /**
     * ARN of the trust store associated with the portal.
     */
    readonly trustStoreArn: pulumi.Output<string>;
    /**
     * ARN of the user access logging settings associated with the portal.
     */
    readonly userAccessLoggingSettingsArn: pulumi.Output<string>;
    /**
     * ARN of the user settings associated with the portal.
     */
    readonly userSettingsArn: pulumi.Output<string>;
    /**
     * Create a Portal resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: PortalArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Portal resources.
 */
export interface PortalState {
    /**
     * Additional encryption context for the customer managed key. Forces replacement if changed.
     */
    additionalEncryptionContext?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Authentication type for the portal. Valid values: `Standard`, `IAM_Identity_Center`.
     */
    authenticationType?: pulumi.Input<string>;
    /**
     * ARN of the browser settings to use for the portal.
     */
    browserSettingsArn?: pulumi.Input<string>;
    /**
     * Browser type of the portal.
     */
    browserType?: pulumi.Input<string>;
    /**
     * Creation date of the portal.
     */
    creationDate?: pulumi.Input<string>;
    /**
     * ARN of the customer managed key. Forces replacement if changed.
     */
    customerManagedKey?: pulumi.Input<string>;
    /**
     * ARN of the data protection settings associated with the portal.
     */
    dataProtectionSettingsArn?: pulumi.Input<string>;
    /**
     * Display name of the portal.
     */
    displayName?: pulumi.Input<string>;
    /**
     * Instance type for the portal. Valid values: `standard.regular`, `standard.large`.
     */
    instanceType?: pulumi.Input<string>;
    /**
     * ARN of the IP access settings associated with the portal.
     */
    ipAccessSettingsArn?: pulumi.Input<string>;
    /**
     * Maximum number of concurrent sessions for the portal.
     */
    maxConcurrentSessions?: pulumi.Input<number>;
    /**
     * ARN of the network settings associated with the portal.
     */
    networkSettingsArn?: pulumi.Input<string>;
    /**
     * ARN of the portal.
     */
    portalArn?: pulumi.Input<string>;
    /**
     * Endpoint URL of the portal.
     */
    portalEndpoint?: pulumi.Input<string>;
    /**
     * Status of the portal.
     */
    portalStatus?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Renderer type of the portal.
     */
    rendererType?: pulumi.Input<string>;
    /**
     * ARN of the session logger associated with the portal.
     */
    sessionLoggerArn?: pulumi.Input<string>;
    /**
     * Reason for the current status of the portal.
     */
    statusReason?: pulumi.Input<string>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
    timeouts?: pulumi.Input<inputs.workspacesweb.PortalTimeouts>;
    /**
     * ARN of the trust store associated with the portal.
     */
    trustStoreArn?: pulumi.Input<string>;
    /**
     * ARN of the user access logging settings associated with the portal.
     */
    userAccessLoggingSettingsArn?: pulumi.Input<string>;
    /**
     * ARN of the user settings associated with the portal.
     */
    userSettingsArn?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a Portal resource.
 */
export interface PortalArgs {
    /**
     * Additional encryption context for the customer managed key. Forces replacement if changed.
     */
    additionalEncryptionContext?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Authentication type for the portal. Valid values: `Standard`, `IAM_Identity_Center`.
     */
    authenticationType?: pulumi.Input<string>;
    /**
     * ARN of the browser settings to use for the portal.
     */
    browserSettingsArn?: pulumi.Input<string>;
    /**
     * ARN of the customer managed key. Forces replacement if changed.
     */
    customerManagedKey?: pulumi.Input<string>;
    /**
     * Display name of the portal.
     */
    displayName?: pulumi.Input<string>;
    /**
     * Instance type for the portal. Valid values: `standard.regular`, `standard.large`.
     */
    instanceType?: pulumi.Input<string>;
    /**
     * Maximum number of concurrent sessions for the portal.
     */
    maxConcurrentSessions?: pulumi.Input<number>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    timeouts?: pulumi.Input<inputs.workspacesweb.PortalTimeouts>;
}
