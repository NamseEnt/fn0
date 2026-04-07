import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides an ElastiCache user resource.
 *
 * > **Note:** All arguments including the username and passwords will be stored in the raw state as plain-text.
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = new aws.elasticache.User("test", {
 *     userId: "testUserId",
 *     userName: "testUserName",
 *     accessString: "on ~app::* -@all +@read +@hash +@bitmap +@geo -setbit -bitfield -hset -hsetnx -hmset -hincrby -hincrbyfloat -hdel -bitop -geoadd -georadius -georadiusbymember",
 *     engine: "redis",
 *     passwords: ["password123456789"],
 * });
 * ```
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = new aws.elasticache.User("test", {
 *     userId: "testUserId",
 *     userName: "testUserName",
 *     accessString: "on ~* +@all",
 *     engine: "redis",
 *     authenticationMode: {
 *         type: "iam",
 *     },
 * });
 * ```
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = new aws.elasticache.User("test", {
 *     userId: "testUserId",
 *     userName: "testUserName",
 *     accessString: "on ~* +@all",
 *     engine: "redis",
 *     authenticationMode: {
 *         type: "password",
 *         passwords: [
 *             "password1",
 *             "password2",
 *         ],
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import ElastiCache users using the `user_id`. For example:
 *
 * ```sh
 * $ pulumi import aws:elasticache/user:User my_user userId1
 * ```
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
     * Access permissions string used for this user. See [Specifying Permissions Using an Access String](https://docs.aws.amazon.com/AmazonElastiCache/latest/red-ug/Clusters.RBAC.html#Access-string) for more details.
     */
    readonly accessString: pulumi.Output<string>;
    /**
     * The ARN of the created ElastiCache User.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Denotes the user's authentication properties. Detailed below.
     */
    readonly authenticationMode: pulumi.Output<outputs.elasticache.UserAuthenticationMode>;
    /**
     * The current supported values are `redis`, `valkey` (case insensitive).
     */
    readonly engine: pulumi.Output<string>;
    /**
     * Indicates a password is not required for this user.
     */
    readonly noPasswordRequired: pulumi.Output<boolean | undefined>;
    /**
     * Passwords used for this user. You can create up to two passwords for each user.
     */
    readonly passwords: pulumi.Output<string[] | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * A list of tags to be added to this resource. A tag is a key-value pair.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * The ID of the user.
     */
    readonly userId: pulumi.Output<string>;
    /**
     * The username of the user.
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
     * Access permissions string used for this user. See [Specifying Permissions Using an Access String](https://docs.aws.amazon.com/AmazonElastiCache/latest/red-ug/Clusters.RBAC.html#Access-string) for more details.
     */
    accessString?: pulumi.Input<string>;
    /**
     * The ARN of the created ElastiCache User.
     */
    arn?: pulumi.Input<string>;
    /**
     * Denotes the user's authentication properties. Detailed below.
     */
    authenticationMode?: pulumi.Input<inputs.elasticache.UserAuthenticationMode>;
    /**
     * The current supported values are `redis`, `valkey` (case insensitive).
     */
    engine?: pulumi.Input<string>;
    /**
     * Indicates a password is not required for this user.
     */
    noPasswordRequired?: pulumi.Input<boolean>;
    /**
     * Passwords used for this user. You can create up to two passwords for each user.
     */
    passwords?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A list of tags to be added to this resource. A tag is a key-value pair.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The ID of the user.
     */
    userId?: pulumi.Input<string>;
    /**
     * The username of the user.
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
     * Access permissions string used for this user. See [Specifying Permissions Using an Access String](https://docs.aws.amazon.com/AmazonElastiCache/latest/red-ug/Clusters.RBAC.html#Access-string) for more details.
     */
    accessString: pulumi.Input<string>;
    /**
     * Denotes the user's authentication properties. Detailed below.
     */
    authenticationMode?: pulumi.Input<inputs.elasticache.UserAuthenticationMode>;
    /**
     * The current supported values are `redis`, `valkey` (case insensitive).
     */
    engine: pulumi.Input<string>;
    /**
     * Indicates a password is not required for this user.
     */
    noPasswordRequired?: pulumi.Input<boolean>;
    /**
     * Passwords used for this user. You can create up to two passwords for each user.
     */
    passwords?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A list of tags to be added to this resource. A tag is a key-value pair.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The ID of the user.
     */
    userId: pulumi.Input<string>;
    /**
     * The username of the user.
     *
     * The following arguments are optional:
     */
    userName: pulumi.Input<string>;
}
