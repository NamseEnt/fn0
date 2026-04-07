import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * `aws.ec2.getVpcIpamPools` provides details about IPAM pools.
 *
 * This resource can prove useful when IPAM pools are created in another root
 * module and you need the pool ids as input variables. For example, pools
 * can be shared via RAM and used to create vpcs with CIDRs from that pool.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = aws.ec2.getVpcIpamPools({
 *     filters: [
 *         {
 *             name: "description",
 *             values: ["*test*"],
 *         },
 *         {
 *             name: "address-family",
 *             values: ["ipv4"],
 *         },
 *     ],
 * });
 * ```
 */
export declare function getVpcIpamPools(args?: GetVpcIpamPoolsArgs, opts?: pulumi.InvokeOptions): Promise<GetVpcIpamPoolsResult>;
/**
 * A collection of arguments for invoking getVpcIpamPools.
 */
export interface GetVpcIpamPoolsArgs {
    /**
     * Custom filter block as described below.
     *
     * The arguments of this data source act as filters for querying the available IPAM Pools in the current region.
     */
    filters?: inputs.ec2.GetVpcIpamPoolsFilter[];
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
}
/**
 * A collection of values returned by getVpcIpamPools.
 */
export interface GetVpcIpamPoolsResult {
    readonly filters?: outputs.ec2.GetVpcIpamPoolsFilter[];
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    /**
     * List of IPAM pools and their attributes. See below for details
     */
    readonly ipamPools: outputs.ec2.GetVpcIpamPoolsIpamPool[];
    readonly region: string;
}
/**
 * `aws.ec2.getVpcIpamPools` provides details about IPAM pools.
 *
 * This resource can prove useful when IPAM pools are created in another root
 * module and you need the pool ids as input variables. For example, pools
 * can be shared via RAM and used to create vpcs with CIDRs from that pool.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = aws.ec2.getVpcIpamPools({
 *     filters: [
 *         {
 *             name: "description",
 *             values: ["*test*"],
 *         },
 *         {
 *             name: "address-family",
 *             values: ["ipv4"],
 *         },
 *     ],
 * });
 * ```
 */
export declare function getVpcIpamPoolsOutput(args?: GetVpcIpamPoolsOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetVpcIpamPoolsResult>;
/**
 * A collection of arguments for invoking getVpcIpamPools.
 */
export interface GetVpcIpamPoolsOutputArgs {
    /**
     * Custom filter block as described below.
     *
     * The arguments of this data source act as filters for querying the available IPAM Pools in the current region.
     */
    filters?: pulumi.Input<pulumi.Input<inputs.ec2.GetVpcIpamPoolsFilterArgs>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
