import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a SageMaker AI Hub resource.
 *
 * ## Example Usage
 *
 * ### Basic usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.sagemaker.Hub("example", {
 *     hubName: "example",
 *     hubDescription: "example",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import SageMaker AI Hubs using the `name`. For example:
 *
 * ```sh
 * $ pulumi import aws:sagemaker/hub:Hub test_hub my-code-repo
 * ```
 */
export declare class Hub extends pulumi.CustomResource {
    /**
     * Get an existing Hub resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: HubState, opts?: pulumi.CustomResourceOptions): Hub;
    /**
     * Returns true if the given object is an instance of Hub.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Hub;
    /**
     * The Amazon Resource Name (ARN) assigned by AWS to this Hub.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * A description of the hub.
     */
    readonly hubDescription: pulumi.Output<string>;
    /**
     * The display name of the hub.
     */
    readonly hubDisplayName: pulumi.Output<string | undefined>;
    /**
     * The name of the hub.
     */
    readonly hubName: pulumi.Output<string>;
    /**
     * The searchable keywords for the hub.
     */
    readonly hubSearchKeywords: pulumi.Output<string[] | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The Amazon S3 storage configuration for the hub. See S3 Storage Config details below.
     */
    readonly s3StorageConfig: pulumi.Output<outputs.sagemaker.HubS3StorageConfig | undefined>;
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
    /**
     * Create a Hub resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: HubArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Hub resources.
 */
export interface HubState {
    /**
     * The Amazon Resource Name (ARN) assigned by AWS to this Hub.
     */
    arn?: pulumi.Input<string>;
    /**
     * A description of the hub.
     */
    hubDescription?: pulumi.Input<string>;
    /**
     * The display name of the hub.
     */
    hubDisplayName?: pulumi.Input<string>;
    /**
     * The name of the hub.
     */
    hubName?: pulumi.Input<string>;
    /**
     * The searchable keywords for the hub.
     */
    hubSearchKeywords?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The Amazon S3 storage configuration for the hub. See S3 Storage Config details below.
     */
    s3StorageConfig?: pulumi.Input<inputs.sagemaker.HubS3StorageConfig>;
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
}
/**
 * The set of arguments for constructing a Hub resource.
 */
export interface HubArgs {
    /**
     * A description of the hub.
     */
    hubDescription: pulumi.Input<string>;
    /**
     * The display name of the hub.
     */
    hubDisplayName?: pulumi.Input<string>;
    /**
     * The name of the hub.
     */
    hubName: pulumi.Input<string>;
    /**
     * The searchable keywords for the hub.
     */
    hubSearchKeywords?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The Amazon S3 storage configuration for the hub. See S3 Storage Config details below.
     */
    s3StorageConfig?: pulumi.Input<inputs.sagemaker.HubS3StorageConfig>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
