import * as pulumi from "@pulumi/pulumi";
/**
 * Provides an environment member to an AWS Cloud9 development environment.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = new aws.cloud9.EnvironmentEC2("test", {
 *     instanceType: "t2.micro",
 *     name: "some-env",
 * });
 * const testUser = new aws.iam.User("test", {name: "some-user"});
 * const testEnvironmentMembership = new aws.cloud9.EnvironmentMembership("test", {
 *     environmentId: test.id,
 *     permissions: "read-only",
 *     userArn: testUser.arn,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Cloud9 environment membership using the `environment-id#user-arn`. For example:
 *
 * ```sh
 * $ pulumi import aws:cloud9/environmentMembership:EnvironmentMembership test environment-id#user-arn
 * ```
 */
export declare class EnvironmentMembership extends pulumi.CustomResource {
    /**
     * Get an existing EnvironmentMembership resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: EnvironmentMembershipState, opts?: pulumi.CustomResourceOptions): EnvironmentMembership;
    /**
     * Returns true if the given object is an instance of EnvironmentMembership.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is EnvironmentMembership;
    /**
     * The ID of the environment that contains the environment member you want to add.
     */
    readonly environmentId: pulumi.Output<string>;
    /**
     * The type of environment member permissions you want to associate with this environment member. Allowed values are `read-only` and `read-write` .
     */
    readonly permissions: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The Amazon Resource Name (ARN) of the environment member you want to add.
     */
    readonly userArn: pulumi.Output<string>;
    /**
     * The user ID in AWS Identity and Access Management (AWS IAM) of the environment member.
     */
    readonly userId: pulumi.Output<string>;
    /**
     * Create a EnvironmentMembership resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: EnvironmentMembershipArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering EnvironmentMembership resources.
 */
export interface EnvironmentMembershipState {
    /**
     * The ID of the environment that contains the environment member you want to add.
     */
    environmentId?: pulumi.Input<string>;
    /**
     * The type of environment member permissions you want to associate with this environment member. Allowed values are `read-only` and `read-write` .
     */
    permissions?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) of the environment member you want to add.
     */
    userArn?: pulumi.Input<string>;
    /**
     * The user ID in AWS Identity and Access Management (AWS IAM) of the environment member.
     */
    userId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a EnvironmentMembership resource.
 */
export interface EnvironmentMembershipArgs {
    /**
     * The ID of the environment that contains the environment member you want to add.
     */
    environmentId: pulumi.Input<string>;
    /**
     * The type of environment member permissions you want to associate with this environment member. Allowed values are `read-only` and `read-write` .
     */
    permissions: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) of the environment member you want to add.
     */
    userArn: pulumi.Input<string>;
}
