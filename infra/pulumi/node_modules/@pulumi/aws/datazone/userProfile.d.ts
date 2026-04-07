import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS DataZone User Profile.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.datazone.UserProfile("example", {
 *     userIdentifier: exampleAwsIamUser.arn,
 *     domainIdentifier: exampleAwsDatazoneDomain.id,
 *     userType: "IAM_USER",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import DataZone User Profile using the `user_identifier,domain_identifier,type`. For example:
 *
 * ```sh
 * $ pulumi import aws:datazone/userProfile:UserProfile example arn:aws:iam::123456789012:user/example,dzd_54nakfrg9k6suo,IAM
 * ```
 */
export declare class UserProfile extends pulumi.CustomResource {
    /**
     * Get an existing UserProfile resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: UserProfileState, opts?: pulumi.CustomResourceOptions): UserProfile;
    /**
     * Returns true if the given object is an instance of UserProfile.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is UserProfile;
    /**
     * Details about the user profile.
     */
    readonly details: pulumi.Output<outputs.datazone.UserProfileDetail[]>;
    /**
     * The domain identifier.
     */
    readonly domainIdentifier: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The user profile status.
     */
    readonly status: pulumi.Output<string>;
    readonly timeouts: pulumi.Output<outputs.datazone.UserProfileTimeouts | undefined>;
    /**
     * The user profile type.
     */
    readonly type: pulumi.Output<string>;
    /**
     * The user identifier.
     *
     * The following arguments are optional:
     */
    readonly userIdentifier: pulumi.Output<string>;
    /**
     * The user type.
     */
    readonly userType: pulumi.Output<string>;
    /**
     * Create a UserProfile resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: UserProfileArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering UserProfile resources.
 */
export interface UserProfileState {
    /**
     * Details about the user profile.
     */
    details?: pulumi.Input<pulumi.Input<inputs.datazone.UserProfileDetail>[]>;
    /**
     * The domain identifier.
     */
    domainIdentifier?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The user profile status.
     */
    status?: pulumi.Input<string>;
    timeouts?: pulumi.Input<inputs.datazone.UserProfileTimeouts>;
    /**
     * The user profile type.
     */
    type?: pulumi.Input<string>;
    /**
     * The user identifier.
     *
     * The following arguments are optional:
     */
    userIdentifier?: pulumi.Input<string>;
    /**
     * The user type.
     */
    userType?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a UserProfile resource.
 */
export interface UserProfileArgs {
    /**
     * The domain identifier.
     */
    domainIdentifier: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The user profile status.
     */
    status?: pulumi.Input<string>;
    timeouts?: pulumi.Input<inputs.datazone.UserProfileTimeouts>;
    /**
     * The user identifier.
     *
     * The following arguments are optional:
     */
    userIdentifier: pulumi.Input<string>;
    /**
     * The user type.
     */
    userType?: pulumi.Input<string>;
}
