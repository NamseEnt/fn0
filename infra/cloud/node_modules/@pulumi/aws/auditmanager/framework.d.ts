import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS Audit Manager Framework.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = new aws.auditmanager.Framework("test", {
 *     name: "example",
 *     controlSets: [{
 *         name: "example",
 *         controls: [
 *             {
 *                 id: test1.id,
 *             },
 *             {
 *                 id: test2.id,
 *             },
 *         ],
 *     }],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Audit Manager Framework using the framework `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:auditmanager/framework:Framework example abc123-de45
 * ```
 */
export declare class Framework extends pulumi.CustomResource {
    /**
     * Get an existing Framework resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: FrameworkState, opts?: pulumi.CustomResourceOptions): Framework;
    /**
     * Returns true if the given object is an instance of Framework.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Framework;
    /**
     * Amazon Resource Name (ARN) of the framework.
     * * `control_sets[*].id` - Unique identifier for the framework control set.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Compliance type that the new custom framework supports, such as `CIS` or `HIPAA`.
     */
    readonly complianceType: pulumi.Output<string | undefined>;
    /**
     * Configuration block(s) for the control sets that are associated with the framework. See `controlSets` Block below for details.
     *
     * The following arguments are optional:
     */
    readonly controlSets: pulumi.Output<outputs.auditmanager.FrameworkControlSet[] | undefined>;
    /**
     * Description of the framework.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * Framework type, such as a custom framework or a standard framework.
     */
    readonly frameworkType: pulumi.Output<string>;
    /**
     * Name of the framework.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * A map of tags to assign to the framework. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * Create a Framework resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: FrameworkArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Framework resources.
 */
export interface FrameworkState {
    /**
     * Amazon Resource Name (ARN) of the framework.
     * * `control_sets[*].id` - Unique identifier for the framework control set.
     */
    arn?: pulumi.Input<string>;
    /**
     * Compliance type that the new custom framework supports, such as `CIS` or `HIPAA`.
     */
    complianceType?: pulumi.Input<string>;
    /**
     * Configuration block(s) for the control sets that are associated with the framework. See `controlSets` Block below for details.
     *
     * The following arguments are optional:
     */
    controlSets?: pulumi.Input<pulumi.Input<inputs.auditmanager.FrameworkControlSet>[]>;
    /**
     * Description of the framework.
     */
    description?: pulumi.Input<string>;
    /**
     * Framework type, such as a custom framework or a standard framework.
     */
    frameworkType?: pulumi.Input<string>;
    /**
     * Name of the framework.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the framework. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
/**
 * The set of arguments for constructing a Framework resource.
 */
export interface FrameworkArgs {
    /**
     * Compliance type that the new custom framework supports, such as `CIS` or `HIPAA`.
     */
    complianceType?: pulumi.Input<string>;
    /**
     * Configuration block(s) for the control sets that are associated with the framework. See `controlSets` Block below for details.
     *
     * The following arguments are optional:
     */
    controlSets?: pulumi.Input<pulumi.Input<inputs.auditmanager.FrameworkControlSet>[]>;
    /**
     * Description of the framework.
     */
    description?: pulumi.Input<string>;
    /**
     * Name of the framework.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the framework. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
