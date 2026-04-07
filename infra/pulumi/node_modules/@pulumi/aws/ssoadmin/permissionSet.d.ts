import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a Single Sign-On (SSO) Permission Set resource
 *
 * > **NOTE:** Updating this resource will automatically [Provision the Permission Set](https://docs.aws.amazon.com/singlesignon/latest/APIReference/API_ProvisionPermissionSet.html) to apply the corresponding updates to all assigned accounts.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.ssoadmin.getInstances({});
 * const examplePermissionSet = new aws.ssoadmin.PermissionSet("example", {
 *     name: "Example",
 *     description: "An example",
 *     instanceArn: example.then(example => example.arns?.[0]),
 *     relayState: "https://s3.console.aws.amazon.com/s3/home?region=us-east-1#",
 *     sessionDuration: "PT2H",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import SSO Permission Sets using the `arn` and `instance_arn` separated by a comma (`,`). For example:
 *
 * ```sh
 * $ pulumi import aws:ssoadmin/permissionSet:PermissionSet example arn:aws:sso:::permissionSet/ssoins-2938j0x8920sbj72/ps-80383020jr9302rk,arn:aws:sso:::instance/ssoins-2938j0x8920sbj72
 * ```
 */
export declare class PermissionSet extends pulumi.CustomResource {
    /**
     * Get an existing PermissionSet resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: PermissionSetState, opts?: pulumi.CustomResourceOptions): PermissionSet;
    /**
     * Returns true if the given object is an instance of PermissionSet.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is PermissionSet;
    /**
     * The Amazon Resource Name (ARN) of the Permission Set.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The date the Permission Set was created in [RFC3339 format](https://tools.ietf.org/html/rfc3339#section-5.8).
     */
    readonly createdDate: pulumi.Output<string>;
    /**
     * The description of the Permission Set.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * The Amazon Resource Name (ARN) of the SSO Instance under which the operation will be executed.
     */
    readonly instanceArn: pulumi.Output<string>;
    /**
     * The name of the Permission Set.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The relay state URL used to redirect users within the application during the federation authentication process.
     */
    readonly relayState: pulumi.Output<string | undefined>;
    /**
     * The length of time that the application user sessions are valid in the ISO-8601 standard. Default: `PT1H`.
     */
    readonly sessionDuration: pulumi.Output<string | undefined>;
    /**
     * Key-value map of resource tags. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Create a PermissionSet resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: PermissionSetArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering PermissionSet resources.
 */
export interface PermissionSetState {
    /**
     * The Amazon Resource Name (ARN) of the Permission Set.
     */
    arn?: pulumi.Input<string>;
    /**
     * The date the Permission Set was created in [RFC3339 format](https://tools.ietf.org/html/rfc3339#section-5.8).
     */
    createdDate?: pulumi.Input<string>;
    /**
     * The description of the Permission Set.
     */
    description?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) of the SSO Instance under which the operation will be executed.
     */
    instanceArn?: pulumi.Input<string>;
    /**
     * The name of the Permission Set.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The relay state URL used to redirect users within the application during the federation authentication process.
     */
    relayState?: pulumi.Input<string>;
    /**
     * The length of time that the application user sessions are valid in the ISO-8601 standard. Default: `PT1H`.
     */
    sessionDuration?: pulumi.Input<string>;
    /**
     * Key-value map of resource tags. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
 * The set of arguments for constructing a PermissionSet resource.
 */
export interface PermissionSetArgs {
    /**
     * The description of the Permission Set.
     */
    description?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) of the SSO Instance under which the operation will be executed.
     */
    instanceArn: pulumi.Input<string>;
    /**
     * The name of the Permission Set.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The relay state URL used to redirect users within the application during the federation authentication process.
     */
    relayState?: pulumi.Input<string>;
    /**
     * The length of time that the application user sessions are valid in the ISO-8601 standard. Default: `PT1H`.
     */
    sessionDuration?: pulumi.Input<string>;
    /**
     * Key-value map of resource tags. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
