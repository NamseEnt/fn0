import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing a Verified Access Instance.
 *
 * ## Example Usage
 *
 * ### Basic
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.verifiedaccess.Instance("example", {
 *     description: "example",
 *     tags: {
 *         Name: "example",
 *     },
 * });
 * ```
 *
 * ### With `fipsEnabled`
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.verifiedaccess.Instance("example", {fipsEnabled: true});
 * ```
 *
 * ### With `cidrEndpointsCustomSubdomain`
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.verifiedaccess.Instance("example", {cidrEndpointsCustomSubdomain: "test.example.com"});
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Verified Access Instances using the  `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:verifiedaccess/instance:Instance example vai-1234567890abcdef0
 * ```
 */
export declare class Instance extends pulumi.CustomResource {
    /**
     * Get an existing Instance resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: InstanceState, opts?: pulumi.CustomResourceOptions): Instance;
    /**
     * Returns true if the given object is an instance of Instance.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Instance;
    /**
     * The custom subdomain for the CIDR endpoints.
     */
    readonly cidrEndpointsCustomSubdomain: pulumi.Output<string | undefined>;
    /**
     * The time that the Verified Access Instance was created.
     */
    readonly creationTime: pulumi.Output<string>;
    /**
     * A description for the AWS Verified Access Instance.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * Enable or disable support for Federal Information Processing Standards (FIPS) on the AWS Verified Access Instance.
     */
    readonly fipsEnabled: pulumi.Output<boolean | undefined>;
    /**
     * The time that the Verified Access Instance was last updated.
     */
    readonly lastUpdatedTime: pulumi.Output<string>;
    readonly nameServers: pulumi.Output<string[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * One or more blocks of providing information about the AWS Verified Access Trust Providers. See verifiedAccessTrustProviders below for details.One or more blocks
     */
    readonly verifiedAccessTrustProviders: pulumi.Output<outputs.verifiedaccess.InstanceVerifiedAccessTrustProvider[]>;
    /**
     * Create a Instance resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: InstanceArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Instance resources.
 */
export interface InstanceState {
    /**
     * The custom subdomain for the CIDR endpoints.
     */
    cidrEndpointsCustomSubdomain?: pulumi.Input<string>;
    /**
     * The time that the Verified Access Instance was created.
     */
    creationTime?: pulumi.Input<string>;
    /**
     * A description for the AWS Verified Access Instance.
     */
    description?: pulumi.Input<string>;
    /**
     * Enable or disable support for Federal Information Processing Standards (FIPS) on the AWS Verified Access Instance.
     */
    fipsEnabled?: pulumi.Input<boolean>;
    /**
     * The time that the Verified Access Instance was last updated.
     */
    lastUpdatedTime?: pulumi.Input<string>;
    nameServers?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * One or more blocks of providing information about the AWS Verified Access Trust Providers. See verifiedAccessTrustProviders below for details.One or more blocks
     */
    verifiedAccessTrustProviders?: pulumi.Input<pulumi.Input<inputs.verifiedaccess.InstanceVerifiedAccessTrustProvider>[]>;
}
/**
 * The set of arguments for constructing a Instance resource.
 */
export interface InstanceArgs {
    /**
     * The custom subdomain for the CIDR endpoints.
     */
    cidrEndpointsCustomSubdomain?: pulumi.Input<string>;
    /**
     * A description for the AWS Verified Access Instance.
     */
    description?: pulumi.Input<string>;
    /**
     * Enable or disable support for Federal Information Processing Standards (FIPS) on the AWS Verified Access Instance.
     */
    fipsEnabled?: pulumi.Input<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
