import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS Bedrock Guardrail Version.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.bedrock.GuardrailVersion("example", {
 *     description: "example",
 *     guardrailArn: test.guardrailArn,
 *     skipDestroy: true,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Amazon Bedrock Guardrail Version using using a comma-delimited string of `guardrail_arn` and `version`. For example:
 *
 * ```sh
 * $ pulumi import aws:bedrock/guardrailVersion:GuardrailVersion example arn:aws:bedrock:us-west-2:123456789012:guardrail-id-12345678,1
 * ```
 */
export declare class GuardrailVersion extends pulumi.CustomResource {
    /**
     * Get an existing GuardrailVersion resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: GuardrailVersionState, opts?: pulumi.CustomResourceOptions): GuardrailVersion;
    /**
     * Returns true if the given object is an instance of GuardrailVersion.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is GuardrailVersion;
    /**
     * Description of the Guardrail version.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * Guardrail ARN.
     *
     * The following arguments are optional:
     */
    readonly guardrailArn: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Whether to retain the old version of a previously deployed Guardrail. Default is `false`
     */
    readonly skipDestroy: pulumi.Output<boolean | undefined>;
    readonly timeouts: pulumi.Output<outputs.bedrock.GuardrailVersionTimeouts | undefined>;
    /**
     * Guardrail version.
     */
    readonly version: pulumi.Output<string>;
    /**
     * Create a GuardrailVersion resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: GuardrailVersionArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering GuardrailVersion resources.
 */
export interface GuardrailVersionState {
    /**
     * Description of the Guardrail version.
     */
    description?: pulumi.Input<string>;
    /**
     * Guardrail ARN.
     *
     * The following arguments are optional:
     */
    guardrailArn?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Whether to retain the old version of a previously deployed Guardrail. Default is `false`
     */
    skipDestroy?: pulumi.Input<boolean>;
    timeouts?: pulumi.Input<inputs.bedrock.GuardrailVersionTimeouts>;
    /**
     * Guardrail version.
     */
    version?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a GuardrailVersion resource.
 */
export interface GuardrailVersionArgs {
    /**
     * Description of the Guardrail version.
     */
    description?: pulumi.Input<string>;
    /**
     * Guardrail ARN.
     *
     * The following arguments are optional:
     */
    guardrailArn: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Whether to retain the old version of a previously deployed Guardrail. Default is `false`
     */
    skipDestroy?: pulumi.Input<boolean>;
    timeouts?: pulumi.Input<inputs.bedrock.GuardrailVersionTimeouts>;
}
