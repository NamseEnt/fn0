import * as pulumi from "@pulumi/pulumi";
/**
 * Associate an existing ElastiCache user and an existing user group.
 *
 * > Pulumi will detect changes in the `aws.elasticache.UserGroup` since `aws.elasticache.UserGroupAssociation` changes the user IDs associated with the user group. You can ignore these changes with the `lifecycle` `ignoreChanges` meta argument as shown in the example.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const _default = new aws.elasticache.User("default", {
 *     userId: "defaultUserID",
 *     userName: "default",
 *     accessString: "on ~app::* -@all +@read +@hash +@bitmap +@geo -setbit -bitfield -hset -hsetnx -hmset -hincrby -hincrbyfloat -hdel -bitop -geoadd -georadius -georadiusbymember",
 *     engine: "REDIS",
 *     passwords: ["password123456789"],
 * });
 * const example = new aws.elasticache.UserGroup("example", {
 *     engine: "REDIS",
 *     userGroupId: "userGroupId",
 *     userIds: [_default.userId],
 * });
 * const exampleUser = new aws.elasticache.User("example", {
 *     userId: "exampleUserID",
 *     userName: "exampleuser",
 *     accessString: "on ~app::* -@all +@read +@hash +@bitmap +@geo -setbit -bitfield -hset -hsetnx -hmset -hincrby -hincrbyfloat -hdel -bitop -geoadd -georadius -georadiusbymember",
 *     engine: "REDIS",
 *     passwords: ["password123456789"],
 * });
 * const exampleUserGroupAssociation = new aws.elasticache.UserGroupAssociation("example", {
 *     userGroupId: example.userGroupId,
 *     userId: exampleUser.userId,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import ElastiCache user group associations using the `user_group_id` and `user_id`. For example:
 *
 * ```sh
 * $ pulumi import aws:elasticache/userGroupAssociation:UserGroupAssociation example userGoupId1,userId
 * ```
 */
export declare class UserGroupAssociation extends pulumi.CustomResource {
    /**
     * Get an existing UserGroupAssociation resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: UserGroupAssociationState, opts?: pulumi.CustomResourceOptions): UserGroupAssociation;
    /**
     * Returns true if the given object is an instance of UserGroupAssociation.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is UserGroupAssociation;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * ID of the user group.
     */
    readonly userGroupId: pulumi.Output<string>;
    /**
     * ID of the user to associated with the user group.
     */
    readonly userId: pulumi.Output<string>;
    /**
     * Create a UserGroupAssociation resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: UserGroupAssociationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering UserGroupAssociation resources.
 */
export interface UserGroupAssociationState {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ID of the user group.
     */
    userGroupId?: pulumi.Input<string>;
    /**
     * ID of the user to associated with the user group.
     */
    userId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a UserGroupAssociation resource.
 */
export interface UserGroupAssociationArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ID of the user group.
     */
    userGroupId: pulumi.Input<string>;
    /**
     * ID of the user to associated with the user group.
     */
    userId: pulumi.Input<string>;
}
