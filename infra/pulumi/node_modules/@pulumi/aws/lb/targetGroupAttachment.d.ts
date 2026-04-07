import * as pulumi from "@pulumi/pulumi";
/**
 * Provides the ability to register instances and containers with an Application Load Balancer (ALB) or Network Load Balancer (NLB) target group. For attaching resources with Elastic Load Balancer (ELB), see the `aws.elb.Attachment` resource.
 *
 * > **Note:** `aws.alb.TargetGroupAttachment` is known as `aws.lb.TargetGroupAttachment`. The functionality is identical.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const testTargetGroup = new aws.lb.TargetGroup("test", {});
 * const testInstance = new aws.ec2.Instance("test", {});
 * const test = new aws.lb.TargetGroupAttachment("test", {
 *     targetGroupArn: testTargetGroup.arn,
 *     targetId: testInstance.id,
 *     port: 80,
 * });
 * ```
 *
 * ### Lambda Target
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = new aws.lb.TargetGroup("test", {
 *     name: "test",
 *     targetType: "lambda",
 * });
 * const testFunction = new aws.lambda.Function("test", {});
 * const withLb = new aws.lambda.Permission("with_lb", {
 *     statementId: "AllowExecutionFromlb",
 *     action: "lambda:InvokeFunction",
 *     "function": testFunction.name,
 *     principal: "elasticloadbalancing.amazonaws.com",
 *     sourceArn: test.arn,
 * });
 * const testTargetGroupAttachment = new aws.lb.TargetGroupAttachment("test", {
 *     targetGroupArn: test.arn,
 *     targetId: testFunction.arn,
 * }, {
 *     dependsOn: [withLb],
 * });
 * ```
 *
 * ## Import
 *
 * You cannot import Target Group Attachments.
 */
export declare class TargetGroupAttachment extends pulumi.CustomResource {
    /**
     * Get an existing TargetGroupAttachment resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: TargetGroupAttachmentState, opts?: pulumi.CustomResourceOptions): TargetGroupAttachment;
    /**
     * Returns true if the given object is an instance of TargetGroupAttachment.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is TargetGroupAttachment;
    /**
     * The Availability Zone where the IP address of the target is to be registered. If the private IP address is outside of the VPC scope, this value must be set to `all`.
     */
    readonly availabilityZone: pulumi.Output<string | undefined>;
    /**
     * The port on which targets receive traffic.
     */
    readonly port: pulumi.Output<number | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The ARN of the target group with which to register targets.
     */
    readonly targetGroupArn: pulumi.Output<string>;
    /**
     * The ID of the target. This is the Instance ID for an instance, or the container ID for an ECS container. If the target type is `ip`, specify an IP address. If the target type is `lambda`, specify the Lambda function ARN. If the target type is `alb`, specify the ALB ARN.
     *
     * The following arguments are optional:
     */
    readonly targetId: pulumi.Output<string>;
    /**
     * Create a TargetGroupAttachment resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: TargetGroupAttachmentArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering TargetGroupAttachment resources.
 */
export interface TargetGroupAttachmentState {
    /**
     * The Availability Zone where the IP address of the target is to be registered. If the private IP address is outside of the VPC scope, this value must be set to `all`.
     */
    availabilityZone?: pulumi.Input<string>;
    /**
     * The port on which targets receive traffic.
     */
    port?: pulumi.Input<number>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The ARN of the target group with which to register targets.
     */
    targetGroupArn?: pulumi.Input<string>;
    /**
     * The ID of the target. This is the Instance ID for an instance, or the container ID for an ECS container. If the target type is `ip`, specify an IP address. If the target type is `lambda`, specify the Lambda function ARN. If the target type is `alb`, specify the ALB ARN.
     *
     * The following arguments are optional:
     */
    targetId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a TargetGroupAttachment resource.
 */
export interface TargetGroupAttachmentArgs {
    /**
     * The Availability Zone where the IP address of the target is to be registered. If the private IP address is outside of the VPC scope, this value must be set to `all`.
     */
    availabilityZone?: pulumi.Input<string>;
    /**
     * The port on which targets receive traffic.
     */
    port?: pulumi.Input<number>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The ARN of the target group with which to register targets.
     */
    targetGroupArn: pulumi.Input<string>;
    /**
     * The ID of the target. This is the Instance ID for an instance, or the container ID for an ECS container. If the target type is `ip`, specify an IP address. If the target type is `lambda`, specify the Lambda function ARN. If the target type is `alb`, specify the ALB ARN.
     *
     * The following arguments are optional:
     */
    targetId: pulumi.Input<string>;
}
