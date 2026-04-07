import * as pulumi from "@pulumi/pulumi";
/**
 * Resource for managing an AWS Service Catalog AppRegistry Application.
 *
 * > An AWS Service Catalog AppRegistry Application is displayed in the AWS Console under "MyApplications".
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.servicecatalog.AppregistryApplication("example", {name: "example-app"});
 * ```
 *
 * ### Connecting Resources
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.servicecatalog.AppregistryApplication("example", {name: "example-app"});
 * const bucket = new aws.s3.Bucket("bucket", {
 *     bucket: "example-bucket",
 *     tags: example.applicationTag,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import AWS Service Catalog AppRegistry Application using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:servicecatalog/appregistryApplication:AppregistryApplication example application-id-12345678
 * ```
 */
export declare class AppregistryApplication extends pulumi.CustomResource {
    /**
     * Get an existing AppregistryApplication resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: AppregistryApplicationState, opts?: pulumi.CustomResourceOptions): AppregistryApplication;
    /**
     * Returns true if the given object is an instance of AppregistryApplication.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is AppregistryApplication;
    /**
     * A map with a single tag key-value pair used to associate resources with the application. This attribute can be passed directly into the `tags` argument of another resource, or merged into a map of existing tags.
     */
    readonly applicationTag: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * ARN (Amazon Resource Name) of the application.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Description of the application.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * Name of the application. The name must be unique within an AWS region.
     *
     * The following arguments are optional:
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * A map of tags assigned to the Application. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Create a AppregistryApplication resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: AppregistryApplicationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering AppregistryApplication resources.
 */
export interface AppregistryApplicationState {
    /**
     * A map with a single tag key-value pair used to associate resources with the application. This attribute can be passed directly into the `tags` argument of another resource, or merged into a map of existing tags.
     */
    applicationTag?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * ARN (Amazon Resource Name) of the application.
     */
    arn?: pulumi.Input<string>;
    /**
     * Description of the application.
     */
    description?: pulumi.Input<string>;
    /**
     * Name of the application. The name must be unique within an AWS region.
     *
     * The following arguments are optional:
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A map of tags assigned to the Application. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
 * The set of arguments for constructing a AppregistryApplication resource.
 */
export interface AppregistryApplicationArgs {
    /**
     * Description of the application.
     */
    description?: pulumi.Input<string>;
    /**
     * Name of the application. The name must be unique within an AWS region.
     *
     * The following arguments are optional:
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A map of tags assigned to the Application. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
