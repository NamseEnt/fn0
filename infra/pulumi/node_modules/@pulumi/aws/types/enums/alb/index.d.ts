export declare const IpAddressType: {
    /**
     * IPv4 addresses
     */
    readonly Ipv4: "ipv4";
    /**
     * IPv4 and IPv6 addresses
     */
    readonly Dualstack: "dualstack";
    /**
     * Public IPv6 addresses and private IPv4 and IPv6 addresses
     */
    readonly DualstackWithoutPublicIpv4: "dualstack-without-public-ipv4";
};
export type IpAddressType = (typeof IpAddressType)[keyof typeof IpAddressType];
export declare const LoadBalancerType: {
    readonly Application: "application";
    readonly Network: "network";
};
export type LoadBalancerType = (typeof LoadBalancerType)[keyof typeof LoadBalancerType];
