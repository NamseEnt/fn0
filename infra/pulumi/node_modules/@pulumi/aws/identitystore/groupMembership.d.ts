import * as pulumi from "@pulumi/pulumi";
/**
 * Resource for managing an AWS IdentityStore Group Membership.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.ssoadmin.getInstances({});
 * const exampleUser = new aws.identitystore.User("example", {
 *     identityStoreId: example.then(example => example.identityStoreIds?.[0]),
 *     displayName: "John Doe",
 *     userName: "john.doe@example.com",
 *     name: {
 *         familyName: "Doe",
 *         givenName: "John",
 *     },
 * });
 * const exampleGroup = new aws.identitystore.Group("example", {
 *     identityStoreId: example.then(example => example.identityStoreIds?.[0]),
 *     displayName: "MyGroup",
 *     description: "Some group name",
 * });
 * const exampleGroupMembership = new aws.identitystore.GroupMembership("example", {
 *     identityStoreId: example.then(example => example.identityStoreIds?.[0]),
 *     groupId: exampleGroup.groupId,
 *     memberId: exampleUser.userId,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_identitystore_group_membership` using the `identity_store_id/membership_id`. For example:
 *
 * ```sh
 * $ pulumi import aws:identitystore/groupMembership:GroupMembership example d-0000000000/00000000-0000-0000-0000-000000000000
 * ```
 */
export declare class GroupMembership extends pulumi.CustomResource {
    /**
     * Get an existing GroupMembership resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: GroupMembershipState, opts?: pulumi.CustomResourceOptions): GroupMembership;
    /**
     * Returns true if the given object is an instance of GroupMembership.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is GroupMembership;
    /**
     * The identifier for a group in the Identity Store.
     */
    readonly groupId: pulumi.Output<string>;
    /**
     * Identity Store ID associated with the Single Sign-On Instance.
     */
    readonly identityStoreId: pulumi.Output<string>;
    /**
     * The identifier for a user in the Identity Store.
     */
    readonly memberId: pulumi.Output<string>;
    /**
     * The identifier of the newly created group membership in the Identity Store.
     */
    readonly membershipId: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Create a GroupMembership resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: GroupMembershipArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering GroupMembership resources.
 */
export interface GroupMembershipState {
    /**
     * The identifier for a group in the Identity Store.
     */
    groupId?: pulumi.Input<string>;
    /**
     * Identity Store ID associated with the Single Sign-On Instance.
     */
    identityStoreId?: pulumi.Input<string>;
    /**
     * The identifier for a user in the Identity Store.
     */
    memberId?: pulumi.Input<string>;
    /**
     * The identifier of the newly created group membership in the Identity Store.
     */
    membershipId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a GroupMembership resource.
 */
export interface GroupMembershipArgs {
    /**
     * The identifier for a group in the Identity Store.
     */
    groupId: pulumi.Input<string>;
    /**
     * Identity Store ID associated with the Single Sign-On Instance.
     */
    identityStoreId: pulumi.Input<string>;
    /**
     * The identifier for a user in the Identity Store.
     */
    memberId: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
