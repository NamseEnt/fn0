import * as pulumi from "@pulumi/pulumi";
/**
 * Manages an Amazon Managed Service for Prometheus (AMP) Alert Manager Definition
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const demo = new aws.amp.Workspace("demo", {});
 * const demoAlertManagerDefinition = new aws.amp.AlertManagerDefinition("demo", {
 *     workspaceId: demo.id,
 *     definition: `alertmanager_config: |
 *   route:
 *     receiver: 'default'
 *   receivers:
 *     - name: 'default'
 * `,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import the prometheus alert manager definition using the workspace identifier. For example:
 *
 * ```sh
 * $ pulumi import aws:amp/alertManagerDefinition:AlertManagerDefinition demo ws-C6DCB907-F2D7-4D96-957B-66691F865D8B
 * ```
 */
export declare class AlertManagerDefinition extends pulumi.CustomResource {
    /**
     * Get an existing AlertManagerDefinition resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: AlertManagerDefinitionState, opts?: pulumi.CustomResourceOptions): AlertManagerDefinition;
    /**
     * Returns true if the given object is an instance of AlertManagerDefinition.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is AlertManagerDefinition;
    /**
     * the alert manager definition that you want to be applied. See more [in AWS Docs](https://docs.aws.amazon.com/prometheus/latest/userguide/AMP-alert-manager.html).
     */
    readonly definition: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * ID of the prometheus workspace the alert manager definition should be linked to
     */
    readonly workspaceId: pulumi.Output<string>;
    /**
     * Create a AlertManagerDefinition resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: AlertManagerDefinitionArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering AlertManagerDefinition resources.
 */
export interface AlertManagerDefinitionState {
    /**
     * the alert manager definition that you want to be applied. See more [in AWS Docs](https://docs.aws.amazon.com/prometheus/latest/userguide/AMP-alert-manager.html).
     */
    definition?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ID of the prometheus workspace the alert manager definition should be linked to
     */
    workspaceId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a AlertManagerDefinition resource.
 */
export interface AlertManagerDefinitionArgs {
    /**
     * the alert manager definition that you want to be applied. See more [in AWS Docs](https://docs.aws.amazon.com/prometheus/latest/userguide/AMP-alert-manager.html).
     */
    definition: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ID of the prometheus workspace the alert manager definition should be linked to
     */
    workspaceId: pulumi.Input<string>;
}
