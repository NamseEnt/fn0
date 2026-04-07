import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a VPC Endpoint connection notification resource.
 * Connection notifications notify subscribers of VPC Endpoint events.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const topic = aws.iam.getPolicyDocument({
 *     statements: [{
 *         effect: "Allow",
 *         principals: [{
 *             type: "Service",
 *             identifiers: ["vpce.amazonaws.com"],
 *         }],
 *         actions: ["SNS:Publish"],
 *         resources: ["arn:aws:sns:*:*:vpce-notification-topic"],
 *     }],
 * });
 * const topicTopic = new aws.sns.Topic("topic", {
 *     name: "vpce-notification-topic",
 *     policy: topic.then(topic => topic.json),
 * });
 * const foo = new aws.ec2.VpcEndpointService("foo", {
 *     acceptanceRequired: false,
 *     networkLoadBalancerArns: [test.arn],
 * });
 * const fooVpcEndpointConnectionNotification = new aws.ec2.VpcEndpointConnectionNotification("foo", {
 *     vpcEndpointServiceId: foo.id,
 *     connectionNotificationArn: topicTopic.arn,
 *     connectionEvents: [
 *         "Accept",
 *         "Reject",
 *     ],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import VPC Endpoint connection notifications using the VPC endpoint connection notification `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:ec2/vpcEndpointConnectionNotification:VpcEndpointConnectionNotification foo vpce-nfn-09e6ed3b4efba2263
 * ```
 */
export declare class VpcEndpointConnectionNotification extends pulumi.CustomResource {
    /**
     * Get an existing VpcEndpointConnectionNotification resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: VpcEndpointConnectionNotificationState, opts?: pulumi.CustomResourceOptions): VpcEndpointConnectionNotification;
    /**
     * Returns true if the given object is an instance of VpcEndpointConnectionNotification.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is VpcEndpointConnectionNotification;
    /**
     * One or more endpoint [events](https://docs.aws.amazon.com/AWSEC2/latest/APIReference/API_CreateVpcEndpointConnectionNotification.html#API_CreateVpcEndpointConnectionNotification_RequestParameters) for which to receive notifications.
     *
     * > **NOTE:** One of `vpcEndpointServiceId` or `vpcEndpointId` must be specified.
     */
    readonly connectionEvents: pulumi.Output<string[]>;
    /**
     * The ARN of the SNS topic for the notifications.
     */
    readonly connectionNotificationArn: pulumi.Output<string>;
    /**
     * The type of notification.
     */
    readonly notificationType: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The state of the notification.
     */
    readonly state: pulumi.Output<string>;
    /**
     * The ID of the VPC Endpoint to receive notifications for.
     */
    readonly vpcEndpointId: pulumi.Output<string | undefined>;
    /**
     * The ID of the VPC Endpoint Service to receive notifications for.
     */
    readonly vpcEndpointServiceId: pulumi.Output<string | undefined>;
    /**
     * Create a VpcEndpointConnectionNotification resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: VpcEndpointConnectionNotificationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering VpcEndpointConnectionNotification resources.
 */
export interface VpcEndpointConnectionNotificationState {
    /**
     * One or more endpoint [events](https://docs.aws.amazon.com/AWSEC2/latest/APIReference/API_CreateVpcEndpointConnectionNotification.html#API_CreateVpcEndpointConnectionNotification_RequestParameters) for which to receive notifications.
     *
     * > **NOTE:** One of `vpcEndpointServiceId` or `vpcEndpointId` must be specified.
     */
    connectionEvents?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The ARN of the SNS topic for the notifications.
     */
    connectionNotificationArn?: pulumi.Input<string>;
    /**
     * The type of notification.
     */
    notificationType?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The state of the notification.
     */
    state?: pulumi.Input<string>;
    /**
     * The ID of the VPC Endpoint to receive notifications for.
     */
    vpcEndpointId?: pulumi.Input<string>;
    /**
     * The ID of the VPC Endpoint Service to receive notifications for.
     */
    vpcEndpointServiceId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a VpcEndpointConnectionNotification resource.
 */
export interface VpcEndpointConnectionNotificationArgs {
    /**
     * One or more endpoint [events](https://docs.aws.amazon.com/AWSEC2/latest/APIReference/API_CreateVpcEndpointConnectionNotification.html#API_CreateVpcEndpointConnectionNotification_RequestParameters) for which to receive notifications.
     *
     * > **NOTE:** One of `vpcEndpointServiceId` or `vpcEndpointId` must be specified.
     */
    connectionEvents: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The ARN of the SNS topic for the notifications.
     */
    connectionNotificationArn: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The ID of the VPC Endpoint to receive notifications for.
     */
    vpcEndpointId?: pulumi.Input<string>;
    /**
     * The ID of the VPC Endpoint Service to receive notifications for.
     */
    vpcEndpointServiceId?: pulumi.Input<string>;
}
