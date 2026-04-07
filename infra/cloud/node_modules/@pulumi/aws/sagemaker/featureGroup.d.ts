import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a SageMaker AI Feature Group resource.
 *
 * ## Example Usage
 *
 * Basic usage:
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.sagemaker.FeatureGroup("example", {
 *     featureGroupName: "example",
 *     recordIdentifierFeatureName: "example",
 *     eventTimeFeatureName: "example",
 *     roleArn: test.arn,
 *     featureDefinitions: [{
 *         featureName: "example",
 *         featureType: "String",
 *     }],
 *     onlineStoreConfig: {
 *         enableOnlineStore: true,
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Feature Groups using the `name`. For example:
 *
 * ```sh
 * $ pulumi import aws:sagemaker/featureGroup:FeatureGroup test_feature_group feature_group-foo
 * ```
 */
export declare class FeatureGroup extends pulumi.CustomResource {
    /**
     * Get an existing FeatureGroup resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: FeatureGroupState, opts?: pulumi.CustomResourceOptions): FeatureGroup;
    /**
     * Returns true if the given object is an instance of FeatureGroup.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is FeatureGroup;
    /**
     * The Amazon Resource Name (ARN) assigned by AWS to this feature_group.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * A free-form description of a Feature Group.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * The name of the feature that stores the EventTime of a Record in a Feature Group.
     */
    readonly eventTimeFeatureName: pulumi.Output<string>;
    /**
     * A list of Feature names and types. See Feature Definition Below.
     */
    readonly featureDefinitions: pulumi.Output<outputs.sagemaker.FeatureGroupFeatureDefinition[]>;
    /**
     * The name of the Feature Group. The name must be unique within an AWS Region in an AWS account.
     */
    readonly featureGroupName: pulumi.Output<string>;
    /**
     * The Offline Feature Store Configuration. See Offline Store Config Below.
     */
    readonly offlineStoreConfig: pulumi.Output<outputs.sagemaker.FeatureGroupOfflineStoreConfig | undefined>;
    /**
     * The Online Feature Store Configuration. See Online Store Config Below.
     */
    readonly onlineStoreConfig: pulumi.Output<outputs.sagemaker.FeatureGroupOnlineStoreConfig | undefined>;
    /**
     * The name of the Feature whose value uniquely identifies a Record defined in the Feature Store. Only the latest record per identifier value will be stored in the Online Store.
     */
    readonly recordIdentifierFeatureName: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The Amazon Resource Name (ARN) of the IAM execution role used to persist data into the Offline Store if an `offlineStoreConfig` is provided.
     */
    readonly roleArn: pulumi.Output<string>;
    /**
     * Map of resource tags for the resource. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
    readonly throughputConfig: pulumi.Output<outputs.sagemaker.FeatureGroupThroughputConfig>;
    /**
     * Create a FeatureGroup resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: FeatureGroupArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering FeatureGroup resources.
 */
export interface FeatureGroupState {
    /**
     * The Amazon Resource Name (ARN) assigned by AWS to this feature_group.
     */
    arn?: pulumi.Input<string>;
    /**
     * A free-form description of a Feature Group.
     */
    description?: pulumi.Input<string>;
    /**
     * The name of the feature that stores the EventTime of a Record in a Feature Group.
     */
    eventTimeFeatureName?: pulumi.Input<string>;
    /**
     * A list of Feature names and types. See Feature Definition Below.
     */
    featureDefinitions?: pulumi.Input<pulumi.Input<inputs.sagemaker.FeatureGroupFeatureDefinition>[]>;
    /**
     * The name of the Feature Group. The name must be unique within an AWS Region in an AWS account.
     */
    featureGroupName?: pulumi.Input<string>;
    /**
     * The Offline Feature Store Configuration. See Offline Store Config Below.
     */
    offlineStoreConfig?: pulumi.Input<inputs.sagemaker.FeatureGroupOfflineStoreConfig>;
    /**
     * The Online Feature Store Configuration. See Online Store Config Below.
     */
    onlineStoreConfig?: pulumi.Input<inputs.sagemaker.FeatureGroupOnlineStoreConfig>;
    /**
     * The name of the Feature whose value uniquely identifies a Record defined in the Feature Store. Only the latest record per identifier value will be stored in the Online Store.
     */
    recordIdentifierFeatureName?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) of the IAM execution role used to persist data into the Offline Store if an `offlineStoreConfig` is provided.
     */
    roleArn?: pulumi.Input<string>;
    /**
     * Map of resource tags for the resource. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
    throughputConfig?: pulumi.Input<inputs.sagemaker.FeatureGroupThroughputConfig>;
}
/**
 * The set of arguments for constructing a FeatureGroup resource.
 */
export interface FeatureGroupArgs {
    /**
     * A free-form description of a Feature Group.
     */
    description?: pulumi.Input<string>;
    /**
     * The name of the feature that stores the EventTime of a Record in a Feature Group.
     */
    eventTimeFeatureName: pulumi.Input<string>;
    /**
     * A list of Feature names and types. See Feature Definition Below.
     */
    featureDefinitions: pulumi.Input<pulumi.Input<inputs.sagemaker.FeatureGroupFeatureDefinition>[]>;
    /**
     * The name of the Feature Group. The name must be unique within an AWS Region in an AWS account.
     */
    featureGroupName: pulumi.Input<string>;
    /**
     * The Offline Feature Store Configuration. See Offline Store Config Below.
     */
    offlineStoreConfig?: pulumi.Input<inputs.sagemaker.FeatureGroupOfflineStoreConfig>;
    /**
     * The Online Feature Store Configuration. See Online Store Config Below.
     */
    onlineStoreConfig?: pulumi.Input<inputs.sagemaker.FeatureGroupOnlineStoreConfig>;
    /**
     * The name of the Feature whose value uniquely identifies a Record defined in the Feature Store. Only the latest record per identifier value will be stored in the Online Store.
     */
    recordIdentifierFeatureName: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) of the IAM execution role used to persist data into the Offline Store if an `offlineStoreConfig` is provided.
     */
    roleArn: pulumi.Input<string>;
    /**
     * Map of resource tags for the resource. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    throughputConfig?: pulumi.Input<inputs.sagemaker.FeatureGroupThroughputConfig>;
}
