import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.odb.NetworkPeeringConnection("example", {
 *     displayName: "example",
 *     odbNetworkId: "my-odb-network-id",
 *     peerNetworkId: "my-vpc-id",
 *     tags: {
 *         env: "dev",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import odb network peering using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:odb/networkPeeringConnection:NetworkPeeringConnection example example
 * ```
 */
export declare class NetworkPeeringConnection extends pulumi.CustomResource {
    /**
     * Get an existing NetworkPeeringConnection resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: NetworkPeeringConnectionState, opts?: pulumi.CustomResourceOptions): NetworkPeeringConnection;
    /**
     * Returns true if the given object is an instance of NetworkPeeringConnection.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is NetworkPeeringConnection;
    readonly arn: pulumi.Output<string>;
    /**
     * Created time of the ODB network peering connection.
     */
    readonly createdAt: pulumi.Output<string>;
    readonly displayName: pulumi.Output<string>;
    /**
     * ARN of the ODB network peering connection.
     */
    readonly odbNetworkArn: pulumi.Output<string>;
    readonly odbNetworkId: pulumi.Output<string>;
    /**
     * Type of the ODB peering connection.
     */
    readonly odbPeeringConnectionType: pulumi.Output<string>;
    /**
     * ARN of the peer network peering connection.
     */
    readonly peerNetworkArn: pulumi.Output<string>;
    readonly peerNetworkId: pulumi.Output<string>;
    /**
     * Progress of the ODB network peering connection.
     */
    readonly percentProgress: pulumi.Output<number>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Status of the ODB network peering connection.
     */
    readonly status: pulumi.Output<string>;
    /**
     * The reason for the current status of the ODB peering connection.
     */
    readonly statusReason: pulumi.Output<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * A map of tags assigned to the resource, including inherited tags.
     */
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    readonly timeouts: pulumi.Output<outputs.odb.NetworkPeeringConnectionTimeouts | undefined>;
    /**
     * Create a NetworkPeeringConnection resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: NetworkPeeringConnectionArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering NetworkPeeringConnection resources.
 */
export interface NetworkPeeringConnectionState {
    arn?: pulumi.Input<string>;
    /**
     * Created time of the ODB network peering connection.
     */
    createdAt?: pulumi.Input<string>;
    displayName?: pulumi.Input<string>;
    /**
     * ARN of the ODB network peering connection.
     */
    odbNetworkArn?: pulumi.Input<string>;
    odbNetworkId?: pulumi.Input<string>;
    /**
     * Type of the ODB peering connection.
     */
    odbPeeringConnectionType?: pulumi.Input<string>;
    /**
     * ARN of the peer network peering connection.
     */
    peerNetworkArn?: pulumi.Input<string>;
    peerNetworkId?: pulumi.Input<string>;
    /**
     * Progress of the ODB network peering connection.
     */
    percentProgress?: pulumi.Input<number>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Status of the ODB network peering connection.
     */
    status?: pulumi.Input<string>;
    /**
     * The reason for the current status of the ODB peering connection.
     */
    statusReason?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * A map of tags assigned to the resource, including inherited tags.
     */
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    timeouts?: pulumi.Input<inputs.odb.NetworkPeeringConnectionTimeouts>;
}
/**
 * The set of arguments for constructing a NetworkPeeringConnection resource.
 */
export interface NetworkPeeringConnectionArgs {
    displayName: pulumi.Input<string>;
    odbNetworkId: pulumi.Input<string>;
    peerNetworkId: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    timeouts?: pulumi.Input<inputs.odb.NetworkPeeringConnectionTimeouts>;
}
