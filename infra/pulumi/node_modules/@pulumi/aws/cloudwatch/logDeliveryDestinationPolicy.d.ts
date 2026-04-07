import * as pulumi from "@pulumi/pulumi";
/**
 * Resource for managing an AWS CloudWatch Logs Delivery Destination Policy.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.cloudwatch.LogDeliveryDestinationPolicy("example", {
 *     deliveryDestinationName: exampleAwsCloudwatchLogDeliveryDestination.name,
 *     deliveryDestinationPolicy: exampleAwsIamPolicyDocument.json,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import CloudWatch Logs Delivery Destination Policy using the `delivery_destination_name`. For example:
 *
 * ```sh
 * $ pulumi import aws:cloudwatch/logDeliveryDestinationPolicy:LogDeliveryDestinationPolicy example example
 * ```
 */
export declare class LogDeliveryDestinationPolicy extends pulumi.CustomResource {
    /**
     * Get an existing LogDeliveryDestinationPolicy resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: LogDeliveryDestinationPolicyState, opts?: pulumi.CustomResourceOptions): LogDeliveryDestinationPolicy;
    /**
     * Returns true if the given object is an instance of LogDeliveryDestinationPolicy.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is LogDeliveryDestinationPolicy;
    /**
     * The name of the delivery destination to assign this policy to.
     */
    readonly deliveryDestinationName: pulumi.Output<string>;
    /**
     * The contents of the policy.
     */
    readonly deliveryDestinationPolicy: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Create a LogDeliveryDestinationPolicy resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: LogDeliveryDestinationPolicyArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering LogDeliveryDestinationPolicy resources.
 */
export interface LogDeliveryDestinationPolicyState {
    /**
     * The name of the delivery destination to assign this policy to.
     */
    deliveryDestinationName?: pulumi.Input<string>;
    /**
     * The contents of the policy.
     */
    deliveryDestinationPolicy?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a LogDeliveryDestinationPolicy resource.
 */
export interface LogDeliveryDestinationPolicyArgs {
    /**
     * The name of the delivery destination to assign this policy to.
     */
    deliveryDestinationName: pulumi.Input<string>;
    /**
     * The contents of the policy.
     */
    deliveryDestinationPolicy: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
