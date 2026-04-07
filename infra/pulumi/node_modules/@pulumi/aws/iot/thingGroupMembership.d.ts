import * as pulumi from "@pulumi/pulumi";
/**
 * Adds an IoT Thing to an IoT Thing Group.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.iot.ThingGroupMembership("example", {
 *     thingName: "example-thing",
 *     thingGroupName: "example-group",
 *     overrideDynamicGroup: true,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import IoT Thing Group Membership using the thing group name and thing name. For example:
 *
 * ```sh
 * $ pulumi import aws:iot/thingGroupMembership:ThingGroupMembership example thing_group_name/thing_name
 * ```
 */
export declare class ThingGroupMembership extends pulumi.CustomResource {
    /**
     * Get an existing ThingGroupMembership resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ThingGroupMembershipState, opts?: pulumi.CustomResourceOptions): ThingGroupMembership;
    /**
     * Returns true if the given object is an instance of ThingGroupMembership.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ThingGroupMembership;
    /**
     * Override dynamic thing groups with static thing groups when 10-group limit is reached. If a thing belongs to 10 thing groups, and one or more of those groups are dynamic thing groups, adding a thing to a static group removes the thing from the last dynamic group.
     */
    readonly overrideDynamicGroup: pulumi.Output<boolean | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The name of the group to which you are adding a thing.
     */
    readonly thingGroupName: pulumi.Output<string>;
    /**
     * The name of the thing to add to a group.
     */
    readonly thingName: pulumi.Output<string>;
    /**
     * Create a ThingGroupMembership resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ThingGroupMembershipArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ThingGroupMembership resources.
 */
export interface ThingGroupMembershipState {
    /**
     * Override dynamic thing groups with static thing groups when 10-group limit is reached. If a thing belongs to 10 thing groups, and one or more of those groups are dynamic thing groups, adding a thing to a static group removes the thing from the last dynamic group.
     */
    overrideDynamicGroup?: pulumi.Input<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The name of the group to which you are adding a thing.
     */
    thingGroupName?: pulumi.Input<string>;
    /**
     * The name of the thing to add to a group.
     */
    thingName?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a ThingGroupMembership resource.
 */
export interface ThingGroupMembershipArgs {
    /**
     * Override dynamic thing groups with static thing groups when 10-group limit is reached. If a thing belongs to 10 thing groups, and one or more of those groups are dynamic thing groups, adding a thing to a static group removes the thing from the last dynamic group.
     */
    overrideDynamicGroup?: pulumi.Input<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The name of the group to which you are adding a thing.
     */
    thingGroupName: pulumi.Input<string>;
    /**
     * The name of the thing to add to a group.
     */
    thingName: pulumi.Input<string>;
}
