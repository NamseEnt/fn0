import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.iot.TopicRuleDestination("example", {vpcConfiguration: {
 *     roleArn: exampleAwsIamRole.arn,
 *     securityGroups: [exampleAwsSecurityGroup.id],
 *     subnetIds: exampleAwsSubnet.map(__item => __item.id),
 *     vpcId: exampleAwsVpc.id,
 * }});
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import IoT topic rule destinations using the `arn`. For example:
 *
 * ```sh
 * $ pulumi import aws:iot/topicRuleDestination:TopicRuleDestination example arn:aws:iot:us-west-2:123456789012:ruledestination/vpc/2ce781c8-68a6-4c52-9c62-63fe489ecc60
 * ```
 */
export declare class TopicRuleDestination extends pulumi.CustomResource {
    /**
     * Get an existing TopicRuleDestination resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: TopicRuleDestinationState, opts?: pulumi.CustomResourceOptions): TopicRuleDestination;
    /**
     * Returns true if the given object is an instance of TopicRuleDestination.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is TopicRuleDestination;
    /**
     * The ARN of the topic rule destination
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Whether or not to enable the destination. Default: `true`.
     */
    readonly enabled: pulumi.Output<boolean | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Configuration of the virtual private cloud (VPC) connection. For more info, see the [AWS documentation](https://docs.aws.amazon.com/iot/latest/developerguide/vpc-rule-action.html).
     */
    readonly vpcConfiguration: pulumi.Output<outputs.iot.TopicRuleDestinationVpcConfiguration>;
    /**
     * Create a TopicRuleDestination resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: TopicRuleDestinationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering TopicRuleDestination resources.
 */
export interface TopicRuleDestinationState {
    /**
     * The ARN of the topic rule destination
     */
    arn?: pulumi.Input<string>;
    /**
     * Whether or not to enable the destination. Default: `true`.
     */
    enabled?: pulumi.Input<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Configuration of the virtual private cloud (VPC) connection. For more info, see the [AWS documentation](https://docs.aws.amazon.com/iot/latest/developerguide/vpc-rule-action.html).
     */
    vpcConfiguration?: pulumi.Input<inputs.iot.TopicRuleDestinationVpcConfiguration>;
}
/**
 * The set of arguments for constructing a TopicRuleDestination resource.
 */
export interface TopicRuleDestinationArgs {
    /**
     * Whether or not to enable the destination. Default: `true`.
     */
    enabled?: pulumi.Input<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Configuration of the virtual private cloud (VPC) connection. For more info, see the [AWS documentation](https://docs.aws.amazon.com/iot/latest/developerguide/vpc-rule-action.html).
     */
    vpcConfiguration: pulumi.Input<inputs.iot.TopicRuleDestinationVpcConfiguration>;
}
