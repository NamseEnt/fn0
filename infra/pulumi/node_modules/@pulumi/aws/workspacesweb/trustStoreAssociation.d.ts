import * as pulumi from "@pulumi/pulumi";
/**
 * Resource for managing an AWS WorkSpaces Web Trust Store Association.
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
 * const example = new aws.workspacesweb.Portal("example", {displayName: "example"});
 * const exampleTrustStore = new aws.workspacesweb.TrustStore("example", {certificateList: [std.file({
 *     input: "certificate.pem",
 * }).then(invoke => std.base64encode({
 *     input: invoke.result,
 * })).then(invoke => invoke.result)]});
 * const exampleTrustStoreAssociation = new aws.workspacesweb.TrustStoreAssociation("example", {
 *     trustStoreArn: exampleTrustStore.trustStoreArn,
 *     portalArn: example.portalArn,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import WorkSpaces Web Trust Store Association using the `trust_store_arn,portal_arn`. For example:
 *
 * ```sh
 * $ pulumi import aws:workspacesweb/trustStoreAssociation:TrustStoreAssociation example arn:aws:workspaces-web:us-west-2:123456789012:trustStore/trust_store-id-12345678,arn:aws:workspaces-web:us-west-2:123456789012:portal/portal-id-12345678
 * ```
 */
export declare class TrustStoreAssociation extends pulumi.CustomResource {
    /**
     * Get an existing TrustStoreAssociation resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: TrustStoreAssociationState, opts?: pulumi.CustomResourceOptions): TrustStoreAssociation;
    /**
     * Returns true if the given object is an instance of TrustStoreAssociation.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is TrustStoreAssociation;
    /**
     * ARN of the portal to associate with the trust store. Forces replacement if changed.
     *
     * The following arguments are optional:
     */
    readonly portalArn: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * ARN of the trust store to associate with the portal. Forces replacement if changed.
     */
    readonly trustStoreArn: pulumi.Output<string>;
    /**
     * Create a TrustStoreAssociation resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: TrustStoreAssociationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering TrustStoreAssociation resources.
 */
export interface TrustStoreAssociationState {
    /**
     * ARN of the portal to associate with the trust store. Forces replacement if changed.
     *
     * The following arguments are optional:
     */
    portalArn?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ARN of the trust store to associate with the portal. Forces replacement if changed.
     */
    trustStoreArn?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a TrustStoreAssociation resource.
 */
export interface TrustStoreAssociationArgs {
    /**
     * ARN of the portal to associate with the trust store. Forces replacement if changed.
     *
     * The following arguments are optional:
     */
    portalArn: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ARN of the trust store to associate with the portal. Forces replacement if changed.
     */
    trustStoreArn: pulumi.Input<string>;
}
