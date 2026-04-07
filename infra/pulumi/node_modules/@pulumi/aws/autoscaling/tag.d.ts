import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages an individual Autoscaling Group (ASG) tag. This resource should only be used in cases where ASGs are created outside the provider (e.g., ASGs implicitly created by EKS Node Groups).
 *
 * > **NOTE:** This tagging resource should not be combined with the resource for managing the parent resource. For example, using `aws.autoscaling.Group` and `aws.autoscaling.Tag` to manage tags of the same ASG will cause a perpetual difference where the `aws.autoscaling.Group` resource will try to remove the tag being added by the `aws.autoscaling.Tag` resource.
 *
 * > **NOTE:** This tagging resource does not use the provider `ignoreTags` configuration.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 * import * as std from "@pulumi/std";
 *
 * const example = new aws.eks.NodeGroup("example", {
 *     clusterName: "example",
 *     nodeGroupName: "example",
 * });
 * const exampleTag: aws.autoscaling.Tag[] = [];
 * std.tosetOutput({
 *     input: std.flattenOutput({
 *         input: example.resources.apply(resources => resources.map(resources => (resources.autoscalingGroups))),
 *     }).apply(invoke => .map(asg => (asg.name))),
 * }).apply(invoke => {
 *     for (const range of invoke.result.map((v, k) => ({key: k, value: v}))) {
 *         exampleTag.push(new aws.autoscaling.Tag(`example-${range.key}`, {
 *             autoscalingGroupName: range.value,
 *             tag: {
 *                 key: "k8s.io/cluster-autoscaler/node-template/label/eks.amazonaws.com/capacityType",
 *                 value: "SPOT",
 *                 propagateAtLaunch: false,
 *             },
 *         }));
 *     }
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_autoscaling_group_tag` using the ASG name and key, separated by a comma (`,`). For example:
 *
 * ```sh
 * $ pulumi import aws:autoscaling/tag:Tag example asg-example,k8s.io/cluster-autoscaler/node-template/label/eks.amazonaws.com/capacityType
 * ```
 */
export declare class Tag extends pulumi.CustomResource {
    /**
     * Get an existing Tag resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: TagState, opts?: pulumi.CustomResourceOptions): Tag;
    /**
     * Returns true if the given object is an instance of Tag.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Tag;
    /**
     * Name of the Autoscaling Group to apply the tag to.
     */
    readonly autoscalingGroupName: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Tag to create. The `tag` block is documented below.
     */
    readonly tag: pulumi.Output<outputs.autoscaling.TagTag>;
    /**
     * Create a Tag resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: TagArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Tag resources.
 */
export interface TagState {
    /**
     * Name of the Autoscaling Group to apply the tag to.
     */
    autoscalingGroupName?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Tag to create. The `tag` block is documented below.
     */
    tag?: pulumi.Input<inputs.autoscaling.TagTag>;
}
/**
 * The set of arguments for constructing a Tag resource.
 */
export interface TagArgs {
    /**
     * Name of the Autoscaling Group to apply the tag to.
     */
    autoscalingGroupName: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Tag to create. The `tag` block is documented below.
     */
    tag: pulumi.Input<inputs.autoscaling.TagTag>;
}
