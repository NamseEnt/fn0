import * as pulumi from "@pulumi/pulumi";
/**
 * Get an authentication token to communicate with an EKS cluster.
 *
 * Uses IAM credentials from the AWS provider to generate a temporary token that is compatible with
 * [AWS IAM Authenticator](https://github.com/kubernetes-sigs/aws-iam-authenticator) authentication.
 * This can be used to authenticate to an EKS cluster or to a cluster that has the AWS IAM Authenticator
 * server configured.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.eks.getCluster({
 *     name: "example",
 * });
 * const exampleGetClusterAuth = aws.eks.getClusterAuth({
 *     name: "example",
 * });
 * ```
 */
export declare function getClusterAuth(args: GetClusterAuthArgs, opts?: pulumi.InvokeOptions): Promise<GetClusterAuthResult>;
/**
 * A collection of arguments for invoking getClusterAuth.
 */
export interface GetClusterAuthArgs {
    /**
     * Name of the cluster
     */
    name: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
}
/**
 * A collection of values returned by getClusterAuth.
 */
export interface GetClusterAuthResult {
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly name: string;
    readonly region: string;
    /**
     * Token to use to authenticate with the cluster.
     */
    readonly token: string;
}
/**
 * Get an authentication token to communicate with an EKS cluster.
 *
 * Uses IAM credentials from the AWS provider to generate a temporary token that is compatible with
 * [AWS IAM Authenticator](https://github.com/kubernetes-sigs/aws-iam-authenticator) authentication.
 * This can be used to authenticate to an EKS cluster or to a cluster that has the AWS IAM Authenticator
 * server configured.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.eks.getCluster({
 *     name: "example",
 * });
 * const exampleGetClusterAuth = aws.eks.getClusterAuth({
 *     name: "example",
 * });
 * ```
 */
export declare function getClusterAuthOutput(args: GetClusterAuthOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetClusterAuthResult>;
/**
 * A collection of arguments for invoking getClusterAuth.
 */
export interface GetClusterAuthOutputArgs {
    /**
     * Name of the cluster
     */
    name: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
