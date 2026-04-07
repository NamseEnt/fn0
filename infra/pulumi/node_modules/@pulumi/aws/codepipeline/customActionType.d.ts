import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a CodeDeploy CustomActionType
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.codepipeline.CustomActionType("example", {
 *     category: "Build",
 *     inputArtifactDetails: {
 *         maximumCount: 1,
 *         minimumCount: 0,
 *     },
 *     outputArtifactDetails: {
 *         maximumCount: 1,
 *         minimumCount: 0,
 *     },
 *     providerName: "example",
 *     version: "1",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import CodeDeploy CustomActionType using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:codepipeline/customActionType:CustomActionType example Build:pulumi:1
 * ```
 */
export declare class CustomActionType extends pulumi.CustomResource {
    /**
     * Get an existing CustomActionType resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: CustomActionTypeState, opts?: pulumi.CustomResourceOptions): CustomActionType;
    /**
     * Returns true if the given object is an instance of CustomActionType.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is CustomActionType;
    /**
     * The action ARN.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The category of the custom action. Valid values: `Source`, `Build`, `Deploy`, `Test`, `Invoke`, `Approval`
     */
    readonly category: pulumi.Output<string>;
    /**
     * The configuration properties for the custom action. Max 10 items.
     */
    readonly configurationProperties: pulumi.Output<outputs.codepipeline.CustomActionTypeConfigurationProperty[] | undefined>;
    readonly inputArtifactDetails: pulumi.Output<outputs.codepipeline.CustomActionTypeInputArtifactDetails>;
    readonly outputArtifactDetails: pulumi.Output<outputs.codepipeline.CustomActionTypeOutputArtifactDetails>;
    /**
     * The creator of the action being called.
     */
    readonly owner: pulumi.Output<string>;
    readonly providerName: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    readonly settings: pulumi.Output<outputs.codepipeline.CustomActionTypeSettings | undefined>;
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * A map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    readonly version: pulumi.Output<string>;
    /**
     * Create a CustomActionType resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: CustomActionTypeArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering CustomActionType resources.
 */
export interface CustomActionTypeState {
    /**
     * The action ARN.
     */
    arn?: pulumi.Input<string>;
    /**
     * The category of the custom action. Valid values: `Source`, `Build`, `Deploy`, `Test`, `Invoke`, `Approval`
     */
    category?: pulumi.Input<string>;
    /**
     * The configuration properties for the custom action. Max 10 items.
     */
    configurationProperties?: pulumi.Input<pulumi.Input<inputs.codepipeline.CustomActionTypeConfigurationProperty>[]>;
    inputArtifactDetails?: pulumi.Input<inputs.codepipeline.CustomActionTypeInputArtifactDetails>;
    outputArtifactDetails?: pulumi.Input<inputs.codepipeline.CustomActionTypeOutputArtifactDetails>;
    /**
     * The creator of the action being called.
     */
    owner?: pulumi.Input<string>;
    providerName?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    settings?: pulumi.Input<inputs.codepipeline.CustomActionTypeSettings>;
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * A map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    version?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a CustomActionType resource.
 */
export interface CustomActionTypeArgs {
    /**
     * The category of the custom action. Valid values: `Source`, `Build`, `Deploy`, `Test`, `Invoke`, `Approval`
     */
    category: pulumi.Input<string>;
    /**
     * The configuration properties for the custom action. Max 10 items.
     */
    configurationProperties?: pulumi.Input<pulumi.Input<inputs.codepipeline.CustomActionTypeConfigurationProperty>[]>;
    inputArtifactDetails: pulumi.Input<inputs.codepipeline.CustomActionTypeInputArtifactDetails>;
    outputArtifactDetails: pulumi.Input<inputs.codepipeline.CustomActionTypeOutputArtifactDetails>;
    providerName: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    settings?: pulumi.Input<inputs.codepipeline.CustomActionTypeSettings>;
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    version: pulumi.Input<string>;
}
