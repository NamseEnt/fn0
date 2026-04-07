export declare const IpAddressType: {
    readonly Ipv4: "ipv4";
    readonly Dualstack: "dualstack";
};
export type IpAddressType = (typeof IpAddressType)[keyof typeof IpAddressType];
export declare const LoadBalancerType: {
    readonly Application: "application";
    readonly Network: "network";
};
export type LoadBalancerType = (typeof LoadBalancerType)[keyof typeof LoadBalancerType];
