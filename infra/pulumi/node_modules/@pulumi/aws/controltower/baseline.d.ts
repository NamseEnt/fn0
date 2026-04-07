import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS Control Tower Baseline.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.controltower.Baseline("example", {
 *     baselineIdentifier: "arn:aws:controltower:us-east-1::baseline/17BSJV3IGJ2QSGA2",
 *     baselineVersion: "4.0",
 *     targetIdentifier: test.arn,
 *     parameters: {
 *         key: "IdentityCenterEnabledBaselineArn",
 *         value: "arn:aws:controltower:us-east-1:664418989480:enabledbaseline/XALULM96QHI525UOC",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Control Tower Baseline using the `arn`. For example:
 *
 * ```sh
 * $ pulumi import aws:controltower/baseline:Baseline example arn:aws:controltower:us-east-1:012345678912:enabledbaseline/XALULM96QHI525UOC
 * ```
 */
export declare class Baseline extends pulumi.CustomResource {
    /**
     * Get an existing Baseline resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: BaselineState, opts?: pulumi.CustomResourceOptions): Baseline;
    /**
     * Returns true if the given object is an instance of Baseline.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Baseline;
    /**
     * ARN of the Baseline.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The ARN of the baseline to be enabled.
     */
    readonly baselineIdentifier: pulumi.Output<string>;
    /**
     * The version of the baseline to be enabled.
     */
    readonly baselineVersion: pulumi.Output<string>;
    readonly operationIdentifier: pulumi.Output<string>;
    /**
     * A list of key-value objects that specify enablement parameters, where key is a string and value is a document of any type. See Parameter below for details.
     */
    readonly parameters: pulumi.Output<outputs.controltower.BaselineParameters | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Tags to apply to the landing zone. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * A map of tags assigned to the landing zone, including those inherited from the provider `defaultTags` configuration block.
     */
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * The ARN of the target on which the baseline will be enabled. Only OUs are supported as targets.
     *
     * The following arguments are optional:
     */
    readonly targetIdentifier: pulumi.Output<string>;
    readonly timeouts: pulumi.Output<outputs.controltower.BaselineTimeouts | undefined>;
    /**
     * Create a Baseline resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: BaselineArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Baseline resources.
 */
export interface BaselineState {
    /**
     * ARN of the Baseline.
     */
    arn?: pulumi.Input<string>;
    /**
     * The ARN of the baseline to be enabled.
     */
    baselineIdentifier?: pulumi.Input<string>;
    /**
     * The version of the baseline to be enabled.
     */
    baselineVersion?: pulumi.Input<string>;
    operationIdentifier?: pulumi.Input<string>;
    /**
     * A list of key-value objects that specify enablement parameters, where key is a string and value is a document of any type. See Parameter below for details.
     */
    parameters?: pulumi.Input<inputs.controltower.BaselineParameters>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Tags to apply to the landing zone. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * A map of tags assigned to the landing zone, including those inherited from the provider `defaultTags` configuration block.
     */
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The ARN of the target on which the baseline will be enabled. Only OUs are supported as targets.
     *
     * The following arguments are optional:
     */
    targetIdentifier?: pulumi.Input<string>;
    timeouts?: pulumi.Input<inputs.controltower.BaselineTimeouts>;
}
/**
 * The set of arguments for constructing a Baseline resource.
 */
export interface BaselineArgs {
    /**
     * The ARN of the baseline to be enabled.
     */
    baselineIdentifier: pulumi.Input<string>;
    /**
     * The version of the baseline to be enabled.
     */
    baselineVersion: pulumi.Input<string>;
    /**
     * A list of key-value objects that specify enablement parameters, where key is a string and value is a document of any type. See Parameter below for details.
     */
    parameters?: pulumi.Input<inputs.controltower.BaselineParameters>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Tags to apply to the landing zone. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The ARN of the target on which the baseline will be enabled. Only OUs are supported as targets.
     *
     * The following arguments are optional:
     */
    targetIdentifier: pulumi.Input<string>;
    timeouts?: pulumi.Input<inputs.controltower.BaselineTimeouts>;
}
