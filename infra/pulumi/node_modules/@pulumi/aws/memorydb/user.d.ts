import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a MemoryDB User.
 *
 * More information about users and ACL-s can be found in the [MemoryDB User Guide](https://docs.aws.amazon.com/memorydb/latest/devguide/clusters.acls.html).
 *
 * > **Note:** All arguments including the username and passwords will be stored in the raw state as plain-text.
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 * import * as random from "@pulumi/random";
 *
 * const example = new random.index.Password("example", {length: 16});
 * const exampleUser = new aws.memorydb.User("example", {
 *     userName: "my-user",
 *     accessString: "on ~* &* +@all",
 *     authenticationMode: {
 *         type: "password",
 *         passwords: [example.result],
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import a user using the `user_name`. For example:
 *
 * ```sh
 * $ pulumi import aws:memorydb/user:User example my-user
 * ```
 * The `passwords` are not available for imported resources, as this information cannot be read back from the MemoryDB API.
 */
export declare class User extends pulumi.CustomResource {
    /**
     * Get an existing User resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: UserState, opts?: pulumi.CustomResourceOptions): User;
    /**
     * Returns true if the given object is an instance of User.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is User;
    /**
     * Access permissions string used for this user.
     */
    readonly accessString: pulumi.Output<string>;
    /**
     * ARN of the user.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Denotes the user's authentication properties. Detailed below.
     */
    readonly authenticationMode: pulumi.Output<outputs.memorydb.UserAuthenticationMode>;
    /**
     * Minimum engine version supported for the user.
     */
    readonly minimumEngineVersion: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
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
     * Name of the MemoryDB user. Up to 40 characters.
     *
     * The following arguments are optional:
     */
    readonly userName: pulumi.Output<string>;
    /**
     * Create a User resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: UserArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering User resources.
 */
export interface UserState {
    /**
     * Access permissions string used for this user.
     */
    accessString?: pulumi.Input<string>;
    /**
     * ARN of the user.
     */
    arn?: pulumi.Input<string>;
    /**
     * Denotes the user's authentication properties. Detailed below.
     */
    authenticationMode?: pulumi.Input<inputs.memorydb.UserAuthenticationMode>;
    /**
     * Minimum engine version supported for the user.
     */
    minimumEngineVersion?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
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
    /**
     * Name of the MemoryDB user. Up to 40 characters.
     *
     * The following arguments are optional:
     */
    userName?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a User resource.
 */
export interface UserArgs {
    /**
     * Access permissions string used for this user.
     */
    accessString: pulumi.Input<string>;
    /**
     * Denotes the user's authentication properties. Detailed below.
     */
    authenticationMode: pulumi.Input<inputs.memorydb.UserAuthenticationMode>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Name of the MemoryDB user. Up to 40 characters.
     *
     * The following arguments are optional:
     */
    userName: pulumi.Input<string>;
}
