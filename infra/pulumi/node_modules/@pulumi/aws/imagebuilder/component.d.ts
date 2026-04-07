import * as pulumi from "@pulumi/pulumi";
/**
 * Manages an Image Builder Component.
 *
 * ## Example Usage
 *
 * ### URI Document
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.imagebuilder.Component("example", {
 *     name: "example",
 *     platform: "Linux",
 *     uri: `s3://${exampleAwsS3Object.bucket}/${exampleAwsS3Object.key}`,
 *     version: "1.0.0",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_imagebuilder_components` resources using the Amazon Resource Name (ARN). For example:
 *
 * ```sh
 * $ pulumi import aws:imagebuilder/component:Component example arn:aws:imagebuilder:us-east-1:123456789012:component/example/1.0.0/1
 * ```
 * Certain resource arguments, such as `uri`, cannot be read via the API and imported into the provider. The provider will display a difference for these arguments the first run after import if declared in the the provider configuration for an imported resource.
 */
export declare class Component extends pulumi.CustomResource {
    /**
     * Get an existing Component resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ComponentState, opts?: pulumi.CustomResourceOptions): Component;
    /**
     * Returns true if the given object is an instance of Component.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Component;
    /**
     * (Required) Amazon Resource Name (ARN) of the component.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Change description of the component.
     */
    readonly changeDescription: pulumi.Output<string | undefined>;
    /**
     * Inline YAML string with data of the component. Exactly one of `data` and `uri` can be specified. the provider will only perform drift detection of its value when present in a configuration.
     */
    readonly data: pulumi.Output<string>;
    /**
     * Date the component was created.
     */
    readonly dateCreated: pulumi.Output<string>;
    /**
     * Description of the component.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * Encryption status of the component.
     */
    readonly encrypted: pulumi.Output<boolean>;
    /**
     * Amazon Resource Name (ARN) of the Key Management Service (KMS) Key used to encrypt the component.
     */
    readonly kmsKeyId: pulumi.Output<string | undefined>;
    /**
     * Name of the component.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Owner of the component.
     */
    readonly owner: pulumi.Output<string>;
    /**
     * Platform of the component.
     */
    readonly platform: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Whether to retain the old version when the resource is destroyed or replacement is necessary. Defaults to `false`.
     */
    readonly skipDestroy: pulumi.Output<boolean | undefined>;
    /**
     * Set of Operating Systems (OS) supported by the component.
     */
    readonly supportedOsVersions: pulumi.Output<string[] | undefined>;
    /**
     * Key-value map of resource tags for the component. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Type of the component.
     */
    readonly type: pulumi.Output<string>;
    /**
     * S3 URI with data of the component. Exactly one of `data` and `uri` can be specified.
     *
     * > **NOTE:** Updating `data` or `uri` requires specifying a new `version`. This causes replacement of the resource. The `skipDestroy` argument can be used to retain the old version.
     */
    readonly uri: pulumi.Output<string | undefined>;
    /**
     * Version of the component.
     *
     * The following arguments are optional:
     */
    readonly version: pulumi.Output<string>;
    /**
     * Create a Component resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ComponentArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Component resources.
 */
export interface ComponentState {
    /**
     * (Required) Amazon Resource Name (ARN) of the component.
     */
    arn?: pulumi.Input<string>;
    /**
     * Change description of the component.
     */
    changeDescription?: pulumi.Input<string>;
    /**
     * Inline YAML string with data of the component. Exactly one of `data` and `uri` can be specified. the provider will only perform drift detection of its value when present in a configuration.
     */
    data?: pulumi.Input<string>;
    /**
     * Date the component was created.
     */
    dateCreated?: pulumi.Input<string>;
    /**
     * Description of the component.
     */
    description?: pulumi.Input<string>;
    /**
     * Encryption status of the component.
     */
    encrypted?: pulumi.Input<boolean>;
    /**
     * Amazon Resource Name (ARN) of the Key Management Service (KMS) Key used to encrypt the component.
     */
    kmsKeyId?: pulumi.Input<string>;
    /**
     * Name of the component.
     */
    name?: pulumi.Input<string>;
    /**
     * Owner of the component.
     */
    owner?: pulumi.Input<string>;
    /**
     * Platform of the component.
     */
    platform?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Whether to retain the old version when the resource is destroyed or replacement is necessary. Defaults to `false`.
     */
    skipDestroy?: pulumi.Input<boolean>;
    /**
     * Set of Operating Systems (OS) supported by the component.
     */
    supportedOsVersions?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Key-value map of resource tags for the component. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Type of the component.
     */
    type?: pulumi.Input<string>;
    /**
     * S3 URI with data of the component. Exactly one of `data` and `uri` can be specified.
     *
     * > **NOTE:** Updating `data` or `uri` requires specifying a new `version`. This causes replacement of the resource. The `skipDestroy` argument can be used to retain the old version.
     */
    uri?: pulumi.Input<string>;
    /**
     * Version of the component.
     *
     * The following arguments are optional:
     */
    version?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a Component resource.
 */
export interface ComponentArgs {
    /**
     * Change description of the component.
     */
    changeDescription?: pulumi.Input<string>;
    /**
     * Inline YAML string with data of the component. Exactly one of `data` and `uri` can be specified. the provider will only perform drift detection of its value when present in a configuration.
     */
    data?: pulumi.Input<string>;
    /**
     * Description of the component.
     */
    description?: pulumi.Input<string>;
    /**
     * Amazon Resource Name (ARN) of the Key Management Service (KMS) Key used to encrypt the component.
     */
    kmsKeyId?: pulumi.Input<string>;
    /**
     * Name of the component.
     */
    name?: pulumi.Input<string>;
    /**
     * Platform of the component.
     */
    platform: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Whether to retain the old version when the resource is destroyed or replacement is necessary. Defaults to `false`.
     */
    skipDestroy?: pulumi.Input<boolean>;
    /**
     * Set of Operating Systems (OS) supported by the component.
     */
    supportedOsVersions?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Key-value map of resource tags for the component. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * S3 URI with data of the component. Exactly one of `data` and `uri` can be specified.
     *
     * > **NOTE:** Updating `data` or `uri` requires specifying a new `version`. This causes replacement of the resource. The `skipDestroy` argument can be used to retain the old version.
     */
    uri?: pulumi.Input<string>;
    /**
     * Version of the component.
     *
     * The following arguments are optional:
     */
    version: pulumi.Input<string>;
}
