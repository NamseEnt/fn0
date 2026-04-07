import * as pulumi from "@pulumi/pulumi";
/**
 * Resource for managing an AWS Shield Subscription.
 *
 * > This resource creates a subscription to AWS Shield Advanced, which requires a 1 year subscription commitment with a monthly fee. Refer to the [AWS Shield Pricing](https://aws.amazon.com/shield/pricing/) page for more details.
 *
 * > Destruction of this resource will set `autoRenew` to `DISABLED`. Automatic renewal can only be disabled during the last 30 days of a subscription. To unsubscribe outside of this window, you must contact AWS Support. Set `skipDestroy` to `true` to skip modifying the `autoRenew` argument during destruction.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.shield.Subscription("example", {autoRenew: "ENABLED"});
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Shield Subscription using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:shield/subscription:Subscription example 123456789012
 * ```
 */
export declare class Subscription extends pulumi.CustomResource {
    /**
     * Get an existing Subscription resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: SubscriptionState, opts?: pulumi.CustomResourceOptions): Subscription;
    /**
     * Returns true if the given object is an instance of Subscription.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Subscription;
    /**
     * Toggle for automated renewal of the subscription. Valid values are `ENABLED` or `DISABLED`. Default is `ENABLED`.
     */
    readonly autoRenew: pulumi.Output<string>;
    /**
     * Skip attempting to disable automated renewal upon destruction. If set to `true`, the `autoRenew` value will be left as-is and the resource will simply be removed from state.
     */
    readonly skipDestroy: pulumi.Output<boolean | undefined>;
    /**
     * Create a Subscription resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: SubscriptionArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Subscription resources.
 */
export interface SubscriptionState {
    /**
     * Toggle for automated renewal of the subscription. Valid values are `ENABLED` or `DISABLED`. Default is `ENABLED`.
     */
    autoRenew?: pulumi.Input<string>;
    /**
     * Skip attempting to disable automated renewal upon destruction. If set to `true`, the `autoRenew` value will be left as-is and the resource will simply be removed from state.
     */
    skipDestroy?: pulumi.Input<boolean>;
}
/**
 * The set of arguments for constructing a Subscription resource.
 */
export interface SubscriptionArgs {
    /**
     * Toggle for automated renewal of the subscription. Valid values are `ENABLED` or `DISABLED`. Default is `ENABLED`.
     */
    autoRenew?: pulumi.Input<string>;
    /**
     * Skip attempting to disable automated renewal upon destruction. If set to `true`, the `autoRenew` value will be left as-is and the resource will simply be removed from state.
     */
    skipDestroy?: pulumi.Input<boolean>;
}
