import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS MediaLive InputSecurityGroup.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.medialive.InputSecurityGroup("example", {
 *     whitelistRules: [{
 *         cidr: "10.0.0.8/32",
 *     }],
 *     tags: {
 *         ENVIRONMENT: "prod",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import MediaLive InputSecurityGroup using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:medialive/inputSecurityGroup:InputSecurityGroup example 123456
 * ```
 */
export declare class InputSecurityGroup extends pulumi.CustomResource {
    /**
     * Get an existing InputSecurityGroup resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: InputSecurityGroupState, opts?: pulumi.CustomResourceOptions): InputSecurityGroup;
    /**
     * Returns true if the given object is an instance of InputSecurityGroup.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is InputSecurityGroup;
    /**
     * ARN of the InputSecurityGroup.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The list of inputs currently using this InputSecurityGroup.
     */
    readonly inputs: pulumi.Output<string[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * A map of tags to assign to the InputSecurityGroup. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * Whitelist rules. See Whitelist Rules for more details.
     *
     * The following arguments are optional:
     */
    readonly whitelistRules: pulumi.Output<outputs.medialive.InputSecurityGroupWhitelistRule[]>;
    /**
     * Create a InputSecurityGroup resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: InputSecurityGroupArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering InputSecurityGroup resources.
 */
export interface InputSecurityGroupState {
    /**
     * ARN of the InputSecurityGroup.
     */
    arn?: pulumi.Input<string>;
    /**
     * The list of inputs currently using this InputSecurityGroup.
     */
    inputs?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the InputSecurityGroup. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Whitelist rules. See Whitelist Rules for more details.
     *
     * The following arguments are optional:
     */
    whitelistRules?: pulumi.Input<pulumi.Input<inputs.medialive.InputSecurityGroupWhitelistRule>[]>;
}
/**
 * The set of arguments for constructing a InputSecurityGroup resource.
 */
export interface InputSecurityGroupArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the InputSecurityGroup. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Whitelist rules. See Whitelist Rules for more details.
     *
     * The following arguments are optional:
     */
    whitelistRules: pulumi.Input<pulumi.Input<inputs.medialive.InputSecurityGroupWhitelistRule>[]>;
}
