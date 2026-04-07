import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS IdentityStore Group.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const _this = new aws.identitystore.Group("this", {
 *     displayName: "Example group",
 *     description: "Example description",
 *     identityStoreId: example.identityStoreIds[0],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import an Identity Store Group using the combination `identity_store_id/group_id`. For example:
 *
 * ```sh
 * $ pulumi import aws:identitystore/group:Group example d-9c6705e95c/b8a1c340-8031-7071-a2fb-7dc540320c30
 * ```
 */
export declare class Group extends pulumi.CustomResource {
    /**
     * Get an existing Group resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: GroupState, opts?: pulumi.CustomResourceOptions): Group;
    /**
     * Returns true if the given object is an instance of Group.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Group;
    /**
     * ARN of the Group.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * A string containing the description of the group.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * A string containing the name of the group. This value is commonly displayed when the group is referenced.
     */
    readonly displayName: pulumi.Output<string>;
    /**
     * A list of external IDs that contains the identifiers issued to this resource by an external identity provider. See External IDs below.
     */
    readonly externalIds: pulumi.Output<outputs.identitystore.GroupExternalId[]>;
    /**
     * The identifier of the newly created group in the identity store.
     */
    readonly groupId: pulumi.Output<string>;
    /**
     * The globally unique identifier for the identity store.
     *
     * The following arguments are optional:
     */
    readonly identityStoreId: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Create a Group resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: GroupArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Group resources.
 */
export interface GroupState {
    /**
     * ARN of the Group.
     */
    arn?: pulumi.Input<string>;
    /**
     * A string containing the description of the group.
     */
    description?: pulumi.Input<string>;
    /**
     * A string containing the name of the group. This value is commonly displayed when the group is referenced.
     */
    displayName?: pulumi.Input<string>;
    /**
     * A list of external IDs that contains the identifiers issued to this resource by an external identity provider. See External IDs below.
     */
    externalIds?: pulumi.Input<pulumi.Input<inputs.identitystore.GroupExternalId>[]>;
    /**
     * The identifier of the newly created group in the identity store.
     */
    groupId?: pulumi.Input<string>;
    /**
     * The globally unique identifier for the identity store.
     *
     * The following arguments are optional:
     */
    identityStoreId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a Group resource.
 */
export interface GroupArgs {
    /**
     * A string containing the description of the group.
     */
    description?: pulumi.Input<string>;
    /**
     * A string containing the name of the group. This value is commonly displayed when the group is referenced.
     */
    displayName: pulumi.Input<string>;
    /**
     * The globally unique identifier for the identity store.
     *
     * The following arguments are optional:
     */
    identityStoreId: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
