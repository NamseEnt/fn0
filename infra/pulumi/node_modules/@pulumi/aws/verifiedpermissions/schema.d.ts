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
 * const example = new aws.verifiedpermissions.Schema("example", {
 *     policyStoreId: exampleAwsVerifiedpermissionsPolicyStore.policyStoreId,
 *     definition: {
 *         value: JSON.stringify({
 *             Namespace: {
 *                 entityTypes: {},
 *                 actions: {},
 *             },
 *         }),
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Verified Permissions Policy Store Schema using the `policy_store_id`. For example:
 *
 * console
 *
 *  % pulumi import aws_verifiedpermissions_schema.example DxQg2j8xvXJQ1tQCYNWj9T
 */
export declare class Schema extends pulumi.CustomResource {
    /**
     * Get an existing Schema resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: SchemaState, opts?: pulumi.CustomResourceOptions): Schema;
    /**
     * Returns true if the given object is an instance of Schema.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Schema;
    /**
     * The definition of the schema.
     */
    readonly definition: pulumi.Output<outputs.verifiedpermissions.SchemaDefinition | undefined>;
    /**
     * (Optional) Identifies the namespaces of the entities referenced by this schema.
     */
    readonly namespaces: pulumi.Output<string[]>;
    /**
     * The ID of the Policy Store.
     */
    readonly policyStoreId: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Create a Schema resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: SchemaArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Schema resources.
 */
export interface SchemaState {
    /**
     * The definition of the schema.
     */
    definition?: pulumi.Input<inputs.verifiedpermissions.SchemaDefinition>;
    /**
     * (Optional) Identifies the namespaces of the entities referenced by this schema.
     */
    namespaces?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The ID of the Policy Store.
     */
    policyStoreId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a Schema resource.
 */
export interface SchemaArgs {
    /**
     * The definition of the schema.
     */
    definition?: pulumi.Input<inputs.verifiedpermissions.SchemaDefinition>;
    /**
     * The ID of the Policy Store.
     */
    policyStoreId: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
