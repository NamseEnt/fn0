import * as pulumi from "@pulumi/pulumi";
/**
 * Associates an AppConfig Extension with a Resource.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const testTopic = new aws.sns.Topic("test", {name: "test"});
 * const test = aws.iam.getPolicyDocument({
 *     statements: [{
 *         actions: ["sts:AssumeRole"],
 *         principals: [{
 *             type: "Service",
 *             identifiers: ["appconfig.amazonaws.com"],
 *         }],
 *     }],
 * });
 * const testRole = new aws.iam.Role("test", {
 *     name: "test",
 *     assumeRolePolicy: test.then(test => test.json),
 * });
 * const testExtension = new aws.appconfig.Extension("test", {
 *     name: "test",
 *     description: "test description",
 *     actionPoints: [{
 *         point: "ON_DEPLOYMENT_COMPLETE",
 *         actions: [{
 *             name: "test",
 *             roleArn: testRole.arn,
 *             uri: testTopic.arn,
 *         }],
 *     }],
 *     tags: {
 *         Type: "AppConfig Extension",
 *     },
 * });
 * const testApplication = new aws.appconfig.Application("test", {name: "test"});
 * const testExtensionAssociation = new aws.appconfig.ExtensionAssociation("test", {
 *     extensionArn: testExtension.arn,
 *     resourceArn: testApplication.arn,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import AppConfig Extension Associations using their extension association ID. For example:
 *
 * ```sh
 * $ pulumi import aws:appconfig/extensionAssociation:ExtensionAssociation example 71rxuzt
 * ```
 */
export declare class ExtensionAssociation extends pulumi.CustomResource {
    /**
     * Get an existing ExtensionAssociation resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ExtensionAssociationState, opts?: pulumi.CustomResourceOptions): ExtensionAssociation;
    /**
     * Returns true if the given object is an instance of ExtensionAssociation.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ExtensionAssociation;
    /**
     * ARN of the AppConfig Extension Association.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The ARN of the extension defined in the association.
     */
    readonly extensionArn: pulumi.Output<string>;
    /**
     * The version number for the extension defined in the association.
     */
    readonly extensionVersion: pulumi.Output<number>;
    /**
     * The parameter names and values defined for the association.
     */
    readonly parameters: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The ARN of the application, configuration profile, or environment to associate with the extension.
     */
    readonly resourceArn: pulumi.Output<string>;
    /**
     * Create a ExtensionAssociation resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ExtensionAssociationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ExtensionAssociation resources.
 */
export interface ExtensionAssociationState {
    /**
     * ARN of the AppConfig Extension Association.
     */
    arn?: pulumi.Input<string>;
    /**
     * The ARN of the extension defined in the association.
     */
    extensionArn?: pulumi.Input<string>;
    /**
     * The version number for the extension defined in the association.
     */
    extensionVersion?: pulumi.Input<number>;
    /**
     * The parameter names and values defined for the association.
     */
    parameters?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The ARN of the application, configuration profile, or environment to associate with the extension.
     */
    resourceArn?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a ExtensionAssociation resource.
 */
export interface ExtensionAssociationArgs {
    /**
     * The ARN of the extension defined in the association.
     */
    extensionArn: pulumi.Input<string>;
    /**
     * The parameter names and values defined for the association.
     */
    parameters?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The ARN of the application, configuration profile, or environment to associate with the extension.
     */
    resourceArn: pulumi.Input<string>;
}
