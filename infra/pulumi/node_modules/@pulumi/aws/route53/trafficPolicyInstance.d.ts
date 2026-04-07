import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a Route53 traffic policy instance resource.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = new aws.route53.TrafficPolicyInstance("test", {
 *     name: "test.example.com",
 *     trafficPolicyId: "b3gb108f-ea6f-45a5-baab-9d112d8b4037",
 *     trafficPolicyVersion: 1,
 *     hostedZoneId: "Z033120931TAQO548OGJC",
 *     ttl: 360,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Route53 traffic policy instance using its id. For example:
 *
 * ```sh
 * $ pulumi import aws:route53/trafficPolicyInstance:TrafficPolicyInstance test df579d9a-6396-410e-ac22-e7ad60cf9e7e
 * ```
 */
export declare class TrafficPolicyInstance extends pulumi.CustomResource {
    /**
     * Get an existing TrafficPolicyInstance resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: TrafficPolicyInstanceState, opts?: pulumi.CustomResourceOptions): TrafficPolicyInstance;
    /**
     * Returns true if the given object is an instance of TrafficPolicyInstance.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is TrafficPolicyInstance;
    /**
     * Amazon Resource Name (ARN) of the traffic policy instance.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * ID of the hosted zone that you want Amazon Route 53 to create resource record sets in by using the configuration in a traffic policy.
     */
    readonly hostedZoneId: pulumi.Output<string>;
    /**
     * Domain name for which Amazon Route 53 responds to DNS queries by using the resource record sets that Route 53 creates for this traffic policy instance.
     */
    readonly name: pulumi.Output<string>;
    /**
     * ID of the traffic policy that you want to use to create resource record sets in the specified hosted zone.
     */
    readonly trafficPolicyId: pulumi.Output<string>;
    /**
     * Version of the traffic policy
     */
    readonly trafficPolicyVersion: pulumi.Output<number>;
    /**
     * TTL that you want Amazon Route 53 to assign to all the resource record sets that it creates in the specified hosted zone.
     */
    readonly ttl: pulumi.Output<number>;
    /**
     * Create a TrafficPolicyInstance resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: TrafficPolicyInstanceArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering TrafficPolicyInstance resources.
 */
export interface TrafficPolicyInstanceState {
    /**
     * Amazon Resource Name (ARN) of the traffic policy instance.
     */
    arn?: pulumi.Input<string>;
    /**
     * ID of the hosted zone that you want Amazon Route 53 to create resource record sets in by using the configuration in a traffic policy.
     */
    hostedZoneId?: pulumi.Input<string>;
    /**
     * Domain name for which Amazon Route 53 responds to DNS queries by using the resource record sets that Route 53 creates for this traffic policy instance.
     */
    name?: pulumi.Input<string>;
    /**
     * ID of the traffic policy that you want to use to create resource record sets in the specified hosted zone.
     */
    trafficPolicyId?: pulumi.Input<string>;
    /**
     * Version of the traffic policy
     */
    trafficPolicyVersion?: pulumi.Input<number>;
    /**
     * TTL that you want Amazon Route 53 to assign to all the resource record sets that it creates in the specified hosted zone.
     */
    ttl?: pulumi.Input<number>;
}
/**
 * The set of arguments for constructing a TrafficPolicyInstance resource.
 */
export interface TrafficPolicyInstanceArgs {
    /**
     * ID of the hosted zone that you want Amazon Route 53 to create resource record sets in by using the configuration in a traffic policy.
     */
    hostedZoneId: pulumi.Input<string>;
    /**
     * Domain name for which Amazon Route 53 responds to DNS queries by using the resource record sets that Route 53 creates for this traffic policy instance.
     */
    name?: pulumi.Input<string>;
    /**
     * ID of the traffic policy that you want to use to create resource record sets in the specified hosted zone.
     */
    trafficPolicyId: pulumi.Input<string>;
    /**
     * Version of the traffic policy
     */
    trafficPolicyVersion: pulumi.Input<number>;
    /**
     * TTL that you want Amazon Route 53 to assign to all the resource record sets that it creates in the specified hosted zone.
     */
    ttl: pulumi.Input<number>;
}
