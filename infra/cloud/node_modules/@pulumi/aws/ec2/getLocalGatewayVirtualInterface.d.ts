import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides details about an EC2 Local Gateway Virtual Interface. More information can be found in the [Outposts User Guide](https://docs.aws.amazon.com/outposts/latest/userguide/outposts-networking-components.html#routing).
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = .reduce((__obj, [__key, __value]) => ({ ...__obj, [__key]: aws.ec2.getLocalGatewayVirtualInterface({
 *     id: __value,
 * }) }));
 * ```
 */
export declare function getLocalGatewayVirtualInterface(args?: GetLocalGatewayVirtualInterfaceArgs, opts?: pulumi.InvokeOptions): Promise<GetLocalGatewayVirtualInterfaceResult>;
/**
 * A collection of arguments for invoking getLocalGatewayVirtualInterface.
 */
export interface GetLocalGatewayVirtualInterfaceArgs {
    /**
     * One or more configuration blocks containing name-values filters. See the [EC2 API Reference](https://docs.aws.amazon.com/AWSEC2/latest/APIReference/API_DescribeLocalGatewayVirtualInterfaces.html) for supported filters. Detailed below.
     */
    filters?: inputs.ec2.GetLocalGatewayVirtualInterfaceFilter[];
    /**
     * Identifier of EC2 Local Gateway Virtual Interface.
     */
    id?: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
    /**
     * Key-value map of resource tags, each pair of which must exactly match a pair on the desired local gateway route table.
     */
    tags?: {
        [key: string]: string;
    };
}
/**
 * A collection of values returned by getLocalGatewayVirtualInterface.
 */
export interface GetLocalGatewayVirtualInterfaceResult {
    readonly filters?: outputs.ec2.GetLocalGatewayVirtualInterfaceFilter[];
    readonly id: string;
    /**
     * Local address.
     */
    readonly localAddress: string;
    /**
     * Border Gateway Protocol (BGP) Autonomous System Number (ASN) of the EC2 Local Gateway.
     */
    readonly localBgpAsn: number;
    /**
     * Identifier of the EC2 Local Gateway.
     */
    readonly localGatewayId: string;
    readonly localGatewayVirtualInterfaceIds: string[];
    /**
     * Peer address.
     */
    readonly peerAddress: string;
    /**
     * Border Gateway Protocol (BGP) Autonomous System Number (ASN) of the peer.
     */
    readonly peerBgpAsn: number;
    readonly region: string;
    readonly tags: {
        [key: string]: string;
    };
    /**
     * Virtual Local Area Network.
     */
    readonly vlan: number;
}
/**
 * Provides details about an EC2 Local Gateway Virtual Interface. More information can be found in the [Outposts User Guide](https://docs.aws.amazon.com/outposts/latest/userguide/outposts-networking-components.html#routing).
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = .reduce((__obj, [__key, __value]) => ({ ...__obj, [__key]: aws.ec2.getLocalGatewayVirtualInterface({
 *     id: __value,
 * }) }));
 * ```
 */
export declare function getLocalGatewayVirtualInterfaceOutput(args?: GetLocalGatewayVirtualInterfaceOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetLocalGatewayVirtualInterfaceResult>;
/**
 * A collection of arguments for invoking getLocalGatewayVirtualInterface.
 */
export interface GetLocalGatewayVirtualInterfaceOutputArgs {
    /**
     * One or more configuration blocks containing name-values filters. See the [EC2 API Reference](https://docs.aws.amazon.com/AWSEC2/latest/APIReference/API_DescribeLocalGatewayVirtualInterfaces.html) for supported filters. Detailed below.
     */
    filters?: pulumi.Input<pulumi.Input<inputs.ec2.GetLocalGatewayVirtualInterfaceFilterArgs>[]>;
    /**
     * Identifier of EC2 Local Gateway Virtual Interface.
     */
    id?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Key-value map of resource tags, each pair of which must exactly match a pair on the desired local gateway route table.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
