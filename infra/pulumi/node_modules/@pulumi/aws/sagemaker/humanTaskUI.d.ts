import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a SageMaker AI Human Task UI resource.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 * import * as std from "@pulumi/std";
 *
 * const example = new aws.sagemaker.HumanTaskUI("example", {
 *     humanTaskUiName: "example",
 *     uiTemplate: {
 *         content: std.file({
 *             input: "sagemaker-human-task-ui-template.html",
 *         }).then(invoke => invoke.result),
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import SageMaker AI Human Task UIs using the `human_task_ui_name`. For example:
 *
 * ```sh
 * $ pulumi import aws:sagemaker/humanTaskUI:HumanTaskUI example example
 * ```
 */
export declare class HumanTaskUI extends pulumi.CustomResource {
    /**
     * Get an existing HumanTaskUI resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: HumanTaskUIState, opts?: pulumi.CustomResourceOptions): HumanTaskUI;
    /**
     * Returns true if the given object is an instance of HumanTaskUI.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is HumanTaskUI;
    /**
     * The Amazon Resource Name (ARN) assigned by AWS to this Human Task UI.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The name of the Human Task UI.
     */
    readonly humanTaskUiName: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
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
     * The Liquid template for the worker user interface. See UI Template below.
     */
    readonly uiTemplate: pulumi.Output<outputs.sagemaker.HumanTaskUIUiTemplate>;
    /**
     * Create a HumanTaskUI resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: HumanTaskUIArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering HumanTaskUI resources.
 */
export interface HumanTaskUIState {
    /**
     * The Amazon Resource Name (ARN) assigned by AWS to this Human Task UI.
     */
    arn?: pulumi.Input<string>;
    /**
     * The name of the Human Task UI.
     */
    humanTaskUiName?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
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
    /**
     * The Liquid template for the worker user interface. See UI Template below.
     */
    uiTemplate?: pulumi.Input<inputs.sagemaker.HumanTaskUIUiTemplate>;
}
/**
 * The set of arguments for constructing a HumanTaskUI resource.
 */
export interface HumanTaskUIArgs {
    /**
     * The name of the Human Task UI.
     */
    humanTaskUiName: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The Liquid template for the worker user interface. See UI Template below.
     */
    uiTemplate: pulumi.Input<inputs.sagemaker.HumanTaskUIUiTemplate>;
}
