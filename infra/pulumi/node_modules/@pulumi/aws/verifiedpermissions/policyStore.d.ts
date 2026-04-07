import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.verifiedpermissions.PolicyStore("example", {validationSettings: {
 *     mode: "STRICT",
 * }});
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Verified Permissions Policy Store using the `policy_store_id`. For example:
 *
 * console
 *
 *  % pulumi import aws_verifiedpermissions_policy_store.example DxQg2j8xvXJQ1tQCYNWj9T
 */
export declare class PolicyStore extends pulumi.CustomResource {
    /**
     * Get an existing PolicyStore resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: PolicyStoreState, opts?: pulumi.CustomResourceOptions): PolicyStore;
    /**
     * Returns true if the given object is an instance of PolicyStore.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is PolicyStore;
    /**
     * The ARN of the Policy Store.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Specifies whether the policy store can be deleted. If enabled, the policy store can't be deleted. Valid Values: `ENABLED`, `DISABLED`. Default value: `DISABLED`.
     */
    readonly deletionProtection: pulumi.Output<string>;
    /**
     * A description of the Policy Store.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * The ID of the Policy Store.
     */
    readonly policyStoreId: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
    /**
     * Validation settings for the policy store.
     */
    readonly validationSettings: pulumi.Output<outputs.verifiedpermissions.PolicyStoreValidationSettings | undefined>;
    /**
     * Create a PolicyStore resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: PolicyStoreArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering PolicyStore resources.
 */
export interface PolicyStoreState {
    /**
     * The ARN of the Policy Store.
     */
    arn?: pulumi.Input<string>;
    /**
     * Specifies whether the policy store can be deleted. If enabled, the policy store can't be deleted. Valid Values: `ENABLED`, `DISABLED`. Default value: `DISABLED`.
     */
    deletionProtection?: pulumi.Input<string>;
    /**
     * A description of the Policy Store.
     */
    description?: pulumi.Input<string>;
    /**
     * The ID of the Policy Store.
     */
    policyStoreId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
    /**
     * Validation settings for the policy store.
     */
    validationSettings?: pulumi.Input<inputs.verifiedpermissions.PolicyStoreValidationSettings>;
}
/**
 * The set of arguments for constructing a PolicyStore resource.
 */
export interface PolicyStoreArgs {
    /**
     * Specifies whether the policy store can be deleted. If enabled, the policy store can't be deleted. Valid Values: `ENABLED`, `DISABLED`. Default value: `DISABLED`.
     */
    deletionProtection?: pulumi.Input<string>;
    /**
     * A description of the Policy Store.
     */
    description?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Validation settings for the policy store.
     */
    validationSettings?: pulumi.Input<inputs.verifiedpermissions.PolicyStoreValidationSettings>;
}
