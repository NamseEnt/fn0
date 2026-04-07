import * as pulumi from "@pulumi/pulumi";
/**
 * Resource for managing QuickSight Group Membership
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.quicksight.GroupMembership("example", {
 *     groupName: "all-access-users",
 *     memberName: "john_smith",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import QuickSight Group membership using the AWS account ID, namespace, group name and member name separated by `/`. For example:
 *
 * ```sh
 * $ pulumi import aws:quicksight/groupMembership:GroupMembership example 123456789123/default/all-access-users/john_smith
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
    readonly arn: pulumi.Output<string>;
    readonly awsAccountId: pulumi.Output<string>;
    /**
     * The name of the group in which the member will be added.
     */
    readonly groupName: pulumi.Output<string>;
    /**
     * The name of the member to add to the group.
     */
    readonly memberName: pulumi.Output<string>;
    /**
     * The namespace that you want the user to be a part of. Defaults to `default`.
     */
    readonly namespace: pulumi.Output<string | undefined>;
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
    arn?: pulumi.Input<string>;
    awsAccountId?: pulumi.Input<string>;
    /**
     * The name of the group in which the member will be added.
     */
    groupName?: pulumi.Input<string>;
    /**
     * The name of the member to add to the group.
     */
    memberName?: pulumi.Input<string>;
    /**
     * The namespace that you want the user to be a part of. Defaults to `default`.
     */
    namespace?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a GroupMembership resource.
 */
export interface GroupMembershipArgs {
    awsAccountId?: pulumi.Input<string>;
    /**
     * The name of the group in which the member will be added.
     */
    groupName: pulumi.Input<string>;
    /**
     * The name of the member to add to the group.
     */
    memberName: pulumi.Input<string>;
    /**
     * The namespace that you want the user to be a part of. Defaults to `default`.
     */
    namespace?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
