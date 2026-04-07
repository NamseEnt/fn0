import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a Glue Schema resource.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.glue.Schema("example", {
 *     schemaName: "example",
 *     registryArn: test.arn,
 *     dataFormat: "AVRO",
 *     compatibility: "NONE",
 *     schemaDefinition: "{\"type\": \"record\", \"name\": \"r1\", \"fields\": [ {\"name\": \"f1\", \"type\": \"int\"}, {\"name\": \"f2\", \"type\": \"string\"} ]}",
 * });
 * ```
 *
 * ## Import
 *
 * ### Identity Schema
 *
 * #### Required
 *
 * - `arn` (String) Amazon Resource Name (ARN) of the Glue schema.
 *
 * Using `pulumi import`, import Glue Registries using `arn`. For example:
 *
 * console
 *
 * % pulumi import aws_glue_schema.example arn:aws:glue:us-west-2:123456789012:schema/example/example
 */
export declare class Schema extends pulumi.CustomResource {
    /**
     * Get an existing Schema resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: SchemaState, opts?: pulumi.CustomResourceOptions): Schema;
    /**
     * Returns true if the given object is an instance of Schema.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Schema;
    /**
     * Amazon Resource Name (ARN) of the schema.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The compatibility mode of the schema. Values values are: `NONE`, `DISABLED`, `BACKWARD`, `BACKWARD_ALL`, `FORWARD`, `FORWARD_ALL`, `FULL`, and `FULL_ALL`.
     */
    readonly compatibility: pulumi.Output<string>;
    /**
     * The data format of the schema definition. Valid values are `AVRO`, `JSON` and `PROTOBUF`.
     */
    readonly dataFormat: pulumi.Output<string>;
    /**
     * A description of the schema.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * The latest version of the schema associated with the returned schema definition.
     */
    readonly latestSchemaVersion: pulumi.Output<number>;
    /**
     * The next version of the schema associated with the returned schema definition.
     */
    readonly nextSchemaVersion: pulumi.Output<number>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The ARN of the Glue Registry to create the schema in.
     */
    readonly registryArn: pulumi.Output<string>;
    /**
     * The name of the Glue Registry.
     */
    readonly registryName: pulumi.Output<string>;
    /**
     * The version number of the checkpoint (the last time the compatibility mode was changed).
     */
    readonly schemaCheckpoint: pulumi.Output<number>;
    /**
     * The schema definition using the `dataFormat` setting for `schemaName`.
     */
    readonly schemaDefinition: pulumi.Output<string>;
    /**
     * The Name of the schema.
     */
    readonly schemaName: pulumi.Output<string>;
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
     * Create a Schema resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: SchemaArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Schema resources.
 */
export interface SchemaState {
    /**
     * Amazon Resource Name (ARN) of the schema.
     */
    arn?: pulumi.Input<string>;
    /**
     * The compatibility mode of the schema. Values values are: `NONE`, `DISABLED`, `BACKWARD`, `BACKWARD_ALL`, `FORWARD`, `FORWARD_ALL`, `FULL`, and `FULL_ALL`.
     */
    compatibility?: pulumi.Input<string>;
    /**
     * The data format of the schema definition. Valid values are `AVRO`, `JSON` and `PROTOBUF`.
     */
    dataFormat?: pulumi.Input<string>;
    /**
     * A description of the schema.
     */
    description?: pulumi.Input<string>;
    /**
     * The latest version of the schema associated with the returned schema definition.
     */
    latestSchemaVersion?: pulumi.Input<number>;
    /**
     * The next version of the schema associated with the returned schema definition.
     */
    nextSchemaVersion?: pulumi.Input<number>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The ARN of the Glue Registry to create the schema in.
     */
    registryArn?: pulumi.Input<string>;
    /**
     * The name of the Glue Registry.
     */
    registryName?: pulumi.Input<string>;
    /**
     * The version number of the checkpoint (the last time the compatibility mode was changed).
     */
    schemaCheckpoint?: pulumi.Input<number>;
    /**
     * The schema definition using the `dataFormat` setting for `schemaName`.
     */
    schemaDefinition?: pulumi.Input<string>;
    /**
     * The Name of the schema.
     */
    schemaName?: pulumi.Input<string>;
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
 * The set of arguments for constructing a Schema resource.
 */
export interface SchemaArgs {
    /**
     * The compatibility mode of the schema. Values values are: `NONE`, `DISABLED`, `BACKWARD`, `BACKWARD_ALL`, `FORWARD`, `FORWARD_ALL`, `FULL`, and `FULL_ALL`.
     */
    compatibility: pulumi.Input<string>;
    /**
     * The data format of the schema definition. Valid values are `AVRO`, `JSON` and `PROTOBUF`.
     */
    dataFormat: pulumi.Input<string>;
    /**
     * A description of the schema.
     */
    description?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The ARN of the Glue Registry to create the schema in.
     */
    registryArn?: pulumi.Input<string>;
    /**
     * The schema definition using the `dataFormat` setting for `schemaName`.
     */
    schemaDefinition: pulumi.Input<string>;
    /**
     * The Name of the schema.
     */
    schemaName: pulumi.Input<string>;
    /**
     * Key-value map of resource tags. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
