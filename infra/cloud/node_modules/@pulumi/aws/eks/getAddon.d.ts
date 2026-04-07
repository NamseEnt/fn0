import * as pulumi from "@pulumi/pulumi";
import * as outputs from "../types/output";
/**
 * Retrieve information about an EKS add-on.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.eks.getAddon({
 *     addonName: "vpc-cni",
 *     clusterName: exampleAwsEksCluster.name,
 * });
 * export const eksAddonOutputs = exampleAwsEksAddon;
 * ```
 */
export declare function getAddon(args: GetAddonArgs, opts?: pulumi.InvokeOptions): Promise<GetAddonResult>;
/**
 * A collection of arguments for invoking getAddon.
 */
export interface GetAddonArgs {
    /**
     * Name of the EKS add-on. The name must match one of
     * the names returned by [list-addon](https://docs.aws.amazon.com/cli/latest/reference/eks/list-addons.html).
     */
    addonName: string;
    /**
     * Name of the EKS Cluster.
     */
    clusterName: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
    tags?: {
        [key: string]: string;
    };
}
/**
 * A collection of values returned by getAddon.
 */
export interface GetAddonResult {
    readonly addonName: string;
    /**
     * Version of EKS add-on.
     */
    readonly addonVersion: string;
    /**
     * ARN of the EKS add-on.
     */
    readonly arn: string;
    readonly clusterName: string;
    /**
     * Configuration values for the addon with a single JSON string.
     */
    readonly configurationValues: string;
    /**
     * Date and time in [RFC3339 format](https://tools.ietf.org/html/rfc3339#section-5.8) that the EKS add-on was created.
     */
    readonly createdAt: string;
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    /**
     * Date and time in [RFC3339 format](https://tools.ietf.org/html/rfc3339#section-5.8) that the EKS add-on was updated.
     */
    readonly modifiedAt: string;
    /**
     * Pod identity association for the EKS add-on.
     */
    readonly podIdentityAssociations: outputs.eks.GetAddonPodIdentityAssociation[];
    readonly region: string;
    /**
     * ARN of IAM role used for EKS add-on. If value is empty -
     * then add-on uses the IAM role assigned to the EKS Cluster node.
     */
    readonly serviceAccountRoleArn: string;
    readonly tags: {
        [key: string]: string;
    };
}
/**
 * Retrieve information about an EKS add-on.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.eks.getAddon({
 *     addonName: "vpc-cni",
 *     clusterName: exampleAwsEksCluster.name,
 * });
 * export const eksAddonOutputs = exampleAwsEksAddon;
 * ```
 */
export declare function getAddonOutput(args: GetAddonOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetAddonResult>;
/**
 * A collection of arguments for invoking getAddon.
 */
export interface GetAddonOutputArgs {
    /**
     * Name of the EKS add-on. The name must match one of
     * the names returned by [list-addon](https://docs.aws.amazon.com/cli/latest/reference/eks/list-addons.html).
     */
    addonName: pulumi.Input<string>;
    /**
     * Name of the EKS Cluster.
     */
    clusterName: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
