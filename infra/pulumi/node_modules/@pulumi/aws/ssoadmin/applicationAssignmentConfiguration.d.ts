import * as pulumi from "@pulumi/pulumi";
/**
 * Resource for managing an AWS SSO Admin Application Assignment Configuration.
 *
 * By default, applications will require users to have an explicit assignment in order to access an application.
 * This resource can be used to adjust this default behavior if necessary.
 *
 * > Deleting this resource will return the assignment configuration for the application to the default AWS behavior (ie. `assignmentRequired = true`).
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.ssoadmin.ApplicationAssignmentConfiguration("example", {
 *     applicationArn: exampleAwsSsoadminApplication.arn,
 *     assignmentRequired: true,
 * });
 * ```
 *
 * ## Import
 *
 * ### Identity Schema
 *
 * #### Required
 *
 * - `arn` (String) Amazon Resource Name (ARN) of the SSO application.
 *
 * Using `pulumi import`, import SSO Admin Application Assignment Configuration using the `id`. For example:
 *
 * console
 *
 * % pulumi import aws_ssoadmin_application_assignment_configuration.example arn:aws:sso::123456789012:application/id-12345678
 */
export declare class ApplicationAssignmentConfiguration extends pulumi.CustomResource {
    /**
     * Get an existing ApplicationAssignmentConfiguration resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ApplicationAssignmentConfigurationState, opts?: pulumi.CustomResourceOptions): ApplicationAssignmentConfiguration;
    /**
     * Returns true if the given object is an instance of ApplicationAssignmentConfiguration.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ApplicationAssignmentConfiguration;
    /**
     * ARN of the application.
     */
    readonly applicationArn: pulumi.Output<string>;
    /**
     * Indicates whether users must have an explicit assignment to access the application. If `false`, all users have access to the application.
     */
    readonly assignmentRequired: pulumi.Output<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Create a ApplicationAssignmentConfiguration resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ApplicationAssignmentConfigurationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ApplicationAssignmentConfiguration resources.
 */
export interface ApplicationAssignmentConfigurationState {
    /**
     * ARN of the application.
     */
    applicationArn?: pulumi.Input<string>;
    /**
     * Indicates whether users must have an explicit assignment to access the application. If `false`, all users have access to the application.
     */
    assignmentRequired?: pulumi.Input<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a ApplicationAssignmentConfiguration resource.
 */
export interface ApplicationAssignmentConfigurationArgs {
    /**
     * ARN of the application.
     */
    applicationArn: pulumi.Input<string>;
    /**
     * Indicates whether users must have an explicit assignment to access the application. If `false`, all users have access to the application.
     */
    assignmentRequired: pulumi.Input<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
