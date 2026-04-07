import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a AWS Transfer Workflow resource.
 *
 * ## Example Usage
 *
 * ### Basic single step example
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.transfer.Workflow("example", {steps: [{
 *     deleteStepDetails: {
 *         name: "example",
 *         sourceFileLocation: "${original.file}",
 *     },
 *     type: "DELETE",
 * }]});
 * ```
 *
 * ### Multistep example
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.transfer.Workflow("example", {steps: [
 *     {
 *         customStepDetails: {
 *             name: "example",
 *             sourceFileLocation: "${original.file}",
 *             target: exampleAwsLambdaFunction.arn,
 *             timeoutSeconds: 60,
 *         },
 *         type: "CUSTOM",
 *     },
 *     {
 *         tagStepDetails: {
 *             name: "example",
 *             sourceFileLocation: "${original.file}",
 *             tags: [{
 *                 key: "Name",
 *                 value: "Hello World",
 *             }],
 *         },
 *         type: "TAG",
 *     },
 * ]});
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Transfer Workflows using the `worflow_id`. For example:
 *
 * ```sh
 * $ pulumi import aws:transfer/workflow:Workflow example example
 * ```
 */
export declare class Workflow extends pulumi.CustomResource {
    /**
     * Get an existing Workflow resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: WorkflowState, opts?: pulumi.CustomResourceOptions): Workflow;
    /**
     * Returns true if the given object is an instance of Workflow.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Workflow;
    /**
     * The Workflow ARN.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * A textual description for the workflow.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * Specifies the steps (actions) to take if errors are encountered during execution of the workflow. See Workflow Steps below.
     */
    readonly onExceptionSteps: pulumi.Output<outputs.transfer.WorkflowOnExceptionStep[] | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Specifies the details for the steps that are in the specified workflow. See Workflow Steps below.
     */
    readonly steps: pulumi.Output<outputs.transfer.WorkflowStep[]>;
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
     * Create a Workflow resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: WorkflowArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Workflow resources.
 */
export interface WorkflowState {
    /**
     * The Workflow ARN.
     */
    arn?: pulumi.Input<string>;
    /**
     * A textual description for the workflow.
     */
    description?: pulumi.Input<string>;
    /**
     * Specifies the steps (actions) to take if errors are encountered during execution of the workflow. See Workflow Steps below.
     */
    onExceptionSteps?: pulumi.Input<pulumi.Input<inputs.transfer.WorkflowOnExceptionStep>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Specifies the details for the steps that are in the specified workflow. See Workflow Steps below.
     */
    steps?: pulumi.Input<pulumi.Input<inputs.transfer.WorkflowStep>[]>;
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
 * The set of arguments for constructing a Workflow resource.
 */
export interface WorkflowArgs {
    /**
     * A textual description for the workflow.
     */
    description?: pulumi.Input<string>;
    /**
     * Specifies the steps (actions) to take if errors are encountered during execution of the workflow. See Workflow Steps below.
     */
    onExceptionSteps?: pulumi.Input<pulumi.Input<inputs.transfer.WorkflowOnExceptionStep>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Specifies the details for the steps that are in the specified workflow. See Workflow Steps below.
     */
    steps: pulumi.Input<pulumi.Input<inputs.transfer.WorkflowStep>[]>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
