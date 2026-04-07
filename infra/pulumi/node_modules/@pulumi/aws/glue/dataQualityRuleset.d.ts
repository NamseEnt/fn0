import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a Glue Data Quality Ruleset Resource. You can refer to the [Glue Developer Guide](https://docs.aws.amazon.com/glue/latest/dg/glue-data-quality.html) for a full explanation of the Glue Data Quality Ruleset functionality
 *
 * ## Example Usage
 *
 * ### Basic
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.glue.DataQualityRuleset("example", {
 *     name: "example",
 *     ruleset: "Rules = [Completeness \"colA\" between 0.4 and 0.8]",
 * });
 * ```
 *
 * ### With description
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.glue.DataQualityRuleset("example", {
 *     name: "example",
 *     description: "example",
 *     ruleset: "Rules = [Completeness \"colA\" between 0.4 and 0.8]",
 * });
 * ```
 *
 * ### With tags
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.glue.DataQualityRuleset("example", {
 *     name: "example",
 *     ruleset: "Rules = [Completeness \"colA\" between 0.4 and 0.8]",
 *     tags: {
 *         hello: "world",
 *     },
 * });
 * ```
 *
 * ### With targetTable
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.glue.DataQualityRuleset("example", {
 *     name: "example",
 *     ruleset: "Rules = [Completeness \"colA\" between 0.4 and 0.8]",
 *     targetTable: {
 *         databaseName: exampleAwsGlueCatalogDatabase.name,
 *         tableName: exampleAwsGlueCatalogTable.name,
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Glue Data Quality Ruleset using the `name`. For example:
 *
 * ```sh
 * $ pulumi import aws:glue/dataQualityRuleset:DataQualityRuleset example exampleName
 * ```
 */
export declare class DataQualityRuleset extends pulumi.CustomResource {
    /**
     * Get an existing DataQualityRuleset resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: DataQualityRulesetState, opts?: pulumi.CustomResourceOptions): DataQualityRuleset;
    /**
     * Returns true if the given object is an instance of DataQualityRuleset.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is DataQualityRuleset;
    /**
     * ARN of the Glue Data Quality Ruleset.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The time and date that this data quality ruleset was created.
     */
    readonly createdOn: pulumi.Output<string>;
    /**
     * Description of the data quality ruleset.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * The time and date that this data quality ruleset was created.
     */
    readonly lastModifiedOn: pulumi.Output<string>;
    /**
     * Name of the data quality ruleset.
     */
    readonly name: pulumi.Output<string>;
    /**
     * When a ruleset was created from a recommendation run, this run ID is generated to link the two together.
     */
    readonly recommendationRunId: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * A Data Quality Definition Language (DQDL) ruleset. For more information, see the AWS Glue developer guide.
     */
    readonly ruleset: pulumi.Output<string>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * A Configuration block specifying a target table associated with the data quality ruleset. See `targetTable` below.
     */
    readonly targetTable: pulumi.Output<outputs.glue.DataQualityRulesetTargetTable | undefined>;
    /**
     * Create a DataQualityRuleset resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: DataQualityRulesetArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering DataQualityRuleset resources.
 */
export interface DataQualityRulesetState {
    /**
     * ARN of the Glue Data Quality Ruleset.
     */
    arn?: pulumi.Input<string>;
    /**
     * The time and date that this data quality ruleset was created.
     */
    createdOn?: pulumi.Input<string>;
    /**
     * Description of the data quality ruleset.
     */
    description?: pulumi.Input<string>;
    /**
     * The time and date that this data quality ruleset was created.
     */
    lastModifiedOn?: pulumi.Input<string>;
    /**
     * Name of the data quality ruleset.
     */
    name?: pulumi.Input<string>;
    /**
     * When a ruleset was created from a recommendation run, this run ID is generated to link the two together.
     */
    recommendationRunId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A Data Quality Definition Language (DQDL) ruleset. For more information, see the AWS Glue developer guide.
     */
    ruleset?: pulumi.Input<string>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * A Configuration block specifying a target table associated with the data quality ruleset. See `targetTable` below.
     */
    targetTable?: pulumi.Input<inputs.glue.DataQualityRulesetTargetTable>;
}
/**
 * The set of arguments for constructing a DataQualityRuleset resource.
 */
export interface DataQualityRulesetArgs {
    /**
     * Description of the data quality ruleset.
     */
    description?: pulumi.Input<string>;
    /**
     * Name of the data quality ruleset.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A Data Quality Definition Language (DQDL) ruleset. For more information, see the AWS Glue developer guide.
     */
    ruleset: pulumi.Input<string>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * A Configuration block specifying a target table associated with the data quality ruleset. See `targetTable` below.
     */
    targetTable?: pulumi.Input<inputs.glue.DataQualityRulesetTargetTable>;
}
