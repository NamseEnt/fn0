import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides an AppConfig Extension resource.
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
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import AppConfig Extensions using their extension ID. For example:
 *
 * ```sh
 * $ pulumi import aws:appconfig/extension:Extension example 71rxuzt
 * ```
 */
export declare class Extension extends pulumi.CustomResource {
    /**
     * Get an existing Extension resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ExtensionState, opts?: pulumi.CustomResourceOptions): Extension;
    /**
     * Returns true if the given object is an instance of Extension.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Extension;
    /**
     * The action points defined in the extension. Detailed below.
     */
    readonly actionPoints: pulumi.Output<outputs.appconfig.ExtensionActionPoint[]>;
    /**
     * ARN of the AppConfig Extension.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Information about the extension.
     */
    readonly description: pulumi.Output<string>;
    /**
     * A name for the extension. Each extension name in your account must be unique. Extension versions use the same name.
     */
    readonly name: pulumi.Output<string>;
    /**
     * The parameters accepted by the extension. You specify parameter values when you associate the extension to an AppConfig resource by using the CreateExtensionAssociation API action. For Lambda extension actions, these parameters are included in the Lambda request object. Detailed below.
     */
    readonly parameters: pulumi.Output<outputs.appconfig.ExtensionParameter[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * The version number for the extension.
     */
    readonly version: pulumi.Output<number>;
    /**
     * Create a Extension resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ExtensionArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Extension resources.
 */
export interface ExtensionState {
    /**
     * The action points defined in the extension. Detailed below.
     */
    actionPoints?: pulumi.Input<pulumi.Input<inputs.appconfig.ExtensionActionPoint>[]>;
    /**
     * ARN of the AppConfig Extension.
     */
    arn?: pulumi.Input<string>;
    /**
     * Information about the extension.
     */
    description?: pulumi.Input<string>;
    /**
     * A name for the extension. Each extension name in your account must be unique. Extension versions use the same name.
     */
    name?: pulumi.Input<string>;
    /**
     * The parameters accepted by the extension. You specify parameter values when you associate the extension to an AppConfig resource by using the CreateExtensionAssociation API action. For Lambda extension actions, these parameters are included in the Lambda request object. Detailed below.
     */
    parameters?: pulumi.Input<pulumi.Input<inputs.appconfig.ExtensionParameter>[]>;
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
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The version number for the extension.
     */
    version?: pulumi.Input<number>;
}
/**
 * The set of arguments for constructing a Extension resource.
 */
export interface ExtensionArgs {
    /**
     * The action points defined in the extension. Detailed below.
     */
    actionPoints: pulumi.Input<pulumi.Input<inputs.appconfig.ExtensionActionPoint>[]>;
    /**
     * Information about the extension.
     */
    description?: pulumi.Input<string>;
    /**
     * A name for the extension. Each extension name in your account must be unique. Extension versions use the same name.
     */
    name?: pulumi.Input<string>;
    /**
     * The parameters accepted by the extension. You specify parameter values when you associate the extension to an AppConfig resource by using the CreateExtensionAssociation API action. For Lambda extension actions, these parameters are included in the Lambda request object. Detailed below.
     */
    parameters?: pulumi.Input<pulumi.Input<inputs.appconfig.ExtensionParameter>[]>;
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
}
