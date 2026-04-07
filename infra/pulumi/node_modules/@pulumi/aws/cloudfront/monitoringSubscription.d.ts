import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a CloudFront real-time log configuration resource.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.cloudfront.MonitoringSubscription("example", {
 *     distributionId: exampleAwsCloudfrontDistribution.id,
 *     monitoringSubscription: {
 *         realtimeMetricsSubscriptionConfig: {
 *             realtimeMetricsSubscriptionStatus: "Enabled",
 *         },
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import CloudFront monitoring subscription using the id. For example:
 *
 * ```sh
 * $ pulumi import aws:cloudfront/monitoringSubscription:MonitoringSubscription example E3QYSUHO4VYRGB
 * ```
 */
export declare class MonitoringSubscription extends pulumi.CustomResource {
    /**
     * Get an existing MonitoringSubscription resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: MonitoringSubscriptionState, opts?: pulumi.CustomResourceOptions): MonitoringSubscription;
    /**
     * Returns true if the given object is an instance of MonitoringSubscription.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is MonitoringSubscription;
    /**
     * The ID of the distribution that you are enabling metrics for.
     */
    readonly distributionId: pulumi.Output<string>;
    /**
     * A monitoring subscription. This structure contains information about whether additional CloudWatch metrics are enabled for a given CloudFront distribution.
     */
    readonly monitoringSubscription: pulumi.Output<outputs.cloudfront.MonitoringSubscriptionMonitoringSubscription>;
    /**
     * Create a MonitoringSubscription resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: MonitoringSubscriptionArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering MonitoringSubscription resources.
 */
export interface MonitoringSubscriptionState {
    /**
     * The ID of the distribution that you are enabling metrics for.
     */
    distributionId?: pulumi.Input<string>;
    /**
     * A monitoring subscription. This structure contains information about whether additional CloudWatch metrics are enabled for a given CloudFront distribution.
     */
    monitoringSubscription?: pulumi.Input<inputs.cloudfront.MonitoringSubscriptionMonitoringSubscription>;
}
/**
 * The set of arguments for constructing a MonitoringSubscription resource.
 */
export interface MonitoringSubscriptionArgs {
    /**
     * The ID of the distribution that you are enabling metrics for.
     */
    distributionId: pulumi.Input<string>;
    /**
     * A monitoring subscription. This structure contains information about whether additional CloudWatch metrics are enabled for a given CloudFront distribution.
     */
    monitoringSubscription: pulumi.Input<inputs.cloudfront.MonitoringSubscriptionMonitoringSubscription>;
}
