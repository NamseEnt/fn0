import * as pulumi from "@pulumi/pulumi";
/**
 * Creates Security Hub custom action.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.securityhub.Account("example", {});
 * const exampleActionTarget = new aws.securityhub.ActionTarget("example", {
 *     name: "Send notification to chat",
 *     identifier: "SendToChat",
 *     description: "This is custom action sends selected findings to chat",
 * }, {
 *     dependsOn: [example],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Security Hub custom action using the action target ARN. For example:
 *
 * ```sh
 * $ pulumi import aws:securityhub/actionTarget:ActionTarget example arn:aws:securityhub:eu-west-1:312940875350:action/custom/a
 * ```
 */
export declare class ActionTarget extends pulumi.CustomResource {
    /**
     * Get an existing ActionTarget resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ActionTargetState, opts?: pulumi.CustomResourceOptions): ActionTarget;
    /**
     * Returns true if the given object is an instance of ActionTarget.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ActionTarget;
    /**
     * Amazon Resource Name (ARN) of the Security Hub custom action target.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The name of the custom action target.
     */
    readonly description: pulumi.Output<string>;
    /**
     * The ID for the custom action target.
     */
    readonly identifier: pulumi.Output<string>;
    /**
     * The description for the custom action target.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Create a ActionTarget resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ActionTargetArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ActionTarget resources.
 */
export interface ActionTargetState {
    /**
     * Amazon Resource Name (ARN) of the Security Hub custom action target.
     */
    arn?: pulumi.Input<string>;
    /**
     * The name of the custom action target.
     */
    description?: pulumi.Input<string>;
    /**
     * The ID for the custom action target.
     */
    identifier?: pulumi.Input<string>;
    /**
     * The description for the custom action target.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a ActionTarget resource.
 */
export interface ActionTargetArgs {
    /**
     * The name of the custom action target.
     */
    description: pulumi.Input<string>;
    /**
     * The ID for the custom action target.
     */
    identifier: pulumi.Input<string>;
    /**
     * The description for the custom action target.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
