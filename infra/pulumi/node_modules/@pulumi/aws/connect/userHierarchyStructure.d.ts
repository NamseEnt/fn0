import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides an Amazon Connect User Hierarchy Structure resource. For more information see
 * [Amazon Connect: Getting Started](https://docs.aws.amazon.com/connect/latest/adminguide/amazon-connect-get-started.html)
 *
 * ## Example Usage
 *
 * ### Basic
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.connect.UserHierarchyStructure("example", {
 *     instanceId: "aaaaaaaa-bbbb-cccc-dddd-111111111111",
 *     hierarchyStructure: {
 *         levelOne: {
 *             name: "levelone",
 *         },
 *     },
 * });
 * ```
 *
 * ### With Five Levels
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.connect.UserHierarchyStructure("example", {
 *     instanceId: "aaaaaaaa-bbbb-cccc-dddd-111111111111",
 *     hierarchyStructure: {
 *         levelOne: {
 *             name: "levelone",
 *         },
 *         levelTwo: {
 *             name: "leveltwo",
 *         },
 *         levelThree: {
 *             name: "levelthree",
 *         },
 *         levelFour: {
 *             name: "levelfour",
 *         },
 *         levelFive: {
 *             name: "levelfive",
 *         },
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Amazon Connect User Hierarchy Structures using the `instance_id`. For example:
 *
 * ```sh
 * $ pulumi import aws:connect/userHierarchyStructure:UserHierarchyStructure example f1288a1f-6193-445a-b47e-af739b2
 * ```
 */
export declare class UserHierarchyStructure extends pulumi.CustomResource {
    /**
     * Get an existing UserHierarchyStructure resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: UserHierarchyStructureState, opts?: pulumi.CustomResourceOptions): UserHierarchyStructure;
    /**
     * Returns true if the given object is an instance of UserHierarchyStructure.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is UserHierarchyStructure;
    /**
     * A block that defines the hierarchy structure's levels. The `hierarchyStructure` block is documented below.
     */
    readonly hierarchyStructure: pulumi.Output<outputs.connect.UserHierarchyStructureHierarchyStructure>;
    /**
     * Specifies the identifier of the hosting Amazon Connect Instance.
     */
    readonly instanceId: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Create a UserHierarchyStructure resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: UserHierarchyStructureArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering UserHierarchyStructure resources.
 */
export interface UserHierarchyStructureState {
    /**
     * A block that defines the hierarchy structure's levels. The `hierarchyStructure` block is documented below.
     */
    hierarchyStructure?: pulumi.Input<inputs.connect.UserHierarchyStructureHierarchyStructure>;
    /**
     * Specifies the identifier of the hosting Amazon Connect Instance.
     */
    instanceId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a UserHierarchyStructure resource.
 */
export interface UserHierarchyStructureArgs {
    /**
     * A block that defines the hierarchy structure's levels. The `hierarchyStructure` block is documented below.
     */
    hierarchyStructure: pulumi.Input<inputs.connect.UserHierarchyStructureHierarchyStructure>;
    /**
     * Specifies the identifier of the hosting Amazon Connect Instance.
     */
    instanceId: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
