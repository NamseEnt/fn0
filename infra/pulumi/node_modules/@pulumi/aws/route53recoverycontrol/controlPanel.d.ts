import * as pulumi from "@pulumi/pulumi";
/**
 * Provides an AWS Route 53 Recovery Control Config Control Panel.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.route53recoverycontrol.ControlPanel("example", {
 *     name: "balmorhea",
 *     clusterArn: "arn:aws:route53-recovery-control::123456789012:cluster/8d47920e-d789-437d-803a-2dcc4b204393",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Route53 Recovery Control Config Control Panel using the control panel arn. For example:
 *
 * ```sh
 * $ pulumi import aws:route53recoverycontrol/controlPanel:ControlPanel mypanel arn:aws:route53-recovery-control::313517334327:controlpanel/1bfba17df8684f5dab0467b71424f7e8
 * ```
 */
export declare class ControlPanel extends pulumi.CustomResource {
    /**
     * Get an existing ControlPanel resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ControlPanelState, opts?: pulumi.CustomResourceOptions): ControlPanel;
    /**
     * Returns true if the given object is an instance of ControlPanel.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ControlPanel;
    /**
     * ARN of the control panel.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * ARN of the cluster in which this control panel will reside.
     */
    readonly clusterArn: pulumi.Output<string>;
    /**
     * Whether a control panel is default.
     */
    readonly defaultControlPanel: pulumi.Output<boolean>;
    /**
     * Name describing the control panel.
     *
     * The following arguments are optional:
     */
    readonly name: pulumi.Output<string>;
    /**
     * Number routing controls in a control panel.
     */
    readonly routingControlCount: pulumi.Output<number>;
    /**
     * Status of control panel: `PENDING` when it is being created/updated, `PENDING_DELETION` when it is being deleted, and `DEPLOYED` otherwise.
     */
    readonly status: pulumi.Output<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * A map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * Create a ControlPanel resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ControlPanelArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ControlPanel resources.
 */
export interface ControlPanelState {
    /**
     * ARN of the control panel.
     */
    arn?: pulumi.Input<string>;
    /**
     * ARN of the cluster in which this control panel will reside.
     */
    clusterArn?: pulumi.Input<string>;
    /**
     * Whether a control panel is default.
     */
    defaultControlPanel?: pulumi.Input<boolean>;
    /**
     * Name describing the control panel.
     *
     * The following arguments are optional:
     */
    name?: pulumi.Input<string>;
    /**
     * Number routing controls in a control panel.
     */
    routingControlCount?: pulumi.Input<number>;
    /**
     * Status of control panel: `PENDING` when it is being created/updated, `PENDING_DELETION` when it is being deleted, and `DEPLOYED` otherwise.
     */
    status?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * A map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
/**
 * The set of arguments for constructing a ControlPanel resource.
 */
export interface ControlPanelArgs {
    /**
     * ARN of the cluster in which this control panel will reside.
     */
    clusterArn: pulumi.Input<string>;
    /**
     * Name describing the control panel.
     *
     * The following arguments are optional:
     */
    name?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
