import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS AppSync Source API Association.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = new aws.appsync.SourceApiAssociation("test", {
 *     description: "My source API Merged",
 *     mergedApiId: "gzos6bteufdunffzzifiowisoe",
 *     sourceApiId: "fzzifiowisoegzos6bteufdunf",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import AppSync Source API Association using the `association_id` and `merged_api_id` separated by `,`. For example:
 *
 * ```sh
 * $ pulumi import aws:appsync/sourceApiAssociation:SourceApiAssociation example gzos6bteufdunffzzifiowisoe,243685a0-9347-4a1a-89c1-9b57dea01e31
 * ```
 */
export declare class SourceApiAssociation extends pulumi.CustomResource {
    /**
     * Get an existing SourceApiAssociation resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: SourceApiAssociationState, opts?: pulumi.CustomResourceOptions): SourceApiAssociation;
    /**
     * Returns true if the given object is an instance of SourceApiAssociation.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is SourceApiAssociation;
    /**
     * ARN of the Source API Association.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * ID of the Source API Association.
     */
    readonly associationId: pulumi.Output<string>;
    /**
     * Description of the source API being merged.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * ARN of the merged API. One of `mergedApiArn` or `mergedApiId` must be specified.
     */
    readonly mergedApiArn: pulumi.Output<string>;
    /**
     * ID of the merged API. One of `mergedApiArn` or `mergedApiId` must be specified.
     */
    readonly mergedApiId: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * ARN of the source API. One of `sourceApiArn` or `sourceApiId` must be specified.
     */
    readonly sourceApiArn: pulumi.Output<string>;
    readonly sourceApiAssociationConfigs: pulumi.Output<outputs.appsync.SourceApiAssociationSourceApiAssociationConfig[]>;
    /**
     * ID of the source API. One of `sourceApiArn` or `sourceApiId` must be specified.
     */
    readonly sourceApiId: pulumi.Output<string>;
    readonly timeouts: pulumi.Output<outputs.appsync.SourceApiAssociationTimeouts | undefined>;
    /**
     * Create a SourceApiAssociation resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: SourceApiAssociationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering SourceApiAssociation resources.
 */
export interface SourceApiAssociationState {
    /**
     * ARN of the Source API Association.
     */
    arn?: pulumi.Input<string>;
    /**
     * ID of the Source API Association.
     */
    associationId?: pulumi.Input<string>;
    /**
     * Description of the source API being merged.
     */
    description?: pulumi.Input<string>;
    /**
     * ARN of the merged API. One of `mergedApiArn` or `mergedApiId` must be specified.
     */
    mergedApiArn?: pulumi.Input<string>;
    /**
     * ID of the merged API. One of `mergedApiArn` or `mergedApiId` must be specified.
     */
    mergedApiId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ARN of the source API. One of `sourceApiArn` or `sourceApiId` must be specified.
     */
    sourceApiArn?: pulumi.Input<string>;
    sourceApiAssociationConfigs?: pulumi.Input<pulumi.Input<inputs.appsync.SourceApiAssociationSourceApiAssociationConfig>[]>;
    /**
     * ID of the source API. One of `sourceApiArn` or `sourceApiId` must be specified.
     */
    sourceApiId?: pulumi.Input<string>;
    timeouts?: pulumi.Input<inputs.appsync.SourceApiAssociationTimeouts>;
}
/**
 * The set of arguments for constructing a SourceApiAssociation resource.
 */
export interface SourceApiAssociationArgs {
    /**
     * Description of the source API being merged.
     */
    description?: pulumi.Input<string>;
    /**
     * ARN of the merged API. One of `mergedApiArn` or `mergedApiId` must be specified.
     */
    mergedApiArn?: pulumi.Input<string>;
    /**
     * ID of the merged API. One of `mergedApiArn` or `mergedApiId` must be specified.
     */
    mergedApiId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ARN of the source API. One of `sourceApiArn` or `sourceApiId` must be specified.
     */
    sourceApiArn?: pulumi.Input<string>;
    sourceApiAssociationConfigs?: pulumi.Input<pulumi.Input<inputs.appsync.SourceApiAssociationSourceApiAssociationConfig>[]>;
    /**
     * ID of the source API. One of `sourceApiArn` or `sourceApiId` must be specified.
     */
    sourceApiId?: pulumi.Input<string>;
    timeouts?: pulumi.Input<inputs.appsync.SourceApiAssociationTimeouts>;
}
