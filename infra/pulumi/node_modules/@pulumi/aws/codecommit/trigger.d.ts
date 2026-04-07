import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a CodeCommit Trigger Resource.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = new aws.codecommit.Repository("test", {repositoryName: "test"});
 * const testTrigger = new aws.codecommit.Trigger("test", {
 *     repositoryName: test.repositoryName,
 *     triggers: [{
 *         name: "all",
 *         events: ["all"],
 *         destinationArn: testAwsSnsTopic.arn,
 *     }],
 * });
 * ```
 */
export declare class Trigger extends pulumi.CustomResource {
    /**
     * Get an existing Trigger resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: TriggerState, opts?: pulumi.CustomResourceOptions): Trigger;
    /**
     * Returns true if the given object is an instance of Trigger.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Trigger;
    /**
     * System-generated unique identifier.
     */
    readonly configurationId: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The name for the repository. This needs to be less than 100 characters.
     */
    readonly repositoryName: pulumi.Output<string>;
    /**
     * The name of the trigger.
     */
    readonly triggers: pulumi.Output<outputs.codecommit.TriggerTrigger[]>;
    /**
     * Create a Trigger resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: TriggerArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Trigger resources.
 */
export interface TriggerState {
    /**
     * System-generated unique identifier.
     */
    configurationId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The name for the repository. This needs to be less than 100 characters.
     */
    repositoryName?: pulumi.Input<string>;
    /**
     * The name of the trigger.
     */
    triggers?: pulumi.Input<pulumi.Input<inputs.codecommit.TriggerTrigger>[]>;
}
/**
 * The set of arguments for constructing a Trigger resource.
 */
export interface TriggerArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The name for the repository. This needs to be less than 100 characters.
     */
    repositoryName: pulumi.Input<string>;
    /**
     * The name of the trigger.
     */
    triggers: pulumi.Input<pulumi.Input<inputs.codecommit.TriggerTrigger>[]>;
}
