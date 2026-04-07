import * as pulumi from "@pulumi/pulumi";
/**
 * Access Entry Configurations for an EKS Cluster.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.eks.getAccessEntry({
 *     clusterName: exampleAwsEksCluster.name,
 *     principalArn: exampleAwsIamRole.arn,
 * });
 * export const eksAccessEntryOutputs = exampleAwsEksAccessEntry;
 * ```
 */
export declare function getAccessEntry(args: GetAccessEntryArgs, opts?: pulumi.InvokeOptions): Promise<GetAccessEntryResult>;
/**
 * A collection of arguments for invoking getAccessEntry.
 */
export interface GetAccessEntryArgs {
    /**
     * Name of the EKS Cluster.
     */
    clusterName: string;
    /**
     * The IAM Principal ARN which requires Authentication access to the EKS cluster.
     */
    principalArn: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
    tags?: {
        [key: string]: string;
    };
    /**
     * (Optional) Key-value map of resource tags, including those inherited from the provider `defaultTags` configuration block.
     */
    tagsAll?: {
        [key: string]: string;
    };
}
/**
 * A collection of values returned by getAccessEntry.
 */
export interface GetAccessEntryResult {
    /**
     * Amazon Resource Name (ARN) of the Access Entry.
     */
    readonly accessEntryArn: string;
    readonly clusterName: string;
    /**
     * Date and time in [RFC3339 format](https://tools.ietf.org/html/rfc3339#section-5.8) that the EKS add-on was created.
     */
    readonly createdAt: string;
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    /**
     * List of string which can optionally specify the Kubernetes groups the user would belong to when creating an access entry.
     */
    readonly kubernetesGroups: string[];
    /**
     * Date and time in [RFC3339 format](https://tools.ietf.org/html/rfc3339#section-5.8) that the EKS add-on was updated.
     */
    readonly modifiedAt: string;
    readonly principalArn: string;
    readonly region: string;
    readonly tags?: {
        [key: string]: string;
    };
    /**
     * (Optional) Key-value map of resource tags, including those inherited from the provider `defaultTags` configuration block.
     */
    readonly tagsAll: {
        [key: string]: string;
    };
    /**
     * Defaults to STANDARD which provides the standard workflow. EC2_LINUX, EC2_WINDOWS, FARGATE_LINUX types disallow users to input a username or groups, and prevent associations.
     */
    readonly type: string;
    /**
     * Defaults to principal ARN if user is principal else defaults to assume-role/session-name is role is used.
     */
    readonly userName: string;
}
/**
 * Access Entry Configurations for an EKS Cluster.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.eks.getAccessEntry({
 *     clusterName: exampleAwsEksCluster.name,
 *     principalArn: exampleAwsIamRole.arn,
 * });
 * export const eksAccessEntryOutputs = exampleAwsEksAccessEntry;
 * ```
 */
export declare function getAccessEntryOutput(args: GetAccessEntryOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetAccessEntryResult>;
/**
 * A collection of arguments for invoking getAccessEntry.
 */
export interface GetAccessEntryOutputArgs {
    /**
     * Name of the EKS Cluster.
     */
    clusterName: pulumi.Input<string>;
    /**
     * The IAM Principal ARN which requires Authentication access to the EKS cluster.
     */
    principalArn: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * (Optional) Key-value map of resource tags, including those inherited from the provider `defaultTags` configuration block.
     */
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
