import * as pulumi from "@pulumi/pulumi";
/**
 * Resource for managing an AWS Verified Permissions Policy Template.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.verifiedpermissions.PolicyTemplate("example", {
 *     policyStoreId: exampleAwsVerifiedpermissionsPolicyStore.id,
 *     statement: "permit (principal in ?principal, action in PhotoFlash::Action::\"FullPhotoAccess\", resource == ?resource) unless { resource.IsPrivate };",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Verified Permissions Policy Store using the `policy_store_id:policy_template_id`. For example:
 *
 * ```sh
 * $ pulumi import aws:verifiedpermissions/policyTemplate:PolicyTemplate example policyStoreId:policyTemplateId
 * ```
 */
export declare class PolicyTemplate extends pulumi.CustomResource {
    /**
     * Get an existing PolicyTemplate resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: PolicyTemplateState, opts?: pulumi.CustomResourceOptions): PolicyTemplate;
    /**
     * Returns true if the given object is an instance of PolicyTemplate.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is PolicyTemplate;
    /**
     * The date the Policy Store was created.
     */
    readonly createdDate: pulumi.Output<string>;
    /**
     * Provides a description for the policy template.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * The ID of the Policy Store.
     */
    readonly policyStoreId: pulumi.Output<string>;
    /**
     * The ID of the Policy Store.
     */
    readonly policyTemplateId: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Defines the content of the statement, written in Cedar policy language.
     *
     * The following arguments are optional:
     */
    readonly statement: pulumi.Output<string>;
    /**
     * Create a PolicyTemplate resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: PolicyTemplateArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering PolicyTemplate resources.
 */
export interface PolicyTemplateState {
    /**
     * The date the Policy Store was created.
     */
    createdDate?: pulumi.Input<string>;
    /**
     * Provides a description for the policy template.
     */
    description?: pulumi.Input<string>;
    /**
     * The ID of the Policy Store.
     */
    policyStoreId?: pulumi.Input<string>;
    /**
     * The ID of the Policy Store.
     */
    policyTemplateId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Defines the content of the statement, written in Cedar policy language.
     *
     * The following arguments are optional:
     */
    statement?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a PolicyTemplate resource.
 */
export interface PolicyTemplateArgs {
    /**
     * Provides a description for the policy template.
     */
    description?: pulumi.Input<string>;
    /**
     * The ID of the Policy Store.
     */
    policyStoreId: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Defines the content of the statement, written in Cedar policy language.
     *
     * The following arguments are optional:
     */
    statement: pulumi.Input<string>;
}
