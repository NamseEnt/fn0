import * as pulumi from "@pulumi/pulumi";
/**
 * Enable (Opt-In) or Disable (Opt-Out) a particular Region for an AWS account.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.account.Region("example", {
 *     regionName: "ap-southeast-3",
 *     enabled: true,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`. For example:
 *
 * ```sh
 * $ pulumi import aws:account/region:Region example ap-southeast-3
 * ```
 */
export declare class Region extends pulumi.CustomResource {
    /**
     * Get an existing Region resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: RegionState, opts?: pulumi.CustomResourceOptions): Region;
    /**
     * Returns true if the given object is an instance of Region.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Region;
    /**
     * The ID of the target account when managing member accounts. Will manage current user's account by default if omitted. To use this parameter, the caller must be an identity in the organization's management account or a delegated administrator account. The specified account ID must also be a member account in the same organization. The organization must have all features enabled, and the organization must have trusted access enabled for the Account Management service, and optionally a delegated admin account assigned.
     */
    readonly accountId: pulumi.Output<string | undefined>;
    /**
     * Whether the region is enabled.
     */
    readonly enabled: pulumi.Output<boolean>;
    /**
     * The region opt status.
     */
    readonly optStatus: pulumi.Output<string>;
    /**
     * The region name to manage.
     */
    readonly regionName: pulumi.Output<string>;
    /**
     * Create a Region resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: RegionArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Region resources.
 */
export interface RegionState {
    /**
     * The ID of the target account when managing member accounts. Will manage current user's account by default if omitted. To use this parameter, the caller must be an identity in the organization's management account or a delegated administrator account. The specified account ID must also be a member account in the same organization. The organization must have all features enabled, and the organization must have trusted access enabled for the Account Management service, and optionally a delegated admin account assigned.
     */
    accountId?: pulumi.Input<string>;
    /**
     * Whether the region is enabled.
     */
    enabled?: pulumi.Input<boolean>;
    /**
     * The region opt status.
     */
    optStatus?: pulumi.Input<string>;
    /**
     * The region name to manage.
     */
    regionName?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a Region resource.
 */
export interface RegionArgs {
    /**
     * The ID of the target account when managing member accounts. Will manage current user's account by default if omitted. To use this parameter, the caller must be an identity in the organization's management account or a delegated administrator account. The specified account ID must also be a member account in the same organization. The organization must have all features enabled, and the organization must have trusted access enabled for the Account Management service, and optionally a delegated admin account assigned.
     */
    accountId?: pulumi.Input<string>;
    /**
     * Whether the region is enabled.
     */
    enabled: pulumi.Input<boolean>;
    /**
     * The region name to manage.
     */
    regionName: pulumi.Input<string>;
}
