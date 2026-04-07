import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS User Notifications Notification Hub.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.notifications.NotificationHub("example", {notificationHubRegion: "us-west-2"});
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import User Notifications Notification Hub using the `notification_hub_region`. For example:
 *
 * ```sh
 * $ pulumi import aws:notifications/notificationHub:NotificationHub example us-west-2
 * ```
 */
export declare class NotificationHub extends pulumi.CustomResource {
    /**
     * Get an existing NotificationHub resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: NotificationHubState, opts?: pulumi.CustomResourceOptions): NotificationHub;
    /**
     * Returns true if the given object is an instance of NotificationHub.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is NotificationHub;
    /**
     * Notification Hub region.
     */
    readonly notificationHubRegion: pulumi.Output<string>;
    readonly timeouts: pulumi.Output<outputs.notifications.NotificationHubTimeouts | undefined>;
    /**
     * Create a NotificationHub resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: NotificationHubArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering NotificationHub resources.
 */
export interface NotificationHubState {
    /**
     * Notification Hub region.
     */
    notificationHubRegion?: pulumi.Input<string>;
    timeouts?: pulumi.Input<inputs.notifications.NotificationHubTimeouts>;
}
/**
 * The set of arguments for constructing a NotificationHub resource.
 */
export interface NotificationHubArgs {
    /**
     * Notification Hub region.
     */
    notificationHubRegion: pulumi.Input<string>;
    timeouts?: pulumi.Input<inputs.notifications.NotificationHubTimeouts>;
}
