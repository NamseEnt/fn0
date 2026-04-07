import * as pulumi from "@pulumi/pulumi";
/**
 * Manages an AppStream User Stack association.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = new aws.appstream.Stack("test", {name: "STACK NAME"});
 * const testUser = new aws.appstream.User("test", {
 *     authenticationType: "USERPOOL",
 *     userName: "EMAIL",
 * });
 * const testUserStackAssociation = new aws.appstream.UserStackAssociation("test", {
 *     authenticationType: testUser.authenticationType,
 *     stackName: test.name,
 *     userName: testUser.userName,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import AppStream User Stack Association using the `user_name`, `authentication_type`, and `stack_name`, separated by a slash (`/`). For example:
 *
 * ```sh
 * $ pulumi import aws:appstream/userStackAssociation:UserStackAssociation example userName/auhtenticationType/stackName
 * ```
 */
export declare class UserStackAssociation extends pulumi.CustomResource {
    /**
     * Get an existing UserStackAssociation resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: UserStackAssociationState, opts?: pulumi.CustomResourceOptions): UserStackAssociation;
    /**
     * Returns true if the given object is an instance of UserStackAssociation.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is UserStackAssociation;
    /**
     * Authentication type for the user.
     */
    readonly authenticationType: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Whether a welcome email is sent to a user after the user is created in the user pool.
     */
    readonly sendEmailNotification: pulumi.Output<boolean | undefined>;
    /**
     * Name of the stack that is associated with the user.
     */
    readonly stackName: pulumi.Output<string>;
    /**
     * Email address of the user who is associated with the stack.
     *
     * The following arguments are optional:
     */
    readonly userName: pulumi.Output<string>;
    /**
     * Create a UserStackAssociation resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: UserStackAssociationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering UserStackAssociation resources.
 */
export interface UserStackAssociationState {
    /**
     * Authentication type for the user.
     */
    authenticationType?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Whether a welcome email is sent to a user after the user is created in the user pool.
     */
    sendEmailNotification?: pulumi.Input<boolean>;
    /**
     * Name of the stack that is associated with the user.
     */
    stackName?: pulumi.Input<string>;
    /**
     * Email address of the user who is associated with the stack.
     *
     * The following arguments are optional:
     */
    userName?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a UserStackAssociation resource.
 */
export interface UserStackAssociationArgs {
    /**
     * Authentication type for the user.
     */
    authenticationType: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Whether a welcome email is sent to a user after the user is created in the user pool.
     */
    sendEmailNotification?: pulumi.Input<boolean>;
    /**
     * Name of the stack that is associated with the user.
     */
    stackName: pulumi.Input<string>;
    /**
     * Email address of the user who is associated with the stack.
     *
     * The following arguments are optional:
     */
    userName: pulumi.Input<string>;
}
