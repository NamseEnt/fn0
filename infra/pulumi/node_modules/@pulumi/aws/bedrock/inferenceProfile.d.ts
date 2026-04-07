import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS Bedrock Inference Profile.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const current = aws.getCallerIdentity({});
 * const example = new aws.bedrock.InferenceProfile("example", {
 *     name: "Claude Sonnet for Project 123",
 *     description: "Profile with tag for cost allocation tracking",
 *     modelSource: {
 *         copyFrom: "arn:aws:bedrock:us-west-2::foundation-model/anthropic.claude-3-5-sonnet-20241022-v2:0",
 *     },
 *     tags: {
 *         ProjectID: "123",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Bedrock Inference Profile using the `name`. For example:
 *
 * ```sh
 * $ pulumi import aws:bedrock/inferenceProfile:InferenceProfile example inference_profile-id-12345678
 * ```
 */
export declare class InferenceProfile extends pulumi.CustomResource {
    /**
     * Get an existing InferenceProfile resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: InferenceProfileState, opts?: pulumi.CustomResourceOptions): InferenceProfile;
    /**
     * Returns true if the given object is an instance of InferenceProfile.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is InferenceProfile;
    /**
     * The Amazon Resource Name (ARN) of the inference profile.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The time at which the inference profile was created.
     */
    readonly createdAt: pulumi.Output<string>;
    /**
     * The description of the inference profile.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * The source of the model this inference profile will track metrics and cost for. See `modelSource`.
     *
     * The following arguments are optional:
     */
    readonly modelSource: pulumi.Output<outputs.bedrock.InferenceProfileModelSource | undefined>;
    /**
     * A list of information about each model in the inference profile. See `models`.
     */
    readonly models: pulumi.Output<outputs.bedrock.InferenceProfileModel[]>;
    /**
     * The name of the inference profile.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The status of the inference profile. `ACTIVE` means that the inference profile is available to use.
     */
    readonly status: pulumi.Output<string>;
    /**
     * Key-value mapping of resource tags for the inference profile.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    readonly timeouts: pulumi.Output<outputs.bedrock.InferenceProfileTimeouts | undefined>;
    /**
     * The type of the inference profile. `SYSTEM_DEFINED` means that the inference profile is defined by Amazon Bedrock. `APPLICATION` means that the inference profile is defined by the user.
     */
    readonly type: pulumi.Output<string>;
    /**
     * The time at which the inference profile was last updated.
     */
    readonly updatedAt: pulumi.Output<string>;
    /**
     * Create a InferenceProfile resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: InferenceProfileArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering InferenceProfile resources.
 */
export interface InferenceProfileState {
    /**
     * The Amazon Resource Name (ARN) of the inference profile.
     */
    arn?: pulumi.Input<string>;
    /**
     * The time at which the inference profile was created.
     */
    createdAt?: pulumi.Input<string>;
    /**
     * The description of the inference profile.
     */
    description?: pulumi.Input<string>;
    /**
     * The source of the model this inference profile will track metrics and cost for. See `modelSource`.
     *
     * The following arguments are optional:
     */
    modelSource?: pulumi.Input<inputs.bedrock.InferenceProfileModelSource>;
    /**
     * A list of information about each model in the inference profile. See `models`.
     */
    models?: pulumi.Input<pulumi.Input<inputs.bedrock.InferenceProfileModel>[]>;
    /**
     * The name of the inference profile.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The status of the inference profile. `ACTIVE` means that the inference profile is available to use.
     */
    status?: pulumi.Input<string>;
    /**
     * Key-value mapping of resource tags for the inference profile.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    timeouts?: pulumi.Input<inputs.bedrock.InferenceProfileTimeouts>;
    /**
     * The type of the inference profile. `SYSTEM_DEFINED` means that the inference profile is defined by Amazon Bedrock. `APPLICATION` means that the inference profile is defined by the user.
     */
    type?: pulumi.Input<string>;
    /**
     * The time at which the inference profile was last updated.
     */
    updatedAt?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a InferenceProfile resource.
 */
export interface InferenceProfileArgs {
    /**
     * The description of the inference profile.
     */
    description?: pulumi.Input<string>;
    /**
     * The source of the model this inference profile will track metrics and cost for. See `modelSource`.
     *
     * The following arguments are optional:
     */
    modelSource?: pulumi.Input<inputs.bedrock.InferenceProfileModelSource>;
    /**
     * The name of the inference profile.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Key-value mapping of resource tags for the inference profile.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    timeouts?: pulumi.Input<inputs.bedrock.InferenceProfileTimeouts>;
}
