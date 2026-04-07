import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a Glue Workflow resource.
 * The workflow graph (DAG) can be build using the `aws.glue.Trigger` resource.
 * See the example below for creating a graph with four nodes (two triggers and two jobs).
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.glue.Workflow("example", {name: "example"});
 * const example_start = new aws.glue.Trigger("example-start", {
 *     name: "trigger-start",
 *     type: "ON_DEMAND",
 *     workflowName: example.name,
 *     actions: [{
 *         jobName: "example-job",
 *     }],
 * });
 * const example_inner = new aws.glue.Trigger("example-inner", {
 *     name: "trigger-inner",
 *     type: "CONDITIONAL",
 *     workflowName: example.name,
 *     predicate: {
 *         conditions: [{
 *             jobName: "example-job",
 *             state: "SUCCEEDED",
 *         }],
 *     },
 *     actions: [{
 *         jobName: "another-example-job",
 *     }],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Glue Workflows using `name`. For example:
 *
 * ```sh
 * $ pulumi import aws:glue/workflow:Workflow MyWorkflow MyWorkflow
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
     * Amazon Resource Name (ARN) of Glue Workflow
     */
    readonly arn: pulumi.Output<string>;
    /**
     * A map of default run properties for this workflow. These properties are passed to all jobs associated to the workflow.
     */
    readonly defaultRunProperties: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * Description of the workflow.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * Prevents exceeding the maximum number of concurrent runs of any of the component jobs. If you leave this parameter blank, there is no limit to the number of concurrent workflow runs.
     */
    readonly maxConcurrentRuns: pulumi.Output<number | undefined>;
    /**
     * The name you assign to this workflow.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Key-value map of resource tags. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
    constructor(name: string, args?: WorkflowArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Workflow resources.
 */
export interface WorkflowState {
    /**
     * Amazon Resource Name (ARN) of Glue Workflow
     */
    arn?: pulumi.Input<string>;
    /**
     * A map of default run properties for this workflow. These properties are passed to all jobs associated to the workflow.
     */
    defaultRunProperties?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Description of the workflow.
     */
    description?: pulumi.Input<string>;
    /**
     * Prevents exceeding the maximum number of concurrent runs of any of the component jobs. If you leave this parameter blank, there is no limit to the number of concurrent workflow runs.
     */
    maxConcurrentRuns?: pulumi.Input<number>;
    /**
     * The name you assign to this workflow.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Key-value map of resource tags. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * A map of default run properties for this workflow. These properties are passed to all jobs associated to the workflow.
     */
    defaultRunProperties?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Description of the workflow.
     */
    description?: pulumi.Input<string>;
    /**
     * Prevents exceeding the maximum number of concurrent runs of any of the component jobs. If you leave this parameter blank, there is no limit to the number of concurrent workflow runs.
     */
    maxConcurrentRuns?: pulumi.Input<number>;
    /**
     * The name you assign to this workflow.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Key-value map of resource tags. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
