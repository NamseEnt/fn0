import * as pulumi from "@pulumi/pulumi";
/**
 * Provides an AWS Route 53 Recovery Control Config Routing Control.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.route53recoverycontrol.RoutingControl("example", {
 *     name: "tinlicker",
 *     clusterArn: "arn:aws:route53-recovery-control::881188118811:cluster/8d47920e-d789-437d-803a-2dcc4b204393",
 * });
 * ```
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.route53recoverycontrol.RoutingControl("example", {
 *     name: "thomasoliver",
 *     clusterArn: "arn:aws:route53-recovery-control::881188118811:cluster/8d47920e-d789-437d-803a-2dcc4b204393",
 *     controlPanelArn: "arn:aws:route53-recovery-control::428113431245:controlpanel/abd5fbfc052d4844a082dbf400f61da8",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Route53 Recovery Control Config Routing Control using the routing control arn. For example:
 *
 * ```sh
 * $ pulumi import aws:route53recoverycontrol/routingControl:RoutingControl mycontrol arn:aws:route53-recovery-control::313517334327:controlpanel/abd5fbfc052d4844a082dbf400f61da8/routingcontrol/d5d90e587870494b
 * ```
 */
export declare class RoutingControl extends pulumi.CustomResource {
    /**
     * Get an existing RoutingControl resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: RoutingControlState, opts?: pulumi.CustomResourceOptions): RoutingControl;
    /**
     * Returns true if the given object is an instance of RoutingControl.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is RoutingControl;
    /**
     * ARN of the routing control.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * ARN of the cluster in which this routing control will reside.
     */
    readonly clusterArn: pulumi.Output<string>;
    /**
     * ARN of the control panel in which this routing control will reside.
     */
    readonly controlPanelArn: pulumi.Output<string>;
    /**
     * The name describing the routing control.
     *
     * The following arguments are optional:
     */
    readonly name: pulumi.Output<string>;
    /**
     * Status of routing control. `PENDING` when it is being created/updated, `PENDING_DELETION` when it is being deleted, and `DEPLOYED` otherwise.
     */
    readonly status: pulumi.Output<string>;
    /**
     * Create a RoutingControl resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: RoutingControlArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering RoutingControl resources.
 */
export interface RoutingControlState {
    /**
     * ARN of the routing control.
     */
    arn?: pulumi.Input<string>;
    /**
     * ARN of the cluster in which this routing control will reside.
     */
    clusterArn?: pulumi.Input<string>;
    /**
     * ARN of the control panel in which this routing control will reside.
     */
    controlPanelArn?: pulumi.Input<string>;
    /**
     * The name describing the routing control.
     *
     * The following arguments are optional:
     */
    name?: pulumi.Input<string>;
    /**
     * Status of routing control. `PENDING` when it is being created/updated, `PENDING_DELETION` when it is being deleted, and `DEPLOYED` otherwise.
     */
    status?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a RoutingControl resource.
 */
export interface RoutingControlArgs {
    /**
     * ARN of the cluster in which this routing control will reside.
     */
    clusterArn: pulumi.Input<string>;
    /**
     * ARN of the control panel in which this routing control will reside.
     */
    controlPanelArn?: pulumi.Input<string>;
    /**
     * The name describing the routing control.
     *
     * The following arguments are optional:
     */
    name?: pulumi.Input<string>;
}
