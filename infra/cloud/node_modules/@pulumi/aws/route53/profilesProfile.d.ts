import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS Route 53 Profile.
 *
 * ## Example Usage
 *
 * ### Empty Profile
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.route53.ProfilesProfile("example", {
 *     name: "example",
 *     tags: {
 *         Environment: "dev",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Route 53 Profiles Profile using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:route53/profilesProfile:ProfilesProfile example rp-12345678
 * ```
 */
export declare class ProfilesProfile extends pulumi.CustomResource {
    /**
     * Get an existing ProfilesProfile resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ProfilesProfileState, opts?: pulumi.CustomResourceOptions): ProfilesProfile;
    /**
     * Returns true if the given object is an instance of ProfilesProfile.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ProfilesProfile;
    /**
     * ARN of the Profile.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Name of the Profile.
     */
    readonly name: pulumi.Output<string>;
    readonly ownerId: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Share status of the Profile.
     */
    readonly shareStatus: pulumi.Output<string>;
    /**
     * Status of the Profile.
     */
    readonly status: pulumi.Output<string>;
    /**
     * Status message of the Profile.
     */
    readonly statusMessage: pulumi.Output<string>;
    /**
     * Map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * Map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    readonly timeouts: pulumi.Output<outputs.route53.ProfilesProfileTimeouts | undefined>;
    /**
     * Create a ProfilesProfile resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: ProfilesProfileArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ProfilesProfile resources.
 */
export interface ProfilesProfileState {
    /**
     * ARN of the Profile.
     */
    arn?: pulumi.Input<string>;
    /**
     * Name of the Profile.
     */
    name?: pulumi.Input<string>;
    ownerId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Share status of the Profile.
     */
    shareStatus?: pulumi.Input<string>;
    /**
     * Status of the Profile.
     */
    status?: pulumi.Input<string>;
    /**
     * Status message of the Profile.
     */
    statusMessage?: pulumi.Input<string>;
    /**
     * Map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    timeouts?: pulumi.Input<inputs.route53.ProfilesProfileTimeouts>;
}
/**
 * The set of arguments for constructing a ProfilesProfile resource.
 */
export interface ProfilesProfileArgs {
    /**
     * Name of the Profile.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    timeouts?: pulumi.Input<inputs.route53.ProfilesProfileTimeouts>;
}
