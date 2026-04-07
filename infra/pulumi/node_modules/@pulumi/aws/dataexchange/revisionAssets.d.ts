import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing AWS Data Exchange Revision Assets.
 *
 * > Note: This resource creates a new revision and adds associated assets. Destroying this resource will delete the revision and all associated assets.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.dataexchange.RevisionAssets("example", {
 *     dataSetId: "example-data-set-id",
 *     assets: [{
 *         createS3DataAccessFromS3Bucket: {
 *             assetSource: {
 *                 bucket: "example-bucket",
 *             },
 *         },
 *     }],
 *     tags: {
 *         Environment: "Production",
 *     },
 * });
 * ```
 */
export declare class RevisionAssets extends pulumi.CustomResource {
    /**
     * Get an existing RevisionAssets resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: RevisionAssetsState, opts?: pulumi.CustomResourceOptions): RevisionAssets;
    /**
     * Returns true if the given object is an instance of RevisionAssets.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is RevisionAssets;
    /**
     * The ARN of the Data Exchange Revision Assets.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * A block to define the asset associated with the revision. See Asset for more details.
     *
     * The following arguments are optional:
     */
    readonly assets: pulumi.Output<outputs.dataexchange.RevisionAssetsAsset[] | undefined>;
    /**
     * A comment for the revision. Maximum length is 16,348 characters.
     */
    readonly comment: pulumi.Output<string | undefined>;
    /**
     * The timestamp when the revision was created, in RFC3339 format.
     */
    readonly createdAt: pulumi.Output<string>;
    /**
     * Unique identifier for the data set associated with the revision.
     */
    readonly dataSetId: pulumi.Output<string>;
    readonly finalized: pulumi.Output<boolean>;
    readonly forceDestroy: pulumi.Output<boolean | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
    readonly timeouts: pulumi.Output<outputs.dataexchange.RevisionAssetsTimeouts | undefined>;
    /**
     * The timestamp when the revision was last updated, in RFC3339 format.
     */
    readonly updatedAt: pulumi.Output<string>;
    /**
     * Create a RevisionAssets resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: RevisionAssetsArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering RevisionAssets resources.
 */
export interface RevisionAssetsState {
    /**
     * The ARN of the Data Exchange Revision Assets.
     */
    arn?: pulumi.Input<string>;
    /**
     * A block to define the asset associated with the revision. See Asset for more details.
     *
     * The following arguments are optional:
     */
    assets?: pulumi.Input<pulumi.Input<inputs.dataexchange.RevisionAssetsAsset>[]>;
    /**
     * A comment for the revision. Maximum length is 16,348 characters.
     */
    comment?: pulumi.Input<string>;
    /**
     * The timestamp when the revision was created, in RFC3339 format.
     */
    createdAt?: pulumi.Input<string>;
    /**
     * Unique identifier for the data set associated with the revision.
     */
    dataSetId?: pulumi.Input<string>;
    finalized?: pulumi.Input<boolean>;
    forceDestroy?: pulumi.Input<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
    timeouts?: pulumi.Input<inputs.dataexchange.RevisionAssetsTimeouts>;
    /**
     * The timestamp when the revision was last updated, in RFC3339 format.
     */
    updatedAt?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a RevisionAssets resource.
 */
export interface RevisionAssetsArgs {
    /**
     * A block to define the asset associated with the revision. See Asset for more details.
     *
     * The following arguments are optional:
     */
    assets?: pulumi.Input<pulumi.Input<inputs.dataexchange.RevisionAssetsAsset>[]>;
    /**
     * A comment for the revision. Maximum length is 16,348 characters.
     */
    comment?: pulumi.Input<string>;
    /**
     * Unique identifier for the data set associated with the revision.
     */
    dataSetId: pulumi.Input<string>;
    finalized?: pulumi.Input<boolean>;
    forceDestroy?: pulumi.Input<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    timeouts?: pulumi.Input<inputs.dataexchange.RevisionAssetsTimeouts>;
}
