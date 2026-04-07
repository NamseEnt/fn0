import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS DevOps Guru Resource Collection.
 *
 * > Only one type of resource collection (All Account Resources, CloudFormation, or Tags) can be enabled in an account at a time. To avoid persistent differences, this resource should be defined only once.
 *
 * ## Example Usage
 *
 * ### All Account Resources
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.devopsguru.ResourceCollection("example", {
 *     type: "AWS_SERVICE",
 *     cloudformation: {
 *         stackNames: ["*"],
 *     },
 * });
 * ```
 *
 * ### CloudFormation Stacks
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.devopsguru.ResourceCollection("example", {
 *     type: "AWS_CLOUD_FORMATION",
 *     cloudformation: {
 *         stackNames: ["ExampleStack"],
 *     },
 * });
 * ```
 *
 * ### Tags
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.devopsguru.ResourceCollection("example", {
 *     type: "AWS_TAGS",
 *     tags: {
 *         appBoundaryKey: "DevOps-Guru-Example",
 *         tagValues: ["Example-Value"],
 *     },
 * });
 * ```
 *
 * ### Tags All Resources
 *
 * To analyze all resources with the `appBoundaryKey` regardless of the corresponding tag value, set `tagValues` to `["*"]`.
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.devopsguru.ResourceCollection("example", {
 *     type: "AWS_TAGS",
 *     tags: {
 *         appBoundaryKey: "DevOps-Guru-Example",
 *         tagValues: ["*"],
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import DevOps Guru Resource Collection using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:devopsguru/resourceCollection:ResourceCollection example AWS_CLOUD_FORMATION
 * ```
 */
export declare class ResourceCollection extends pulumi.CustomResource {
    /**
     * Get an existing ResourceCollection resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ResourceCollectionState, opts?: pulumi.CustomResourceOptions): ResourceCollection;
    /**
     * Returns true if the given object is an instance of ResourceCollection.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ResourceCollection;
    /**
     * A collection of AWS CloudFormation stacks. See `cloudformation` below for additional details.
     */
    readonly cloudformation: pulumi.Output<outputs.devopsguru.ResourceCollectionCloudformation | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * AWS tags used to filter the resources in the resource collection. See `tags` below for additional details.
     */
    readonly tags: pulumi.Output<outputs.devopsguru.ResourceCollectionTags | undefined>;
    /**
     * Type of AWS resource collection to create. Valid values are `AWS_CLOUD_FORMATION`, `AWS_SERVICE`, and `AWS_TAGS`.
     *
     * The following arguments are optional:
     */
    readonly type: pulumi.Output<string>;
    /**
     * Create a ResourceCollection resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ResourceCollectionArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ResourceCollection resources.
 */
export interface ResourceCollectionState {
    /**
     * A collection of AWS CloudFormation stacks. See `cloudformation` below for additional details.
     */
    cloudformation?: pulumi.Input<inputs.devopsguru.ResourceCollectionCloudformation>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * AWS tags used to filter the resources in the resource collection. See `tags` below for additional details.
     */
    tags?: pulumi.Input<inputs.devopsguru.ResourceCollectionTags>;
    /**
     * Type of AWS resource collection to create. Valid values are `AWS_CLOUD_FORMATION`, `AWS_SERVICE`, and `AWS_TAGS`.
     *
     * The following arguments are optional:
     */
    type?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a ResourceCollection resource.
 */
export interface ResourceCollectionArgs {
    /**
     * A collection of AWS CloudFormation stacks. See `cloudformation` below for additional details.
     */
    cloudformation?: pulumi.Input<inputs.devopsguru.ResourceCollectionCloudformation>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * AWS tags used to filter the resources in the resource collection. See `tags` below for additional details.
     */
    tags?: pulumi.Input<inputs.devopsguru.ResourceCollectionTags>;
    /**
     * Type of AWS resource collection to create. Valid values are `AWS_CLOUD_FORMATION`, `AWS_SERVICE`, and `AWS_TAGS`.
     *
     * The following arguments are optional:
     */
    type: pulumi.Input<string>;
}
