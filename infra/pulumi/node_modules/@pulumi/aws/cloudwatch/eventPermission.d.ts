import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a resource to create an EventBridge permission to support cross-account events in the current account default event bus.
 *
 * > **Note:** EventBridge was formerly known as CloudWatch Events. The functionality is identical.
 *
 * > **Note:** The EventBridge bus policy resource  (`aws.cloudwatch.EventBusPolicy`) is incompatible with the EventBridge permission resource (`aws.cloudwatch.EventPermission`) and will overwrite permissions.
 *
 * ## Example Usage
 *
 * ### Account Access
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const devAccountAccess = new aws.cloudwatch.EventPermission("DevAccountAccess", {
 *     principal: "123456789012",
 *     statementId: "DevAccountAccess",
 * });
 * ```
 *
 * ### Organization Access
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const organizationAccess = new aws.cloudwatch.EventPermission("OrganizationAccess", {
 *     principal: "*",
 *     statementId: "OrganizationAccess",
 *     condition: {
 *         key: "aws:PrincipalOrgID",
 *         type: "StringEquals",
 *         value: example.id,
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import EventBridge permissions using the `event_bus_name/statement_id` (if you omit `event_bus_name`, the `default` event bus will be used). For example:
 *
 * ```sh
 * $ pulumi import aws:cloudwatch/eventPermission:EventPermission DevAccountAccess example-event-bus/DevAccountAccess
 * ```
 */
export declare class EventPermission extends pulumi.CustomResource {
    /**
     * Get an existing EventPermission resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: EventPermissionState, opts?: pulumi.CustomResourceOptions): EventPermission;
    /**
     * Returns true if the given object is an instance of EventPermission.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is EventPermission;
    /**
     * The action that you are enabling the other account to perform. Defaults to `events:PutEvents`.
     */
    readonly action: pulumi.Output<string | undefined>;
    /**
     * Configuration block to limit the event bus permissions you are granting to only accounts that fulfill the condition. Specified below.
     */
    readonly condition: pulumi.Output<outputs.cloudwatch.EventPermissionCondition | undefined>;
    /**
     * The name of the event bus to set the permissions on.
     * If you omit this, the permissions are set on the `default` event bus.
     */
    readonly eventBusName: pulumi.Output<string | undefined>;
    /**
     * The 12-digit AWS account ID that you are permitting to put events to your default event bus. Specify `*` to permit any account to put events to your default event bus, optionally limited by `condition`.
     */
    readonly principal: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * An identifier string for the external account that you are granting permissions to.
     */
    readonly statementId: pulumi.Output<string>;
    /**
     * Create a EventPermission resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: EventPermissionArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering EventPermission resources.
 */
export interface EventPermissionState {
    /**
     * The action that you are enabling the other account to perform. Defaults to `events:PutEvents`.
     */
    action?: pulumi.Input<string>;
    /**
     * Configuration block to limit the event bus permissions you are granting to only accounts that fulfill the condition. Specified below.
     */
    condition?: pulumi.Input<inputs.cloudwatch.EventPermissionCondition>;
    /**
     * The name of the event bus to set the permissions on.
     * If you omit this, the permissions are set on the `default` event bus.
     */
    eventBusName?: pulumi.Input<string>;
    /**
     * The 12-digit AWS account ID that you are permitting to put events to your default event bus. Specify `*` to permit any account to put events to your default event bus, optionally limited by `condition`.
     */
    principal?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * An identifier string for the external account that you are granting permissions to.
     */
    statementId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a EventPermission resource.
 */
export interface EventPermissionArgs {
    /**
     * The action that you are enabling the other account to perform. Defaults to `events:PutEvents`.
     */
    action?: pulumi.Input<string>;
    /**
     * Configuration block to limit the event bus permissions you are granting to only accounts that fulfill the condition. Specified below.
     */
    condition?: pulumi.Input<inputs.cloudwatch.EventPermissionCondition>;
    /**
     * The name of the event bus to set the permissions on.
     * If you omit this, the permissions are set on the `default` event bus.
     */
    eventBusName?: pulumi.Input<string>;
    /**
     * The 12-digit AWS account ID that you are permitting to put events to your default event bus. Specify `*` to permit any account to put events to your default event bus, optionally limited by `condition`.
     */
    principal: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * An identifier string for the external account that you are granting permissions to.
     */
    statementId: pulumi.Input<string>;
}
