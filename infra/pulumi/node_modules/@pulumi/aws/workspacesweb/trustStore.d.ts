import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS WorkSpaces Web Trust Store.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 * import * as std from "@pulumi/std";
 *
 * const example = new aws.workspacesweb.TrustStore("example", {certificates: [{
 *     body: std.file({
 *         input: "certificate.pem",
 *     }).then(invoke => invoke.result),
 * }]});
 * ```
 *
 * ### Multiple Certificates
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 * import * as std from "@pulumi/std";
 *
 * const example = new aws.workspacesweb.TrustStore("example", {
 *     certificates: [
 *         {
 *             body: std.file({
 *                 input: "certificate1.pem",
 *             }).then(invoke => invoke.result),
 *         },
 *         {
 *             body: std.file({
 *                 input: "certificate2.pem",
 *             }).then(invoke => invoke.result),
 *         },
 *     ],
 *     tags: {
 *         Name: "example-trust-store",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import WorkSpaces Web Trust Store using the `trust_store_arn`. For example:
 *
 * ```sh
 * $ pulumi import aws:workspacesweb/trustStore:TrustStore example arn:aws:workspaces-web:us-west-2:123456789012:trustStore/trust_store-id-12345678
 * ```
 */
export declare class TrustStore extends pulumi.CustomResource {
    /**
     * Get an existing TrustStore resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: TrustStoreState, opts?: pulumi.CustomResourceOptions): TrustStore;
    /**
     * Returns true if the given object is an instance of TrustStore.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is TrustStore;
    /**
     * List of ARNs of the web portals associated with the trust store.
     */
    readonly associatedPortalArns: pulumi.Output<string[]>;
    /**
     * Set of certificates to include in the trust store. See Certificate below.
     */
    readonly certificates: pulumi.Output<outputs.workspacesweb.TrustStoreCertificate[] | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * ARN of the trust store.
     */
    readonly trustStoreArn: pulumi.Output<string>;
    /**
     * Create a TrustStore resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: TrustStoreArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering TrustStore resources.
 */
export interface TrustStoreState {
    /**
     * List of ARNs of the web portals associated with the trust store.
     */
    associatedPortalArns?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Set of certificates to include in the trust store. See Certificate below.
     */
    certificates?: pulumi.Input<pulumi.Input<inputs.workspacesweb.TrustStoreCertificate>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
    /**
     * ARN of the trust store.
     */
    trustStoreArn?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a TrustStore resource.
 */
export interface TrustStoreArgs {
    /**
     * Set of certificates to include in the trust store. See Certificate below.
     */
    certificates?: pulumi.Input<pulumi.Input<inputs.workspacesweb.TrustStoreCertificate>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
