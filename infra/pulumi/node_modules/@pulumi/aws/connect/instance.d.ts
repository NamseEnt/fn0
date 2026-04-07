import * as pulumi from "@pulumi/pulumi";
/**
 * Provides an Amazon Connect instance resource. For more information see
 * [Amazon Connect: Getting Started](https://docs.aws.amazon.com/connect/latest/adminguide/amazon-connect-get-started.html)
 *
 * !> **WARN:** Amazon Connect enforces a limit of [100 combined instance creation and deletions every 30 days](https://docs.aws.amazon.com/connect/latest/adminguide/amazon-connect-service-limits.html#feature-limits). For example, if you create 80 instances and delete 20 of them, you must wait 30 days to create or delete another instance. Use care when creating or deleting instances.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = new aws.connect.Instance("test", {
 *     identityManagementType: "CONNECT_MANAGED",
 *     inboundCallsEnabled: true,
 *     instanceAlias: "friendly-name-connect",
 *     outboundCallsEnabled: true,
 *     tags: {
 *         hello: "world",
 *     },
 * });
 * ```
 *
 * ### With Existing Active Directory
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = new aws.connect.Instance("test", {
 *     directoryId: testAwsDirectoryServiceDirectory.id,
 *     identityManagementType: "EXISTING_DIRECTORY",
 *     inboundCallsEnabled: true,
 *     instanceAlias: "friendly-name-connect",
 *     outboundCallsEnabled: true,
 * });
 * ```
 *
 * ### With SAML
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = new aws.connect.Instance("test", {
 *     identityManagementType: "SAML",
 *     inboundCallsEnabled: true,
 *     instanceAlias: "friendly-name-connect",
 *     outboundCallsEnabled: true,
 * });
 * ```
 *
 * ## Import
 *
 * ### Identity Schema
 *
 * #### Required
 *
 * * `id` - (String) ID of the connect instance.
 *
 * #### Optional
 *
 * - `account_id` (String) AWS Account where this resource is managed.
 *
 * - `region` (String) Region where this resource is managed.
 *
 * Using `pulumi import`, import Connect instances using the `id`. For example:
 *
 * console
 *
 * % pulumi import aws_connect_instance.example f1288a1f-6193-445a-b47e-af739b2
 */
export declare class Instance extends pulumi.CustomResource {
    /**
     * Get an existing Instance resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: InstanceState, opts?: pulumi.CustomResourceOptions): Instance;
    /**
     * Returns true if the given object is an instance of Instance.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Instance;
    /**
     * Amazon Resource Name (ARN) of the instance.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Specifies whether auto resolve best voices is enabled. Defaults to `true`.
     */
    readonly autoResolveBestVoicesEnabled: pulumi.Output<boolean | undefined>;
    /**
     * Specifies whether contact flow logs are enabled. Defaults to `false`.
     */
    readonly contactFlowLogsEnabled: pulumi.Output<boolean | undefined>;
    /**
     * Specifies whether contact lens is enabled. Defaults to `true`.
     */
    readonly contactLensEnabled: pulumi.Output<boolean | undefined>;
    /**
     * When the instance was created.
     */
    readonly createdTime: pulumi.Output<string>;
    /**
     * The identifier for the directory if identityManagementType is `EXISTING_DIRECTORY`.
     */
    readonly directoryId: pulumi.Output<string | undefined>;
    /**
     * Specifies whether early media for outbound calls is enabled . Defaults to `true` if outbound calls is enabled.
     */
    readonly earlyMediaEnabled: pulumi.Output<boolean | undefined>;
    /**
     * Specifies the identity management type attached to the instance. Allowed Values are: `SAML`, `CONNECT_MANAGED`, `EXISTING_DIRECTORY`.
     */
    readonly identityManagementType: pulumi.Output<string>;
    /**
     * Specifies whether inbound calls are enabled.
     */
    readonly inboundCallsEnabled: pulumi.Output<boolean>;
    /**
     * Specifies the name of the instance. Required if `directoryId` not specified.
     */
    readonly instanceAlias: pulumi.Output<string | undefined>;
    /**
     * Specifies whether multi-party calls/conference is enabled. Defaults to `false`.
     */
    readonly multiPartyConferenceEnabled: pulumi.Output<boolean | undefined>;
    /**
     * Specifies whether outbound calls are enabled.
     */
    readonly outboundCallsEnabled: pulumi.Output<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The service role of the instance.
     */
    readonly serviceRole: pulumi.Output<string>;
    /**
     * The state of the instance.
     */
    readonly status: pulumi.Output<string>;
    /**
     * Tags to apply to the Instance. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     * <!-- * `useCustomTtsVoices` - (Optional) Whether use custom tts voices is enabled. Defaults to `false` -->
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
     * Create a Instance resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: InstanceArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Instance resources.
 */
export interface InstanceState {
    /**
     * Amazon Resource Name (ARN) of the instance.
     */
    arn?: pulumi.Input<string>;
    /**
     * Specifies whether auto resolve best voices is enabled. Defaults to `true`.
     */
    autoResolveBestVoicesEnabled?: pulumi.Input<boolean>;
    /**
     * Specifies whether contact flow logs are enabled. Defaults to `false`.
     */
    contactFlowLogsEnabled?: pulumi.Input<boolean>;
    /**
     * Specifies whether contact lens is enabled. Defaults to `true`.
     */
    contactLensEnabled?: pulumi.Input<boolean>;
    /**
     * When the instance was created.
     */
    createdTime?: pulumi.Input<string>;
    /**
     * The identifier for the directory if identityManagementType is `EXISTING_DIRECTORY`.
     */
    directoryId?: pulumi.Input<string>;
    /**
     * Specifies whether early media for outbound calls is enabled . Defaults to `true` if outbound calls is enabled.
     */
    earlyMediaEnabled?: pulumi.Input<boolean>;
    /**
     * Specifies the identity management type attached to the instance. Allowed Values are: `SAML`, `CONNECT_MANAGED`, `EXISTING_DIRECTORY`.
     */
    identityManagementType?: pulumi.Input<string>;
    /**
     * Specifies whether inbound calls are enabled.
     */
    inboundCallsEnabled?: pulumi.Input<boolean>;
    /**
     * Specifies the name of the instance. Required if `directoryId` not specified.
     */
    instanceAlias?: pulumi.Input<string>;
    /**
     * Specifies whether multi-party calls/conference is enabled. Defaults to `false`.
     */
    multiPartyConferenceEnabled?: pulumi.Input<boolean>;
    /**
     * Specifies whether outbound calls are enabled.
     */
    outboundCallsEnabled?: pulumi.Input<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The service role of the instance.
     */
    serviceRole?: pulumi.Input<string>;
    /**
     * The state of the instance.
     */
    status?: pulumi.Input<string>;
    /**
     * Tags to apply to the Instance. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     * <!-- * `useCustomTtsVoices` - (Optional) Whether use custom tts voices is enabled. Defaults to `false` -->
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
 * The set of arguments for constructing a Instance resource.
 */
export interface InstanceArgs {
    /**
     * Specifies whether auto resolve best voices is enabled. Defaults to `true`.
     */
    autoResolveBestVoicesEnabled?: pulumi.Input<boolean>;
    /**
     * Specifies whether contact flow logs are enabled. Defaults to `false`.
     */
    contactFlowLogsEnabled?: pulumi.Input<boolean>;
    /**
     * Specifies whether contact lens is enabled. Defaults to `true`.
     */
    contactLensEnabled?: pulumi.Input<boolean>;
    /**
     * The identifier for the directory if identityManagementType is `EXISTING_DIRECTORY`.
     */
    directoryId?: pulumi.Input<string>;
    /**
     * Specifies whether early media for outbound calls is enabled . Defaults to `true` if outbound calls is enabled.
     */
    earlyMediaEnabled?: pulumi.Input<boolean>;
    /**
     * Specifies the identity management type attached to the instance. Allowed Values are: `SAML`, `CONNECT_MANAGED`, `EXISTING_DIRECTORY`.
     */
    identityManagementType: pulumi.Input<string>;
    /**
     * Specifies whether inbound calls are enabled.
     */
    inboundCallsEnabled: pulumi.Input<boolean>;
    /**
     * Specifies the name of the instance. Required if `directoryId` not specified.
     */
    instanceAlias?: pulumi.Input<string>;
    /**
     * Specifies whether multi-party calls/conference is enabled. Defaults to `false`.
     */
    multiPartyConferenceEnabled?: pulumi.Input<boolean>;
    /**
     * Specifies whether outbound calls are enabled.
     */
    outboundCallsEnabled: pulumi.Input<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Tags to apply to the Instance. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     * <!-- * `useCustomTtsVoices` - (Optional) Whether use custom tts voices is enabled. Defaults to `false` -->
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
