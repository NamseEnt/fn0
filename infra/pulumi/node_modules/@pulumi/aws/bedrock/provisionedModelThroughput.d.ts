import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages [Provisioned Throughput](https://docs.aws.amazon.com/bedrock/latest/userguide/prov-throughput.html) for an Amazon Bedrock model.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.bedrock.ProvisionedModelThroughput("example", {
 *     provisionedModelName: "example-model",
 *     modelArn: "arn:aws:bedrock:us-east-1::foundation-model/anthropic.claude-v2",
 *     commitmentDuration: "SixMonths",
 *     modelUnits: 1,
 * });
 * ```
 *
 * ## Import
 *
 * ### Identity Schema
 *
 * #### Required
 *
 * - `arn` (String) Amazon Resource Name (ARN) of the Bedrock provisioned model throughput.
 *
 * Using `pulumi import`, import Provisioned Throughput using the `provisioned_model_arn`. For example:
 *
 * console
 *
 * % pulumi import aws_bedrock_provisioned_model_throughput.example arn:aws:bedrock:us-west-2:123456789012:provisioned-model/1y5n57gh5y2e
 */
export declare class ProvisionedModelThroughput extends pulumi.CustomResource {
    /**
     * Get an existing ProvisionedModelThroughput resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ProvisionedModelThroughputState, opts?: pulumi.CustomResourceOptions): ProvisionedModelThroughput;
    /**
     * Returns true if the given object is an instance of ProvisionedModelThroughput.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ProvisionedModelThroughput;
    /**
     * Commitment duration requested for the Provisioned Throughput. For custom models, you can purchase on-demand Provisioned Throughput by omitting this argument. Valid values: `OneMonth`, `SixMonths`.
     */
    readonly commitmentDuration: pulumi.Output<string | undefined>;
    /**
     * ARN of the model to associate with this Provisioned Throughput.
     */
    readonly modelArn: pulumi.Output<string>;
    /**
     * Number of model units to allocate. A model unit delivers a specific throughput level for the specified model.
     */
    readonly modelUnits: pulumi.Output<number>;
    /**
     * The ARN of the Provisioned Throughput.
     */
    readonly provisionedModelArn: pulumi.Output<string>;
    /**
     * Unique name for this Provisioned Throughput.
     */
    readonly provisionedModelName: pulumi.Output<string>;
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
     * Map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    readonly timeouts: pulumi.Output<outputs.bedrock.ProvisionedModelThroughputTimeouts | undefined>;
    /**
     * Create a ProvisionedModelThroughput resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ProvisionedModelThroughputArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ProvisionedModelThroughput resources.
 */
export interface ProvisionedModelThroughputState {
    /**
     * Commitment duration requested for the Provisioned Throughput. For custom models, you can purchase on-demand Provisioned Throughput by omitting this argument. Valid values: `OneMonth`, `SixMonths`.
     */
    commitmentDuration?: pulumi.Input<string>;
    /**
     * ARN of the model to associate with this Provisioned Throughput.
     */
    modelArn?: pulumi.Input<string>;
    /**
     * Number of model units to allocate. A model unit delivers a specific throughput level for the specified model.
     */
    modelUnits?: pulumi.Input<number>;
    /**
     * The ARN of the Provisioned Throughput.
     */
    provisionedModelArn?: pulumi.Input<string>;
    /**
     * Unique name for this Provisioned Throughput.
     */
    provisionedModelName?: pulumi.Input<string>;
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
     * Map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    timeouts?: pulumi.Input<inputs.bedrock.ProvisionedModelThroughputTimeouts>;
}
/**
 * The set of arguments for constructing a ProvisionedModelThroughput resource.
 */
export interface ProvisionedModelThroughputArgs {
    /**
     * Commitment duration requested for the Provisioned Throughput. For custom models, you can purchase on-demand Provisioned Throughput by omitting this argument. Valid values: `OneMonth`, `SixMonths`.
     */
    commitmentDuration?: pulumi.Input<string>;
    /**
     * ARN of the model to associate with this Provisioned Throughput.
     */
    modelArn: pulumi.Input<string>;
    /**
     * Number of model units to allocate. A model unit delivers a specific throughput level for the specified model.
     */
    modelUnits: pulumi.Input<number>;
    /**
     * Unique name for this Provisioned Throughput.
     */
    provisionedModelName: pulumi.Input<string>;
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
    timeouts?: pulumi.Input<inputs.bedrock.ProvisionedModelThroughputTimeouts>;
}
