import * as pulumi from "@pulumi/pulumi";
/**
 * Resource for managing an AWS SSO Admin Application Access Scope.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.ssoadmin.getInstances({});
 * const exampleApplication = new aws.ssoadmin.Application("example", {
 *     name: "example",
 *     applicationProviderArn: "arn:aws:sso::aws:applicationProvider/custom",
 *     instanceArn: example.then(example => example.arns?.[0]),
 * });
 * const exampleApplicationAccessScope = new aws.ssoadmin.ApplicationAccessScope("example", {
 *     applicationArn: exampleApplication.arn,
 *     authorizedTargets: ["arn:aws:sso::123456789012:application/ssoins-123456789012/apl-123456789012"],
 *     scope: "sso:account:access",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import SSO Admin Application Access Scope using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:ssoadmin/applicationAccessScope:ApplicationAccessScope example arn:aws:sso::123456789012:application/ssoins-123456789012/apl-123456789012,sso:account:access
 * ```
 */
export declare class ApplicationAccessScope extends pulumi.CustomResource {
    /**
     * Get an existing ApplicationAccessScope resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ApplicationAccessScopeState, opts?: pulumi.CustomResourceOptions): ApplicationAccessScope;
    /**
     * Returns true if the given object is an instance of ApplicationAccessScope.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ApplicationAccessScope;
    /**
     * Specifies the ARN of the application with the access scope with the targets to add or update.
     */
    readonly applicationArn: pulumi.Output<string>;
    /**
     * Specifies an array list of ARNs that represent the authorized targets for this access scope.
     */
    readonly authorizedTargets: pulumi.Output<string[] | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Specifies the name of the access scope to be associated with the specified targets.
     *
     * The following arguments are optional:
     */
    readonly scope: pulumi.Output<string>;
    /**
     * Create a ApplicationAccessScope resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ApplicationAccessScopeArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ApplicationAccessScope resources.
 */
export interface ApplicationAccessScopeState {
    /**
     * Specifies the ARN of the application with the access scope with the targets to add or update.
     */
    applicationArn?: pulumi.Input<string>;
    /**
     * Specifies an array list of ARNs that represent the authorized targets for this access scope.
     */
    authorizedTargets?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Specifies the name of the access scope to be associated with the specified targets.
     *
     * The following arguments are optional:
     */
    scope?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a ApplicationAccessScope resource.
 */
export interface ApplicationAccessScopeArgs {
    /**
     * Specifies the ARN of the application with the access scope with the targets to add or update.
     */
    applicationArn: pulumi.Input<string>;
    /**
     * Specifies an array list of ARNs that represent the authorized targets for this access scope.
     */
    authorizedTargets?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Specifies the name of the access scope to be associated with the specified targets.
     *
     * The following arguments are optional:
     */
    scope: pulumi.Input<string>;
}
