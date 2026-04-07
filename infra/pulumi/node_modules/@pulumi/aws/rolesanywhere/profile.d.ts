import * as pulumi from "@pulumi/pulumi";
/**
 * Resource for managing a Roles Anywhere Profile.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = new aws.iam.Role("test", {
 *     name: "test",
 *     path: "/",
 *     assumeRolePolicy: JSON.stringify({
 *         Version: "2012-10-17",
 *         Statement: [{
 *             Action: [
 *                 "sts:AssumeRole",
 *                 "sts:TagSession",
 *                 "sts:SetSourceIdentity",
 *             ],
 *             Principal: {
 *                 Service: "rolesanywhere.amazonaws.com",
 *             },
 *             Effect: "Allow",
 *             Sid: "",
 *         }],
 *     }),
 * });
 * const testProfile = new aws.rolesanywhere.Profile("test", {
 *     name: "example",
 *     roleArns: [test.arn],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_rolesanywhere_profile` using its `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:rolesanywhere/profile:Profile example db138a85-8925-4f9f-a409-08231233cacf
 * ```
 */
export declare class Profile extends pulumi.CustomResource {
    /**
     * Get an existing Profile resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ProfileState, opts?: pulumi.CustomResourceOptions): Profile;
    /**
     * Returns true if the given object is an instance of Profile.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Profile;
    /**
     * Amazon Resource Name (ARN) of the Profile
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The number of seconds the vended session credentials are valid for. Defaults to 3600.
     */
    readonly durationSeconds: pulumi.Output<number>;
    /**
     * Whether or not the Profile is enabled.
     */
    readonly enabled: pulumi.Output<boolean | undefined>;
    /**
     * A list of managed policy ARNs that apply to the vended session credentials.
     */
    readonly managedPolicyArns: pulumi.Output<string[] | undefined>;
    /**
     * The name of the Profile.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Specifies whether instance properties are required in [CreateSession](https://docs.aws.amazon.com/rolesanywhere/latest/APIReference/API_CreateSession.html) requests with this profile.
     */
    readonly requireInstanceProperties: pulumi.Output<boolean | undefined>;
    /**
     * A list of IAM roles that this profile can assume
     */
    readonly roleArns: pulumi.Output<string[] | undefined>;
    /**
     * A session policy that applies to the trust boundary of the vended session credentials.
     */
    readonly sessionPolicy: pulumi.Output<string | undefined>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Create a Profile resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: ProfileArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Profile resources.
 */
export interface ProfileState {
    /**
     * Amazon Resource Name (ARN) of the Profile
     */
    arn?: pulumi.Input<string>;
    /**
     * The number of seconds the vended session credentials are valid for. Defaults to 3600.
     */
    durationSeconds?: pulumi.Input<number>;
    /**
     * Whether or not the Profile is enabled.
     */
    enabled?: pulumi.Input<boolean>;
    /**
     * A list of managed policy ARNs that apply to the vended session credentials.
     */
    managedPolicyArns?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The name of the Profile.
     */
    name?: pulumi.Input<string>;
    /**
     * Specifies whether instance properties are required in [CreateSession](https://docs.aws.amazon.com/rolesanywhere/latest/APIReference/API_CreateSession.html) requests with this profile.
     */
    requireInstanceProperties?: pulumi.Input<boolean>;
    /**
     * A list of IAM roles that this profile can assume
     */
    roleArns?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * A session policy that applies to the trust boundary of the vended session credentials.
     */
    sessionPolicy?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
 * The set of arguments for constructing a Profile resource.
 */
export interface ProfileArgs {
    /**
     * The number of seconds the vended session credentials are valid for. Defaults to 3600.
     */
    durationSeconds?: pulumi.Input<number>;
    /**
     * Whether or not the Profile is enabled.
     */
    enabled?: pulumi.Input<boolean>;
    /**
     * A list of managed policy ARNs that apply to the vended session credentials.
     */
    managedPolicyArns?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The name of the Profile.
     */
    name?: pulumi.Input<string>;
    /**
     * Specifies whether instance properties are required in [CreateSession](https://docs.aws.amazon.com/rolesanywhere/latest/APIReference/API_CreateSession.html) requests with this profile.
     */
    requireInstanceProperties?: pulumi.Input<boolean>;
    /**
     * A list of IAM roles that this profile can assume
     */
    roleArns?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * A session policy that applies to the trust boundary of the vended session credentials.
     */
    sessionPolicy?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
