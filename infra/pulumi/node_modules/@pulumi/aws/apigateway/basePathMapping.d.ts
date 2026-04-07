import * as pulumi from "@pulumi/pulumi";
import { RestApi } from "./index";
/**
 * Connects a custom domain name registered via `aws.apigateway.DomainName`
 * with a deployed API so that its methods can be called via the
 * custom domain name.
 *
 * ## Import
 *
 * For a non-root `base_path`:
 *
 * For a non-root `base_path` and a private custom domain name:
 *
 * Using `pulumi import`, import `aws_api_gateway_base_path_mapping` using the domain name and base path or domain name, base path and domain name ID (for private custom domain names). For example:
 *
 * For an empty `base_path` or, in other words, a root path (`/`):
 *
 * ```sh
 * $ pulumi import aws:apigateway/basePathMapping:BasePathMapping example example.com/
 * ```
 * For a non-root `base_path`:
 *
 * ```sh
 * $ pulumi import aws:apigateway/basePathMapping:BasePathMapping example example.com/base-path
 * ```
 * For a non-root `base_path` and a private custom domain name:
 *
 * ```sh
 * $ pulumi import aws:apigateway/basePathMapping:BasePathMapping example api.internal.example.com/base-path/abcde12345
 * ```
 */
export declare class BasePathMapping extends pulumi.CustomResource {
    /**
     * Get an existing BasePathMapping resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: BasePathMappingState, opts?: pulumi.CustomResourceOptions): BasePathMapping;
    /**
     * Returns true if the given object is an instance of BasePathMapping.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is BasePathMapping;
    /**
     * Path segment that must be prepended to the path when accessing the API via this mapping. If omitted, the API is exposed at the root of the given domain.
     */
    readonly basePath: pulumi.Output<string | undefined>;
    /**
     * Already-registered domain name to connect the API to.
     */
    readonly domainName: pulumi.Output<string>;
    /**
     * The identifier for the domain name resource. Supported only for private custom domain names.
     */
    readonly domainNameId: pulumi.Output<string | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * ID of the API to connect.
     */
    readonly restApi: pulumi.Output<string>;
    /**
     * Name of a specific deployment stage to expose at the given path. If omitted, callers may select any stage by including its name as a path element after the base path.
     */
    readonly stageName: pulumi.Output<string | undefined>;
    /**
     * Create a BasePathMapping resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: BasePathMappingArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering BasePathMapping resources.
 */
export interface BasePathMappingState {
    /**
     * Path segment that must be prepended to the path when accessing the API via this mapping. If omitted, the API is exposed at the root of the given domain.
     */
    basePath?: pulumi.Input<string>;
    /**
     * Already-registered domain name to connect the API to.
     */
    domainName?: pulumi.Input<string>;
    /**
     * The identifier for the domain name resource. Supported only for private custom domain names.
     */
    domainNameId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ID of the API to connect.
     */
    restApi?: pulumi.Input<string | RestApi>;
    /**
     * Name of a specific deployment stage to expose at the given path. If omitted, callers may select any stage by including its name as a path element after the base path.
     */
    stageName?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a BasePathMapping resource.
 */
export interface BasePathMappingArgs {
    /**
     * Path segment that must be prepended to the path when accessing the API via this mapping. If omitted, the API is exposed at the root of the given domain.
     */
    basePath?: pulumi.Input<string>;
    /**
     * Already-registered domain name to connect the API to.
     */
    domainName: pulumi.Input<string>;
    /**
     * The identifier for the domain name resource. Supported only for private custom domain names.
     */
    domainNameId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ID of the API to connect.
     */
    restApi: pulumi.Input<string | RestApi>;
    /**
     * Name of a specific deployment stage to expose at the given path. If omitted, callers may select any stage by including its name as a path element after the base path.
     */
    stageName?: pulumi.Input<string>;
}
