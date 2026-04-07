import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages a Service Catalog self-service action.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.servicecatalog.ServiceAction("example", {
 *     description: "Motor generator unit",
 *     name: "MGU",
 *     definition: {
 *         name: "AWS-RestartEC2Instance",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_servicecatalog_service_action` using the service action ID. For example:
 *
 * ```sh
 * $ pulumi import aws:servicecatalog/serviceAction:ServiceAction example act-f1w12eperfslh
 * ```
 */
export declare class ServiceAction extends pulumi.CustomResource {
    /**
     * Get an existing ServiceAction resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ServiceActionState, opts?: pulumi.CustomResourceOptions): ServiceAction;
    /**
     * Returns true if the given object is an instance of ServiceAction.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ServiceAction;
    /**
     * Language code. Valid values are `en` (English), `jp` (Japanese), and `zh` (Chinese). Default is `en`.
     */
    readonly acceptLanguage: pulumi.Output<string | undefined>;
    /**
     * Self-service action definition configuration block. Detailed below.
     */
    readonly definition: pulumi.Output<outputs.servicecatalog.ServiceActionDefinition>;
    /**
     * Self-service action description.
     */
    readonly description: pulumi.Output<string>;
    /**
     * Self-service action name.
     *
     * The following arguments are optional:
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Create a ServiceAction resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ServiceActionArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ServiceAction resources.
 */
export interface ServiceActionState {
    /**
     * Language code. Valid values are `en` (English), `jp` (Japanese), and `zh` (Chinese). Default is `en`.
     */
    acceptLanguage?: pulumi.Input<string>;
    /**
     * Self-service action definition configuration block. Detailed below.
     */
    definition?: pulumi.Input<inputs.servicecatalog.ServiceActionDefinition>;
    /**
     * Self-service action description.
     */
    description?: pulumi.Input<string>;
    /**
     * Self-service action name.
     *
     * The following arguments are optional:
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a ServiceAction resource.
 */
export interface ServiceActionArgs {
    /**
     * Language code. Valid values are `en` (English), `jp` (Japanese), and `zh` (Chinese). Default is `en`.
     */
    acceptLanguage?: pulumi.Input<string>;
    /**
     * Self-service action definition configuration block. Detailed below.
     */
    definition: pulumi.Input<inputs.servicecatalog.ServiceActionDefinition>;
    /**
     * Self-service action description.
     */
    description?: pulumi.Input<string>;
    /**
     * Self-service action name.
     *
     * The following arguments are optional:
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
