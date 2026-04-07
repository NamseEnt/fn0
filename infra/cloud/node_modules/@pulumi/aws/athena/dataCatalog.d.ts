import * as pulumi from "@pulumi/pulumi";
/**
 * Provides an Athena data catalog.
 *
 * More information about Athena and Athena data catalogs can be found in the [Athena User Guide](https://docs.aws.amazon.com/athena/latest/ug/what-is.html).
 *
 * > **Tip:** for a more detailed explanation on the usage of `parameters`, see the [DataCatalog API documentation](https://docs.aws.amazon.com/athena/latest/APIReference/API_DataCatalog.html)
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.athena.DataCatalog("example", {
 *     name: "athena-data-catalog",
 *     description: "Example Athena data catalog",
 *     type: "LAMBDA",
 *     parameters: {
 *         "function": "arn:aws:lambda:eu-central-1:123456789012:function:not-important-lambda-function",
 *     },
 *     tags: {
 *         Name: "example-athena-data-catalog",
 *     },
 * });
 * ```
 *
 * ### Hive based Data Catalog
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.athena.DataCatalog("example", {
 *     name: "hive-data-catalog",
 *     description: "Hive based Data Catalog",
 *     type: "HIVE",
 *     parameters: {
 *         "metadata-function": "arn:aws:lambda:eu-central-1:123456789012:function:not-important-lambda-function",
 *     },
 * });
 * ```
 *
 * ### Glue based Data Catalog
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.athena.DataCatalog("example", {
 *     name: "glue-data-catalog",
 *     description: "Glue based Data Catalog",
 *     type: "GLUE",
 *     parameters: {
 *         "catalog-id": "123456789012",
 *     },
 * });
 * ```
 *
 * ### Lambda based Data Catalog
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.athena.DataCatalog("example", {
 *     name: "lambda-data-catalog",
 *     description: "Lambda based Data Catalog",
 *     type: "LAMBDA",
 *     parameters: {
 *         "metadata-function": "arn:aws:lambda:eu-central-1:123456789012:function:not-important-lambda-function-1",
 *         "record-function": "arn:aws:lambda:eu-central-1:123456789012:function:not-important-lambda-function-2",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import data catalogs using their `name`. For example:
 *
 * ```sh
 * $ pulumi import aws:athena/dataCatalog:DataCatalog example example-data-catalog
 * ```
 */
export declare class DataCatalog extends pulumi.CustomResource {
    /**
     * Get an existing DataCatalog resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: DataCatalogState, opts?: pulumi.CustomResourceOptions): DataCatalog;
    /**
     * Returns true if the given object is an instance of DataCatalog.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is DataCatalog;
    /**
     * ARN of the data catalog.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Description of the data catalog.
     */
    readonly description: pulumi.Output<string>;
    /**
     * Name of the data catalog. The catalog name must be unique for the AWS account and can use a maximum of 128 alphanumeric, underscore, at sign, or hyphen characters.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Key value pairs that specifies the Lambda function or functions to use for the data catalog. The mapping used depends on the catalog type.
     */
    readonly parameters: pulumi.Output<{
        [key: string]: string;
    }>;
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
     * Type of data catalog: `LAMBDA` for a federated catalog, `GLUE` for AWS Glue Catalog, or `HIVE` for an external hive metastore.
     */
    readonly type: pulumi.Output<string>;
    /**
     * Create a DataCatalog resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: DataCatalogArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering DataCatalog resources.
 */
export interface DataCatalogState {
    /**
     * ARN of the data catalog.
     */
    arn?: pulumi.Input<string>;
    /**
     * Description of the data catalog.
     */
    description?: pulumi.Input<string>;
    /**
     * Name of the data catalog. The catalog name must be unique for the AWS account and can use a maximum of 128 alphanumeric, underscore, at sign, or hyphen characters.
     */
    name?: pulumi.Input<string>;
    /**
     * Key value pairs that specifies the Lambda function or functions to use for the data catalog. The mapping used depends on the catalog type.
     */
    parameters?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
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
     * Type of data catalog: `LAMBDA` for a federated catalog, `GLUE` for AWS Glue Catalog, or `HIVE` for an external hive metastore.
     */
    type?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a DataCatalog resource.
 */
export interface DataCatalogArgs {
    /**
     * Description of the data catalog.
     */
    description: pulumi.Input<string>;
    /**
     * Name of the data catalog. The catalog name must be unique for the AWS account and can use a maximum of 128 alphanumeric, underscore, at sign, or hyphen characters.
     */
    name?: pulumi.Input<string>;
    /**
     * Key value pairs that specifies the Lambda function or functions to use for the data catalog. The mapping used depends on the catalog type.
     */
    parameters: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
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
     * Type of data catalog: `LAMBDA` for a federated catalog, `GLUE` for AWS Glue Catalog, or `HIVE` for an external hive metastore.
     */
    type: pulumi.Input<string>;
}
