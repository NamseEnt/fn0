import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS Verified Permissions Policy.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = new aws.verifiedpermissions.Policy("test", {
 *     policyStoreId: testAwsVerifiedpermissionsPolicyStore.id,
 *     definition: {
 *         static: {
 *             statement: "permit (principal, action == Action::\"view\", resource in Album:: \"test_album\");",
 *         },
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Verified Permissions Policy using the `policy_id,policy_store_id`. For example:
 *
 * ```sh
 * $ pulumi import aws:verifiedpermissions/policy:Policy example policy-id-12345678,policy-store-id-12345678
 * ```
 */
export declare class Policy extends pulumi.CustomResource {
    /**
     * Get an existing Policy resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: PolicyState, opts?: pulumi.CustomResourceOptions): Policy;
    /**
     * Returns true if the given object is an instance of Policy.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Policy;
    /**
     * The date the policy was created.
     */
    readonly createdDate: pulumi.Output<string>;
    /**
     * The definition of the policy. See Definition below.
     */
    readonly definition: pulumi.Output<outputs.verifiedpermissions.PolicyDefinition | undefined>;
    /**
     * The Policy ID of the policy.
     */
    readonly policyId: pulumi.Output<string>;
    /**
     * The Policy Store ID of the policy store.
     */
    readonly policyStoreId: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Create a Policy resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: PolicyArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Policy resources.
 */
export interface PolicyState {
    /**
     * The date the policy was created.
     */
    createdDate?: pulumi.Input<string>;
    /**
     * The definition of the policy. See Definition below.
     */
    definition?: pulumi.Input<inputs.verifiedpermissions.PolicyDefinition>;
    /**
     * The Policy ID of the policy.
     */
    policyId?: pulumi.Input<string>;
    /**
     * The Policy Store ID of the policy store.
     */
    policyStoreId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a Policy resource.
 */
export interface PolicyArgs {
    /**
     * The definition of the policy. See Definition below.
     */
    definition?: pulumi.Input<inputs.verifiedpermissions.PolicyDefinition>;
    /**
     * The Policy Store ID of the policy store.
     */
    policyStoreId: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
