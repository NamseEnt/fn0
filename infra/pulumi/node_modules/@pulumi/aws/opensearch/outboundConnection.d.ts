import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages an AWS Opensearch Outbound Connection.
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
 *     connectionMode: "DIRECT",
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
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import AWS Opensearch Outbound Connections using the Outbound Connection ID. For example:
 *
 * ```sh
 * $ pulumi import aws:opensearch/outboundConnection:OutboundConnection foo connection-id
 * ```
 */
export declare class OutboundConnection extends pulumi.CustomResource {
    /**
     * Get an existing OutboundConnection resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: OutboundConnectionState, opts?: pulumi.CustomResourceOptions): OutboundConnection;
    /**
     * Returns true if the given object is an instance of OutboundConnection.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is OutboundConnection;
    /**
     * Accepts the connection.
     */
    readonly acceptConnection: pulumi.Output<boolean | undefined>;
    /**
     * Specifies the connection alias that will be used by the customer for this connection.
     */
    readonly connectionAlias: pulumi.Output<string>;
    /**
     * Specifies the connection mode. Accepted values are `DIRECT` or `VPC_ENDPOINT`.
     */
    readonly connectionMode: pulumi.Output<string>;
    /**
     * Configuration block for the outbound connection.
     */
    readonly connectionProperties: pulumi.Output<outputs.opensearch.OutboundConnectionConnectionProperties>;
    /**
     * Status of the connection request.
     */
    readonly connectionStatus: pulumi.Output<string>;
    /**
     * Configuration block for the local Opensearch domain.
     */
    readonly localDomainInfo: pulumi.Output<outputs.opensearch.OutboundConnectionLocalDomainInfo>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Configuration block for the remote Opensearch domain.
     */
    readonly remoteDomainInfo: pulumi.Output<outputs.opensearch.OutboundConnectionRemoteDomainInfo>;
    /**
     * Create a OutboundConnection resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: OutboundConnectionArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering OutboundConnection resources.
 */
export interface OutboundConnectionState {
    /**
     * Accepts the connection.
     */
    acceptConnection?: pulumi.Input<boolean>;
    /**
     * Specifies the connection alias that will be used by the customer for this connection.
     */
    connectionAlias?: pulumi.Input<string>;
    /**
     * Specifies the connection mode. Accepted values are `DIRECT` or `VPC_ENDPOINT`.
     */
    connectionMode?: pulumi.Input<string>;
    /**
     * Configuration block for the outbound connection.
     */
    connectionProperties?: pulumi.Input<inputs.opensearch.OutboundConnectionConnectionProperties>;
    /**
     * Status of the connection request.
     */
    connectionStatus?: pulumi.Input<string>;
    /**
     * Configuration block for the local Opensearch domain.
     */
    localDomainInfo?: pulumi.Input<inputs.opensearch.OutboundConnectionLocalDomainInfo>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Configuration block for the remote Opensearch domain.
     */
    remoteDomainInfo?: pulumi.Input<inputs.opensearch.OutboundConnectionRemoteDomainInfo>;
}
/**
 * The set of arguments for constructing a OutboundConnection resource.
 */
export interface OutboundConnectionArgs {
    /**
     * Accepts the connection.
     */
    acceptConnection?: pulumi.Input<boolean>;
    /**
     * Specifies the connection alias that will be used by the customer for this connection.
     */
    connectionAlias: pulumi.Input<string>;
    /**
     * Specifies the connection mode. Accepted values are `DIRECT` or `VPC_ENDPOINT`.
     */
    connectionMode?: pulumi.Input<string>;
    /**
     * Configuration block for the outbound connection.
     */
    connectionProperties?: pulumi.Input<inputs.opensearch.OutboundConnectionConnectionProperties>;
    /**
     * Configuration block for the local Opensearch domain.
     */
    localDomainInfo: pulumi.Input<inputs.opensearch.OutboundConnectionLocalDomainInfo>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Configuration block for the remote Opensearch domain.
     */
    remoteDomainInfo: pulumi.Input<inputs.opensearch.OutboundConnectionRemoteDomainInfo>;
}
