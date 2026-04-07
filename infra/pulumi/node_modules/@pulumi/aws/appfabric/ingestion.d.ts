import * as pulumi from "@pulumi/pulumi";
/**
 * Resource for managing an AWS AppFabric Ingestion.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.appfabric.Ingestion("example", {
 *     app: "OKTA",
 *     appBundleArn: exampleAwsAppfabricAppBundle.arn,
 *     tenantId: "example.okta.com",
 *     ingestionType: "auditLog",
 *     tags: {
 *         Environment: "test",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import AppFabric Ingestion using the `app_bundle_identifier` and `arn` separated by `,`. For example:
 *
 * ```sh
 * $ pulumi import aws:appfabric/ingestion:Ingestion example arn:aws:appfabric:[region]:[account]:appbundle/a9b91477-8831-43c0-970c-xxxxxxxxxx,arn:aws:appfabric:[region]:[account]:appbundle/a9b91477-8831-43c0-970c-xxxxxxxxxx/ingestion/32251416-710b-4425-96ca-xxxxxxxxxx
 * ```
 */
export declare class Ingestion extends pulumi.CustomResource {
    /**
     * Get an existing Ingestion resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: IngestionState, opts?: pulumi.CustomResourceOptions): Ingestion;
    /**
     * Returns true if the given object is an instance of Ingestion.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Ingestion;
    /**
     * Name of the application.
     * Refer to the AWS Documentation for the [list of valid values](https://docs.aws.amazon.com/appfabric/latest/api/API_CreateIngestion.html#appfabric-CreateIngestion-request-app)
     */
    readonly app: pulumi.Output<string>;
    /**
     * Amazon Resource Name (ARN) of the app bundle to use for the request.
     */
    readonly appBundleArn: pulumi.Output<string>;
    /**
     * ARN of the Ingestion.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Ingestion type. Valid values are `auditLog`.
     */
    readonly ingestionType: pulumi.Output<string>;
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
    /**
     * Map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * ID of the application tenant.
     */
    readonly tenantId: pulumi.Output<string>;
    /**
     * Create a Ingestion resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: IngestionArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Ingestion resources.
 */
export interface IngestionState {
    /**
     * Name of the application.
     * Refer to the AWS Documentation for the [list of valid values](https://docs.aws.amazon.com/appfabric/latest/api/API_CreateIngestion.html#appfabric-CreateIngestion-request-app)
     */
    app?: pulumi.Input<string>;
    /**
     * Amazon Resource Name (ARN) of the app bundle to use for the request.
     */
    appBundleArn?: pulumi.Input<string>;
    /**
     * ARN of the Ingestion.
     */
    arn?: pulumi.Input<string>;
    /**
     * Ingestion type. Valid values are `auditLog`.
     */
    ingestionType?: pulumi.Input<string>;
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
    /**
     * Map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * ID of the application tenant.
     */
    tenantId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a Ingestion resource.
 */
export interface IngestionArgs {
    /**
     * Name of the application.
     * Refer to the AWS Documentation for the [list of valid values](https://docs.aws.amazon.com/appfabric/latest/api/API_CreateIngestion.html#appfabric-CreateIngestion-request-app)
     */
    app: pulumi.Input<string>;
    /**
     * Amazon Resource Name (ARN) of the app bundle to use for the request.
     */
    appBundleArn: pulumi.Input<string>;
    /**
     * Ingestion type. Valid values are `auditLog`.
     */
    ingestionType: pulumi.Input<string>;
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
    /**
     * ID of the application tenant.
     */
    tenantId: pulumi.Input<string>;
}
