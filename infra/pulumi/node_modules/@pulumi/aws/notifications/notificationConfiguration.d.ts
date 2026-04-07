import * as pulumi from "@pulumi/pulumi";
/**
 * Resource for managing an AWS User Notifications Notification Configuration.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.notifications.NotificationConfiguration("example", {
 *     name: "example",
 *     description: "Example notification configuration",
 *     tags: {
 *         Environment: "production",
 *         Project: "example",
 *     },
 * });
 * ```
 *
 * ### With Aggregation Duration
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.notifications.NotificationConfiguration("example", {
 *     name: "example-aggregation",
 *     description: "Example notification configuration with aggregation",
 *     aggregationDuration: "SHORT",
 *     tags: {
 *         Environment: "production",
 *         Project: "example",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import User Notifications Notification Configuration using the `arn`. For example:
 *
 * ```sh
 * $ pulumi import aws:notifications/notificationConfiguration:NotificationConfiguration example arn:aws:notifications::123456789012:configuration/abcdef1234567890abcdef1234567890
 * ```
 */
export declare class NotificationConfiguration extends pulumi.CustomResource {
    /**
     * Get an existing NotificationConfiguration resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: NotificationConfigurationState, opts?: pulumi.CustomResourceOptions): NotificationConfiguration;
    /**
     * Returns true if the given object is an instance of NotificationConfiguration.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is NotificationConfiguration;
    /**
     * Aggregation preference of the NotificationConfiguration. Valid values: `LONG` (
     * aggregate notifications for 12 hours), `SHORT` (aggregate notifications for 5 minutes), `NONE` (don't aggregate
     * notifications). Default: `NONE`.
     */
    readonly aggregationDuration: pulumi.Output<string>;
    /**
     * Amazon Resource Name (ARN) of the NotificationConfiguration.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Description of the NotificationConfiguration. Length constraints: Minimum length of 0,
     * maximum length of 256.
     */
    readonly description: pulumi.Output<string>;
    /**
     * Name of the NotificationConfiguration. Supports RFC 3986's unreserved characters. Length
     * constraints: Minimum length of 1, maximum length of 64. Pattern: `[A-Za-z0-9_\-]+`.
     *
     * The following arguments are optional:
     */
    readonly name: pulumi.Output<string>;
    /**
     * Map of tags to assign to the resource. A tag is a string-to-string map of key-value pairs. If
     * configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those
     * defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * Map of tags assigned to the resource, including those inherited from the provider `defaultTags`
     * configuration block.
     */
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * Create a NotificationConfiguration resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: NotificationConfigurationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering NotificationConfiguration resources.
 */
export interface NotificationConfigurationState {
    /**
     * Aggregation preference of the NotificationConfiguration. Valid values: `LONG` (
     * aggregate notifications for 12 hours), `SHORT` (aggregate notifications for 5 minutes), `NONE` (don't aggregate
     * notifications). Default: `NONE`.
     */
    aggregationDuration?: pulumi.Input<string>;
    /**
     * Amazon Resource Name (ARN) of the NotificationConfiguration.
     */
    arn?: pulumi.Input<string>;
    /**
     * Description of the NotificationConfiguration. Length constraints: Minimum length of 0,
     * maximum length of 256.
     */
    description?: pulumi.Input<string>;
    /**
     * Name of the NotificationConfiguration. Supports RFC 3986's unreserved characters. Length
     * constraints: Minimum length of 1, maximum length of 64. Pattern: `[A-Za-z0-9_\-]+`.
     *
     * The following arguments are optional:
     */
    name?: pulumi.Input<string>;
    /**
     * Map of tags to assign to the resource. A tag is a string-to-string map of key-value pairs. If
     * configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those
     * defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Map of tags assigned to the resource, including those inherited from the provider `defaultTags`
     * configuration block.
     */
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
/**
 * The set of arguments for constructing a NotificationConfiguration resource.
 */
export interface NotificationConfigurationArgs {
    /**
     * Aggregation preference of the NotificationConfiguration. Valid values: `LONG` (
     * aggregate notifications for 12 hours), `SHORT` (aggregate notifications for 5 minutes), `NONE` (don't aggregate
     * notifications). Default: `NONE`.
     */
    aggregationDuration?: pulumi.Input<string>;
    /**
     * Description of the NotificationConfiguration. Length constraints: Minimum length of 0,
     * maximum length of 256.
     */
    description: pulumi.Input<string>;
    /**
     * Name of the NotificationConfiguration. Supports RFC 3986's unreserved characters. Length
     * constraints: Minimum length of 1, maximum length of 64. Pattern: `[A-Za-z0-9_\-]+`.
     *
     * The following arguments are optional:
     */
    name?: pulumi.Input<string>;
    /**
     * Map of tags to assign to the resource. A tag is a string-to-string map of key-value pairs. If
     * configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those
     * defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
