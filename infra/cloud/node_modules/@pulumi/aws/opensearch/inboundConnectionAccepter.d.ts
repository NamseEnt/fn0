import * as pulumi from "@pulumi/pulumi";
/**
 * Manages an [AWS Opensearch Inbound Connection Accepter](https://docs.aws.amazon.com/opensearch-service/latest/APIReference/API_AcceptInboundConnection.html). If connecting domains from different AWS accounts, ensure that the accepter is configured to use the AWS account where the _remote_ opensearch domain exists.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const current = aws.getCallerIdentity({});
 * const currentGetRegion = aws.getRegion({});
 * const foo = new aws.opensearch.OutboundConnection("foo", {
 *     connectionAlias: "outbound_connection",
 *     localDomainInfo: {
 *         ownerId: current.then(current => current.accountId),
 *         region: currentGetRegion.then(currentGetRegion => currentGetRegion.region),
 *         domainName: localDomain.domainName,
 *     },
 *     remoteDomainInfo: {
 *         ownerId: current.then(current => current.accountId),
 *         region: currentGetRegion.then(currentGetRegion => currentGetRegion.region),
 *         domainName: remoteDomain.domainName,
 *     },
 * });
 * const fooInboundConnectionAccepter = new aws.opensearch.InboundConnectionAccepter("foo", {connectionId: foo.id});
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import AWS Opensearch Inbound Connection Accepters using the Inbound Connection ID. For example:
 *
 * ```sh
 * $ pulumi import aws:opensearch/inboundConnectionAccepter:InboundConnectionAccepter foo connection-id
 * ```
 */
export declare class InboundConnectionAccepter extends pulumi.CustomResource {
    /**
     * Get an existing InboundConnectionAccepter resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: InboundConnectionAccepterState, opts?: pulumi.CustomResourceOptions): InboundConnectionAccepter;
    /**
     * Returns true if the given object is an instance of InboundConnectionAccepter.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is InboundConnectionAccepter;
    /**
     * Specifies the ID of the connection to accept.
     */
    readonly connectionId: pulumi.Output<string>;
    /**
     * Status of the connection request.
     */
    readonly connectionStatus: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Create a InboundConnectionAccepter resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: InboundConnectionAccepterArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering InboundConnectionAccepter resources.
 */
export interface InboundConnectionAccepterState {
    /**
     * Specifies the ID of the connection to accept.
     */
    connectionId?: pulumi.Input<string>;
    /**
     * Status of the connection request.
     */
    connectionStatus?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a InboundConnectionAccepter resource.
 */
export interface InboundConnectionAccepterArgs {
    /**
     * Specifies the ID of the connection to accept.
     */
    connectionId: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
