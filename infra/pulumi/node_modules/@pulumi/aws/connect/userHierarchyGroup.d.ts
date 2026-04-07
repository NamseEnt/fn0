import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides an Amazon Connect User Hierarchy Group resource. For more information see
 * [Amazon Connect: Getting Started](https://docs.aws.amazon.com/connect/latest/adminguide/amazon-connect-get-started.html)
 *
 * > **NOTE:** The User Hierarchy Structure must be created before creating a User Hierarchy Group.
 *
 * ## Example Usage
 *
 * ### Basic
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.connect.UserHierarchyGroup("example", {
 *     instanceId: "aaaaaaaa-bbbb-cccc-dddd-111111111111",
 *     name: "example",
 *     tags: {
 *         Name: "Example User Hierarchy Group",
 *     },
 * });
 * ```
 *
 * ### With a parent group
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const parent = new aws.connect.UserHierarchyGroup("parent", {
 *     instanceId: "aaaaaaaa-bbbb-cccc-dddd-111111111111",
 *     name: "parent",
 *     tags: {
 *         Name: "Example User Hierarchy Group Parent",
 *     },
 * });
 * const child = new aws.connect.UserHierarchyGroup("child", {
 *     instanceId: "aaaaaaaa-bbbb-cccc-dddd-111111111111",
 *     name: "child",
 *     parentGroupId: parent.hierarchyGroupId,
 *     tags: {
 *         Name: "Example User Hierarchy Group Child",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Amazon Connect User Hierarchy Groups using the `instance_id` and `hierarchy_group_id` separated by a colon (`:`). For example:
 *
 * ```sh
 * $ pulumi import aws:connect/userHierarchyGroup:UserHierarchyGroup example f1288a1f-6193-445a-b47e-af739b2:c1d4e5f6-1b3c-1b3c-1b3c-c1d4e5f6c1d4e5
 * ```
 */
export declare class UserHierarchyGroup extends pulumi.CustomResource {
    /**
     * Get an existing UserHierarchyGroup resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: UserHierarchyGroupState, opts?: pulumi.CustomResourceOptions): UserHierarchyGroup;
    /**
     * Returns true if the given object is an instance of UserHierarchyGroup.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is UserHierarchyGroup;
    /**
     * The Amazon Resource Name (ARN) of the hierarchy group.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The identifier for the hierarchy group.
     */
    readonly hierarchyGroupId: pulumi.Output<string>;
    /**
     * A block that contains information about the levels in the hierarchy group. The `hierarchyPath` block is documented below.
     */
    readonly hierarchyPaths: pulumi.Output<outputs.connect.UserHierarchyGroupHierarchyPath[]>;
    /**
     * Specifies the identifier of the hosting Amazon Connect Instance.
     */
    readonly instanceId: pulumi.Output<string>;
    /**
     * The identifier of the level in the hierarchy group.
     */
    readonly levelId: pulumi.Output<string>;
    /**
     * The name of the user hierarchy group. Must not be more than 100 characters.
     */
    readonly name: pulumi.Output<string>;
    /**
     * The identifier for the parent hierarchy group. The user hierarchy is created at level one if the parent group ID is null.
     */
    readonly parentGroupId: pulumi.Output<string | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Tags to apply to the hierarchy group. If configured with a provider
     * `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * A map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * Create a UserHierarchyGroup resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: UserHierarchyGroupArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering UserHierarchyGroup resources.
 */
export interface UserHierarchyGroupState {
    /**
     * The Amazon Resource Name (ARN) of the hierarchy group.
     */
    arn?: pulumi.Input<string>;
    /**
     * The identifier for the hierarchy group.
     */
    hierarchyGroupId?: pulumi.Input<string>;
    /**
     * A block that contains information about the levels in the hierarchy group. The `hierarchyPath` block is documented below.
     */
    hierarchyPaths?: pulumi.Input<pulumi.Input<inputs.connect.UserHierarchyGroupHierarchyPath>[]>;
    /**
     * Specifies the identifier of the hosting Amazon Connect Instance.
     */
    instanceId?: pulumi.Input<string>;
    /**
     * The identifier of the level in the hierarchy group.
     */
    levelId?: pulumi.Input<string>;
    /**
     * The name of the user hierarchy group. Must not be more than 100 characters.
     */
    name?: pulumi.Input<string>;
    /**
     * The identifier for the parent hierarchy group. The user hierarchy is created at level one if the parent group ID is null.
     */
    parentGroupId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Tags to apply to the hierarchy group. If configured with a provider
     * `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * A map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
/**
 * The set of arguments for constructing a UserHierarchyGroup resource.
 */
export interface UserHierarchyGroupArgs {
    /**
     * Specifies the identifier of the hosting Amazon Connect Instance.
     */
    instanceId: pulumi.Input<string>;
    /**
     * The name of the user hierarchy group. Must not be more than 100 characters.
     */
    name?: pulumi.Input<string>;
    /**
     * The identifier for the parent hierarchy group. The user hierarchy is created at level one if the parent group ID is null.
     */
    parentGroupId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Tags to apply to the hierarchy group. If configured with a provider
     * `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
