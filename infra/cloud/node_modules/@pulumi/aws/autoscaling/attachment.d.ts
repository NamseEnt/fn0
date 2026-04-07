import * as pulumi from "@pulumi/pulumi";
/**
 * Attaches a load balancer to an Auto Scaling group.
 *
 * > **NOTE on Auto Scaling Groups, Attachments and Traffic Source Attachments:** Pulumi provides standalone Attachment (for attaching Classic Load Balancers and Application Load Balancer, Gateway Load Balancer, or Network Load Balancer target groups) and Traffic Source Attachment (for attaching Load Balancers and VPC Lattice target groups) resources and an Auto Scaling Group resource with `loadBalancers`, `targetGroupArns` and `trafficSource` attributes. Do not use the same traffic source in more than one of these resources. Doing so will cause a conflict of attachments. A `lifecycle` configuration block can be used to suppress differences if necessary.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * // Create a new load balancer attachment
 * const example = new aws.autoscaling.Attachment("example", {
 *     autoscalingGroupName: exampleAwsAutoscalingGroup.id,
 *     elb: exampleAwsElb.id,
 * });
 * ```
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * // Create a new ALB Target Group attachment
 * const example = new aws.autoscaling.Attachment("example", {
 *     autoscalingGroupName: exampleAwsAutoscalingGroup.id,
 *     lbTargetGroupArn: exampleAwsLbTargetGroup.arn,
 * });
 * ```
 */
export declare class Attachment extends pulumi.CustomResource {
    /**
     * Get an existing Attachment resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: AttachmentState, opts?: pulumi.CustomResourceOptions): Attachment;
    /**
     * Returns true if the given object is an instance of Attachment.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Attachment;
    /**
     * Name of ASG to associate with the ELB.
     */
    readonly autoscalingGroupName: pulumi.Output<string>;
    /**
     * Name of the ELB.
     */
    readonly elb: pulumi.Output<string | undefined>;
    /**
     * ARN of a load balancer target group.
     */
    readonly lbTargetGroupArn: pulumi.Output<string | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Create a Attachment resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: AttachmentArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Attachment resources.
 */
export interface AttachmentState {
    /**
     * Name of ASG to associate with the ELB.
     */
    autoscalingGroupName?: pulumi.Input<string>;
    /**
     * Name of the ELB.
     */
    elb?: pulumi.Input<string>;
    /**
     * ARN of a load balancer target group.
     */
    lbTargetGroupArn?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a Attachment resource.
 */
export interface AttachmentArgs {
    /**
     * Name of ASG to associate with the ELB.
     */
    autoscalingGroupName: pulumi.Input<string>;
    /**
     * Name of the ELB.
     */
    elb?: pulumi.Input<string>;
    /**
     * ARN of a load balancer target group.
     */
    lbTargetGroupArn?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
