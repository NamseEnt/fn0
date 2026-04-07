import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a Public Elastic Container Registry Repository.
 *
 * > **NOTE:** This resource can only be used in the `us-east-1` region.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 * import * as std from "@pulumi/std";
 *
 * const foo = new aws.ecrpublic.Repository("foo", {
 *     repositoryName: "bar",
 *     catalogData: {
 *         aboutText: "About Text",
 *         architectures: ["ARM"],
 *         description: "Description",
 *         logoImageBlob: std.filebase64({
 *             input: png,
 *         }).then(invoke => invoke.result),
 *         operatingSystems: ["Linux"],
 *         usageText: "Usage Text",
 *     },
 *     tags: {
 *         env: "production",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import ECR Public Repositories using the `repository_name`. For example:
 *
 * ```sh
 * $ pulumi import aws:ecrpublic/repository:Repository example example
 * ```
 */
export declare class Repository extends pulumi.CustomResource {
    /**
     * Get an existing Repository resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: RepositoryState, opts?: pulumi.CustomResourceOptions): Repository;
    /**
     * Returns true if the given object is an instance of Repository.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Repository;
    /**
     * Full ARN of the repository.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Catalog data configuration for the repository. See below for schema.
     */
    readonly catalogData: pulumi.Output<outputs.ecrpublic.RepositoryCatalogData | undefined>;
    readonly forceDestroy: pulumi.Output<boolean | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The registry ID where the repository was created.
     */
    readonly registryId: pulumi.Output<string>;
    /**
     * Name of the repository.
     */
    readonly repositoryName: pulumi.Output<string>;
    /**
     * The URI of the repository.
     */
    readonly repositoryUri: pulumi.Output<string>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Create a Repository resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: RepositoryArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Repository resources.
 */
export interface RepositoryState {
    /**
     * Full ARN of the repository.
     */
    arn?: pulumi.Input<string>;
    /**
     * Catalog data configuration for the repository. See below for schema.
     */
    catalogData?: pulumi.Input<inputs.ecrpublic.RepositoryCatalogData>;
    forceDestroy?: pulumi.Input<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The registry ID where the repository was created.
     */
    registryId?: pulumi.Input<string>;
    /**
     * Name of the repository.
     */
    repositoryName?: pulumi.Input<string>;
    /**
     * The URI of the repository.
     */
    repositoryUri?: pulumi.Input<string>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
 * The set of arguments for constructing a Repository resource.
 */
export interface RepositoryArgs {
    /**
     * Catalog data configuration for the repository. See below for schema.
     */
    catalogData?: pulumi.Input<inputs.ecrpublic.RepositoryCatalogData>;
    forceDestroy?: pulumi.Input<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Name of the repository.
     */
    repositoryName: pulumi.Input<string>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
