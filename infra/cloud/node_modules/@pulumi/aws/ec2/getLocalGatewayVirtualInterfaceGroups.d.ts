import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides details about multiple EC2 Local Gateway Virtual Interface Groups, such as identifiers. More information can be found in the [Outposts User Guide](https://docs.aws.amazon.com/outposts/latest/userguide/outposts-networking-components.html#routing).
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const all = aws.ec2.getLocalGatewayVirtualInterfaceGroups({});
 * ```
 */
export declare function getLocalGatewayVirtualInterfaceGroups(args?: GetLocalGatewayVirtualInterfaceGroupsArgs, opts?: pulumi.InvokeOptions): Promise<GetLocalGatewayVirtualInterfaceGroupsResult>;
/**
 * A collection of arguments for invoking getLocalGatewayVirtualInterfaceGroups.
 */
export interface GetLocalGatewayVirtualInterfaceGroupsArgs {
    /**
     * One or more configuration blocks containing name-values filters. See the [EC2 API Reference](https://docs.aws.amazon.com/AWSEC2/latest/APIReference/API_DescribeLocalGatewayVirtualInterfaceGroups.html) for supported filters. Detailed below.
     */
    filters?: inputs.ec2.GetLocalGatewayVirtualInterfaceGroupsFilter[];
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
 * A collection of values returned by getLocalGatewayVirtualInterfaceGroups.
 */
export interface GetLocalGatewayVirtualInterfaceGroupsResult {
    readonly filters?: outputs.ec2.GetLocalGatewayVirtualInterfaceGroupsFilter[];
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    /**
     * Set of EC2 Local Gateway Virtual Interface Group identifiers.
     */
    readonly ids: string[];
    /**
     * Set of EC2 Local Gateway Virtual Interface identifiers.
     */
    readonly localGatewayVirtualInterfaceIds: string[];
    readonly region: string;
    readonly tags?: {
        [key: string]: string;
    };
}
/**
 * Provides details about multiple EC2 Local Gateway Virtual Interface Groups, such as identifiers. More information can be found in the [Outposts User Guide](https://docs.aws.amazon.com/outposts/latest/userguide/outposts-networking-components.html#routing).
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const all = aws.ec2.getLocalGatewayVirtualInterfaceGroups({});
 * ```
 */
export declare function getLocalGatewayVirtualInterfaceGroupsOutput(args?: GetLocalGatewayVirtualInterfaceGroupsOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetLocalGatewayVirtualInterfaceGroupsResult>;
/**
 * A collection of arguments for invoking getLocalGatewayVirtualInterfaceGroups.
 */
export interface GetLocalGatewayVirtualInterfaceGroupsOutputArgs {
    /**
     * One or more configuration blocks containing name-values filters. See the [EC2 API Reference](https://docs.aws.amazon.com/AWSEC2/latest/APIReference/API_DescribeLocalGatewayVirtualInterfaceGroups.html) for supported filters. Detailed below.
     */
    filters?: pulumi.Input<pulumi.Input<inputs.ec2.GetLocalGatewayVirtualInterfaceGroupsFilterArgs>[]>;
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
