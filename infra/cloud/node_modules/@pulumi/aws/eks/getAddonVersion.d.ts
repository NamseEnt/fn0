import * as pulumi from "@pulumi/pulumi";
/**
 * Retrieve information about a specific EKS add-on version compatible with an EKS cluster version.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * export = async () => {
 *     const _default = await aws.eks.getAddonVersion({
 *         addonName: "vpc-cni",
 *         kubernetesVersion: example.version,
 *     });
 *     const latest = await aws.eks.getAddonVersion({
 *         addonName: "vpc-cni",
 *         kubernetesVersion: example.version,
 *         mostRecent: true,
 *     });
 *     const vpcCni = new aws.eks.Addon("vpc_cni", {
 *         clusterName: example.name,
 *         addonName: "vpc-cni",
 *         addonVersion: latest.version,
 *     });
 *     return {
 *         "default": _default.version,
 *         latest: latest.version,
 *     };
 * }
 * ```
 */
export declare function getAddonVersion(args: GetAddonVersionArgs, opts?: pulumi.InvokeOptions): Promise<GetAddonVersionResult>;
/**
 * A collection of arguments for invoking getAddonVersion.
 */
export interface GetAddonVersionArgs {
    /**
     * Name of the EKS add-on. The name must match one of
     * the names returned by [list-addon](https://docs.aws.amazon.com/cli/latest/reference/eks/list-addons.html).
     */
    addonName: string;
    /**
     * Version of the EKS Cluster. Must be between 1-100 characters in length. Must begin with an alphanumeric character, and must only contain alphanumeric characters, dashes and underscores (`^[0-9A-Za-z][A-Za-z0-9\-_]+$`).
     */
    kubernetesVersion: string;
    /**
     * Determines if the most recent or default version of the addon should be returned.
     */
    mostRecent?: boolean;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
}
/**
 * A collection of values returned by getAddonVersion.
 */
export interface GetAddonVersionResult {
    readonly addonName: string;
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly kubernetesVersion: string;
    readonly mostRecent?: boolean;
    readonly region: string;
    /**
     * Version of the EKS add-on.
     */
    readonly version: string;
}
/**
 * Retrieve information about a specific EKS add-on version compatible with an EKS cluster version.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * export = async () => {
 *     const _default = await aws.eks.getAddonVersion({
 *         addonName: "vpc-cni",
 *         kubernetesVersion: example.version,
 *     });
 *     const latest = await aws.eks.getAddonVersion({
 *         addonName: "vpc-cni",
 *         kubernetesVersion: example.version,
 *         mostRecent: true,
 *     });
 *     const vpcCni = new aws.eks.Addon("vpc_cni", {
 *         clusterName: example.name,
 *         addonName: "vpc-cni",
 *         addonVersion: latest.version,
 *     });
 *     return {
 *         "default": _default.version,
 *         latest: latest.version,
 *     };
 * }
 * ```
 */
export declare function getAddonVersionOutput(args: GetAddonVersionOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetAddonVersionResult>;
/**
 * A collection of arguments for invoking getAddonVersion.
 */
export interface GetAddonVersionOutputArgs {
    /**
     * Name of the EKS add-on. The name must match one of
     * the names returned by [list-addon](https://docs.aws.amazon.com/cli/latest/reference/eks/list-addons.html).
     */
    addonName: pulumi.Input<string>;
    /**
     * Version of the EKS Cluster. Must be between 1-100 characters in length. Must begin with an alphanumeric character, and must only contain alphanumeric characters, dashes and underscores (`^[0-9A-Za-z][A-Za-z0-9\-_]+$`).
     */
    kubernetesVersion: pulumi.Input<string>;
    /**
     * Determines if the most recent or default version of the addon should be returned.
     */
    mostRecent?: pulumi.Input<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
