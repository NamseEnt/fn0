import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Attaches a traffic source to an Auto Scaling group.
 *
 * > **NOTE on Auto Scaling Groups, Attachments and Traffic Source Attachments:** Pulumi provides standalone Attachment (for attaching Classic Load Balancers and Application Load Balancer, Gateway Load Balancer, or Network Load Balancer target groups) and Traffic Source Attachment (for attaching Load Balancers and VPC Lattice target groups) resources and an Auto Scaling Group resource with `loadBalancers`, `targetGroupArns` and `trafficSource` attributes. Do not use the same traffic source in more than one of these resources. Doing so will cause a conflict of attachments. A `lifecycle` configuration block can be used to suppress differences if necessary.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.autoscaling.TrafficSourceAttachment("example", {
 *     autoscalingGroupName: exampleAwsAutoscalingGroup.id,
 *     trafficSource: {
 *         identifier: exampleAwsLbTargetGroup.arn,
 *         type: "elbv2",
 *     },
 * });
 * ```
 */
export declare class TrafficSourceAttachment extends pulumi.CustomResource {
    /**
     * Get an existing TrafficSourceAttachment resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: TrafficSourceAttachmentState, opts?: pulumi.CustomResourceOptions): TrafficSourceAttachment;
    /**
     * Returns true if the given object is an instance of TrafficSourceAttachment.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is TrafficSourceAttachment;
    /**
     * The name of the Auto Scaling group.
     */
    readonly autoscalingGroupName: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The unique identifiers of a traffic sources.
     */
    readonly trafficSource: pulumi.Output<outputs.autoscaling.TrafficSourceAttachmentTrafficSource | undefined>;
    /**
     * Create a TrafficSourceAttachment resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: TrafficSourceAttachmentArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering TrafficSourceAttachment resources.
 */
export interface TrafficSourceAttachmentState {
    /**
     * The name of the Auto Scaling group.
     */
    autoscalingGroupName?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The unique identifiers of a traffic sources.
     */
    trafficSource?: pulumi.Input<inputs.autoscaling.TrafficSourceAttachmentTrafficSource>;
}
/**
 * The set of arguments for constructing a TrafficSourceAttachment resource.
 */
export interface TrafficSourceAttachmentArgs {
    /**
     * The name of the Auto Scaling group.
     */
    autoscalingGroupName: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The unique identifiers of a traffic sources.
     */
    trafficSource?: pulumi.Input<inputs.autoscaling.TrafficSourceAttachmentTrafficSource>;
}
