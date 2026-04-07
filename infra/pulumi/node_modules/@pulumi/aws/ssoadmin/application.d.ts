import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS SSO Admin Application.
 *
 * > The `CreateApplication` API only supports custom OAuth 2.0 applications.
 * Creation of 3rd party SAML or OAuth 2.0 applications require setup to be done through the associated app service or AWS console.
 * See this issue for additional context.
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
 * ```
 *
 * ### With Portal Options
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
 *     portalOptions: {
 *         visibility: "ENABLED",
 *         signInOptions: {
 *             applicationUrl: "http://example.com",
 *             origin: "APPLICATION",
 *         },
 *     },
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
 * Using `pulumi import`, import SSO Admin Application using the `id`. For example:
 *
 * console
 *
 * % pulumi import aws_ssoadmin_application.example arn:aws:sso::123456789012:application/id-12345678
 */
export declare class Application extends pulumi.CustomResource {
    /**
     * Get an existing Application resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ApplicationState, opts?: pulumi.CustomResourceOptions): Application;
    /**
     * Returns true if the given object is an instance of Application.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Application;
    /**
     * AWS account ID.
     */
    readonly applicationAccount: pulumi.Output<string>;
    /**
     * (**Deprecated** Reference `arn` instead) ARN of the application.
     *
     * @deprecated Use 'arn' instead. This attribute will be removed in a future verion of the provider.
     */
    readonly applicationArn: pulumi.Output<string>;
    /**
     * ARN of the application provider.
     */
    readonly applicationProviderArn: pulumi.Output<string>;
    /**
     * ARN of the application.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * A unique, case-sensitive ID that you provide to ensure the idempotency of the request. AWS generates a random value when not provided.
     */
    readonly clientToken: pulumi.Output<string | undefined>;
    /**
     * Description of the application.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * ARN of the instance of IAM Identity Center.
     */
    readonly instanceArn: pulumi.Output<string>;
    /**
     * Name of the application.
     *
     * The following arguments are optional:
     */
    readonly name: pulumi.Output<string>;
    /**
     * Options for the portal associated with an application. See `portalOptions` below.
     */
    readonly portalOptions: pulumi.Output<outputs.ssoadmin.ApplicationPortalOptions | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Status of the application. Valid values are `ENABLED` and `DISABLED`.
     */
    readonly status: pulumi.Output<string>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * Map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * Create a Application resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ApplicationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Application resources.
 */
export interface ApplicationState {
    /**
     * AWS account ID.
     */
    applicationAccount?: pulumi.Input<string>;
    /**
     * (**Deprecated** Reference `arn` instead) ARN of the application.
     *
     * @deprecated Use 'arn' instead. This attribute will be removed in a future verion of the provider.
     */
    applicationArn?: pulumi.Input<string>;
    /**
     * ARN of the application provider.
     */
    applicationProviderArn?: pulumi.Input<string>;
    /**
     * ARN of the application.
     */
    arn?: pulumi.Input<string>;
    /**
     * A unique, case-sensitive ID that you provide to ensure the idempotency of the request. AWS generates a random value when not provided.
     */
    clientToken?: pulumi.Input<string>;
    /**
     * Description of the application.
     */
    description?: pulumi.Input<string>;
    /**
     * ARN of the instance of IAM Identity Center.
     */
    instanceArn?: pulumi.Input<string>;
    /**
     * Name of the application.
     *
     * The following arguments are optional:
     */
    name?: pulumi.Input<string>;
    /**
     * Options for the portal associated with an application. See `portalOptions` below.
     */
    portalOptions?: pulumi.Input<inputs.ssoadmin.ApplicationPortalOptions>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Status of the application. Valid values are `ENABLED` and `DISABLED`.
     */
    status?: pulumi.Input<string>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
/**
 * The set of arguments for constructing a Application resource.
 */
export interface ApplicationArgs {
    /**
     * ARN of the application provider.
     */
    applicationProviderArn: pulumi.Input<string>;
    /**
     * A unique, case-sensitive ID that you provide to ensure the idempotency of the request. AWS generates a random value when not provided.
     */
    clientToken?: pulumi.Input<string>;
    /**
     * Description of the application.
     */
    description?: pulumi.Input<string>;
    /**
     * ARN of the instance of IAM Identity Center.
     */
    instanceArn: pulumi.Input<string>;
    /**
     * Name of the application.
     *
     * The following arguments are optional:
     */
    name?: pulumi.Input<string>;
    /**
     * Options for the portal associated with an application. See `portalOptions` below.
     */
    portalOptions?: pulumi.Input<inputs.ssoadmin.ApplicationPortalOptions>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Status of the application. Valid values are `ENABLED` and `DISABLED`.
     */
    status?: pulumi.Input<string>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
