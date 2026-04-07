import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages an EKS Node Group, which can provision and optionally update an Auto Scaling Group of Kubernetes worker nodes compatible with EKS. Additional documentation about this functionality can be found in the [EKS User Guide](https://docs.aws.amazon.com/eks/latest/userguide/managed-node-groups.html).
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.eks.NodeGroup("example", {
 *     clusterName: exampleAwsEksCluster.name,
 *     nodeGroupName: "example",
 *     nodeRoleArn: exampleAwsIamRole.arn,
 *     subnetIds: exampleAwsSubnet.map(__item => __item.id),
 *     scalingConfig: {
 *         desiredSize: 1,
 *         maxSize: 2,
 *         minSize: 1,
 *     },
 *     updateConfig: {
 *         maxUnavailable: 1,
 *     },
 * }, {
 *     dependsOn: [
 *         example_AmazonEKSWorkerNodePolicy,
 *         example_AmazonEKSCNIPolicy,
 *         example_AmazonEC2ContainerRegistryReadOnly,
 *     ],
 * });
 * ```
 *
 * ### Ignoring Changes to Desired Size
 *
 * You can utilize [ignoreChanges](https://www.pulumi.com/docs/intro/concepts/programming-model/#ignorechanges) create an EKS Node Group with an initial size of running instances, then ignore any changes to that count caused externally (e.g. Application Autoscaling).
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.eks.NodeGroup("example", {scalingConfig: {
 *     desiredSize: 2,
 * }});
 * ```
 *
 * ### Example IAM Role for EKS Node Group
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.iam.Role("example", {
 *     name: "eks-node-group-example",
 *     assumeRolePolicy: JSON.stringify({
 *         Statement: [{
 *             Action: "sts:AssumeRole",
 *             Effect: "Allow",
 *             Principal: {
 *                 Service: "ec2.amazonaws.com",
 *             },
 *         }],
 *         Version: "2012-10-17",
 *     }),
 * });
 * const example_AmazonEKSWorkerNodePolicy = new aws.iam.RolePolicyAttachment("example-AmazonEKSWorkerNodePolicy", {
 *     policyArn: "arn:aws:iam::aws:policy/AmazonEKSWorkerNodePolicy",
 *     role: example.name,
 * });
 * const example_AmazonEKSCNIPolicy = new aws.iam.RolePolicyAttachment("example-AmazonEKS_CNI_Policy", {
 *     policyArn: "arn:aws:iam::aws:policy/AmazonEKS_CNI_Policy",
 *     role: example.name,
 * });
 * const example_AmazonEC2ContainerRegistryReadOnly = new aws.iam.RolePolicyAttachment("example-AmazonEC2ContainerRegistryReadOnly", {
 *     policyArn: "arn:aws:iam::aws:policy/AmazonEC2ContainerRegistryReadOnly",
 *     role: example.name,
 * });
 * ```
 *
 * ### Example Subnets for EKS Node Group
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 * import * as std from "@pulumi/std";
 *
 * const available = aws.getAvailabilityZones({
 *     state: "available",
 * });
 * const example: aws.ec2.Subnet[] = [];
 * for (const range = {value: 0}; range.value < 2; range.value++) {
 *     example.push(new aws.ec2.Subnet(`example-${range.value}`, {
 *         availabilityZone: available.then(available => available.names[range.value]),
 *         cidrBlock: std.cidrsubnet({
 *             input: exampleAwsVpc.cidrBlock,
 *             newbits: 8,
 *             netnum: range.value,
 *         }).then(invoke => invoke.result),
 *         vpcId: exampleAwsVpc.id,
 *     }));
 * }
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import EKS Node Groups using the `cluster_name` and `node_group_name` separated by a colon (`:`). For example:
 *
 * ```sh
 * $ pulumi import aws:eks/nodeGroup:NodeGroup my_node_group my_cluster:my_node_group
 * ```
 */
export declare class NodeGroup extends pulumi.CustomResource {
    /**
     * Get an existing NodeGroup resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: NodeGroupState, opts?: pulumi.CustomResourceOptions): NodeGroup;
    /**
     * Returns true if the given object is an instance of NodeGroup.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is NodeGroup;
    /**
     * Type of Amazon Machine Image (AMI) associated with the EKS Node Group. See the [AWS documentation](https://docs.aws.amazon.com/eks/latest/APIReference/API_Nodegroup.html#AmazonEKS-Type-Nodegroup-amiType) for valid values. This provider will only perform drift detection if a configuration value is provided.
     */
    readonly amiType: pulumi.Output<string>;
    /**
     * Amazon Resource Name (ARN) of the EKS Node Group.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Type of capacity associated with the EKS Node Group. Valid values: `ON_DEMAND`, `SPOT`. This provider will only perform drift detection if a configuration value is provided.
     */
    readonly capacityType: pulumi.Output<string>;
    /**
     * Name of the EKS Cluster.
     */
    readonly clusterName: pulumi.Output<string>;
    /**
     * Disk size in GiB for worker nodes. Defaults to `50` for Windows, `20` all other node groups. The provider will only perform drift detection if a configuration value is provided.
     */
    readonly diskSize: pulumi.Output<number>;
    /**
     * Force version update if existing pods are unable to be drained due to a pod disruption budget issue.
     */
    readonly forceUpdateVersion: pulumi.Output<boolean | undefined>;
    /**
     * List of instance types associated with the EKS Node Group. Defaults to `["t3.medium"]`. The provider will only perform drift detection if a configuration value is provided.
     */
    readonly instanceTypes: pulumi.Output<string[]>;
    /**
     * Key-value map of Kubernetes labels. Only labels that are applied with the EKS API are managed by this argument. Other Kubernetes labels applied to the EKS Node Group will not be managed.
     */
    readonly labels: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * Configuration block with Launch Template settings. See `launchTemplate` below for details. Conflicts with `remoteAccess`.
     */
    readonly launchTemplate: pulumi.Output<outputs.eks.NodeGroupLaunchTemplate | undefined>;
    /**
     * Name of the EKS Node Group. If omitted, the provider will assign a random, unique name. Conflicts with `nodeGroupNamePrefix`. The node group name can't be longer than 63 characters. It must start with a letter or digit, but can also include hyphens and underscores for the remaining characters.
     */
    readonly nodeGroupName: pulumi.Output<string>;
    /**
     * Creates a unique name beginning with the specified prefix. Conflicts with `nodeGroupName`.
     */
    readonly nodeGroupNamePrefix: pulumi.Output<string>;
    /**
     * The node auto repair configuration for the node group. See `nodeRepairConfig` below for details.
     */
    readonly nodeRepairConfig: pulumi.Output<outputs.eks.NodeGroupNodeRepairConfig>;
    /**
     * Amazon Resource Name (ARN) of the IAM Role that provides permissions for the EKS Node Group.
     */
    readonly nodeRoleArn: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * AMI version of the EKS Node Group. Defaults to latest version for Kubernetes version.
     */
    readonly releaseVersion: pulumi.Output<string>;
    /**
     * Configuration block with remote access settings. See `remoteAccess` below for details. Conflicts with `launchTemplate`.
     */
    readonly remoteAccess: pulumi.Output<outputs.eks.NodeGroupRemoteAccess | undefined>;
    /**
     * List of objects containing information about underlying resources.
     */
    readonly resources: pulumi.Output<outputs.eks.NodeGroupResource[]>;
    /**
     * Configuration block with scaling settings. See `scalingConfig` below for details.
     */
    readonly scalingConfig: pulumi.Output<outputs.eks.NodeGroupScalingConfig>;
    /**
     * Status of the EKS Node Group.
     */
    readonly status: pulumi.Output<string>;
    /**
     * Identifiers of EC2 Subnets to associate with the EKS Node Group.
     *
     * The following arguments are optional:
     */
    readonly subnetIds: pulumi.Output<string[]>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * A map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * The Kubernetes taints to be applied to the nodes in the node group. Maximum of 50 taints per node group. See taint below for details.
     */
    readonly taints: pulumi.Output<outputs.eks.NodeGroupTaint[] | undefined>;
    /**
     * Configuration block with update settings. See `updateConfig` below for details.
     */
    readonly updateConfig: pulumi.Output<outputs.eks.NodeGroupUpdateConfig>;
    /**
     * Kubernetes version. Defaults to EKS Cluster Kubernetes version. The provider will only perform drift detection if a configuration value is provided.
     */
    readonly version: pulumi.Output<string>;
    /**
     * Create a NodeGroup resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: NodeGroupArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering NodeGroup resources.
 */
export interface NodeGroupState {
    /**
     * Type of Amazon Machine Image (AMI) associated with the EKS Node Group. See the [AWS documentation](https://docs.aws.amazon.com/eks/latest/APIReference/API_Nodegroup.html#AmazonEKS-Type-Nodegroup-amiType) for valid values. This provider will only perform drift detection if a configuration value is provided.
     */
    amiType?: pulumi.Input<string>;
    /**
     * Amazon Resource Name (ARN) of the EKS Node Group.
     */
    arn?: pulumi.Input<string>;
    /**
     * Type of capacity associated with the EKS Node Group. Valid values: `ON_DEMAND`, `SPOT`. This provider will only perform drift detection if a configuration value is provided.
     */
    capacityType?: pulumi.Input<string>;
    /**
     * Name of the EKS Cluster.
     */
    clusterName?: pulumi.Input<string>;
    /**
     * Disk size in GiB for worker nodes. Defaults to `50` for Windows, `20` all other node groups. The provider will only perform drift detection if a configuration value is provided.
     */
    diskSize?: pulumi.Input<number>;
    /**
     * Force version update if existing pods are unable to be drained due to a pod disruption budget issue.
     */
    forceUpdateVersion?: pulumi.Input<boolean>;
    /**
     * List of instance types associated with the EKS Node Group. Defaults to `["t3.medium"]`. The provider will only perform drift detection if a configuration value is provided.
     */
    instanceTypes?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Key-value map of Kubernetes labels. Only labels that are applied with the EKS API are managed by this argument. Other Kubernetes labels applied to the EKS Node Group will not be managed.
     */
    labels?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Configuration block with Launch Template settings. See `launchTemplate` below for details. Conflicts with `remoteAccess`.
     */
    launchTemplate?: pulumi.Input<inputs.eks.NodeGroupLaunchTemplate>;
    /**
     * Name of the EKS Node Group. If omitted, the provider will assign a random, unique name. Conflicts with `nodeGroupNamePrefix`. The node group name can't be longer than 63 characters. It must start with a letter or digit, but can also include hyphens and underscores for the remaining characters.
     */
    nodeGroupName?: pulumi.Input<string>;
    /**
     * Creates a unique name beginning with the specified prefix. Conflicts with `nodeGroupName`.
     */
    nodeGroupNamePrefix?: pulumi.Input<string>;
    /**
     * The node auto repair configuration for the node group. See `nodeRepairConfig` below for details.
     */
    nodeRepairConfig?: pulumi.Input<inputs.eks.NodeGroupNodeRepairConfig>;
    /**
     * Amazon Resource Name (ARN) of the IAM Role that provides permissions for the EKS Node Group.
     */
    nodeRoleArn?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * AMI version of the EKS Node Group. Defaults to latest version for Kubernetes version.
     */
    releaseVersion?: pulumi.Input<string>;
    /**
     * Configuration block with remote access settings. See `remoteAccess` below for details. Conflicts with `launchTemplate`.
     */
    remoteAccess?: pulumi.Input<inputs.eks.NodeGroupRemoteAccess>;
    /**
     * List of objects containing information about underlying resources.
     */
    resources?: pulumi.Input<pulumi.Input<inputs.eks.NodeGroupResource>[]>;
    /**
     * Configuration block with scaling settings. See `scalingConfig` below for details.
     */
    scalingConfig?: pulumi.Input<inputs.eks.NodeGroupScalingConfig>;
    /**
     * Status of the EKS Node Group.
     */
    status?: pulumi.Input<string>;
    /**
     * Identifiers of EC2 Subnets to associate with the EKS Node Group.
     *
     * The following arguments are optional:
     */
    subnetIds?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * A map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The Kubernetes taints to be applied to the nodes in the node group. Maximum of 50 taints per node group. See taint below for details.
     */
    taints?: pulumi.Input<pulumi.Input<inputs.eks.NodeGroupTaint>[]>;
    /**
     * Configuration block with update settings. See `updateConfig` below for details.
     */
    updateConfig?: pulumi.Input<inputs.eks.NodeGroupUpdateConfig>;
    /**
     * Kubernetes version. Defaults to EKS Cluster Kubernetes version. The provider will only perform drift detection if a configuration value is provided.
     */
    version?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a NodeGroup resource.
 */
export interface NodeGroupArgs {
    /**
     * Type of Amazon Machine Image (AMI) associated with the EKS Node Group. See the [AWS documentation](https://docs.aws.amazon.com/eks/latest/APIReference/API_Nodegroup.html#AmazonEKS-Type-Nodegroup-amiType) for valid values. This provider will only perform drift detection if a configuration value is provided.
     */
    amiType?: pulumi.Input<string>;
    /**
     * Type of capacity associated with the EKS Node Group. Valid values: `ON_DEMAND`, `SPOT`. This provider will only perform drift detection if a configuration value is provided.
     */
    capacityType?: pulumi.Input<string>;
    /**
     * Name of the EKS Cluster.
     */
    clusterName: pulumi.Input<string>;
    /**
     * Disk size in GiB for worker nodes. Defaults to `50` for Windows, `20` all other node groups. The provider will only perform drift detection if a configuration value is provided.
     */
    diskSize?: pulumi.Input<number>;
    /**
     * Force version update if existing pods are unable to be drained due to a pod disruption budget issue.
     */
    forceUpdateVersion?: pulumi.Input<boolean>;
    /**
     * List of instance types associated with the EKS Node Group. Defaults to `["t3.medium"]`. The provider will only perform drift detection if a configuration value is provided.
     */
    instanceTypes?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Key-value map of Kubernetes labels. Only labels that are applied with the EKS API are managed by this argument. Other Kubernetes labels applied to the EKS Node Group will not be managed.
     */
    labels?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Configuration block with Launch Template settings. See `launchTemplate` below for details. Conflicts with `remoteAccess`.
     */
    launchTemplate?: pulumi.Input<inputs.eks.NodeGroupLaunchTemplate>;
    /**
     * Name of the EKS Node Group. If omitted, the provider will assign a random, unique name. Conflicts with `nodeGroupNamePrefix`. The node group name can't be longer than 63 characters. It must start with a letter or digit, but can also include hyphens and underscores for the remaining characters.
     */
    nodeGroupName?: pulumi.Input<string>;
    /**
     * Creates a unique name beginning with the specified prefix. Conflicts with `nodeGroupName`.
     */
    nodeGroupNamePrefix?: pulumi.Input<string>;
    /**
     * The node auto repair configuration for the node group. See `nodeRepairConfig` below for details.
     */
    nodeRepairConfig?: pulumi.Input<inputs.eks.NodeGroupNodeRepairConfig>;
    /**
     * Amazon Resource Name (ARN) of the IAM Role that provides permissions for the EKS Node Group.
     */
    nodeRoleArn: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * AMI version of the EKS Node Group. Defaults to latest version for Kubernetes version.
     */
    releaseVersion?: pulumi.Input<string>;
    /**
     * Configuration block with remote access settings. See `remoteAccess` below for details. Conflicts with `launchTemplate`.
     */
    remoteAccess?: pulumi.Input<inputs.eks.NodeGroupRemoteAccess>;
    /**
     * Configuration block with scaling settings. See `scalingConfig` below for details.
     */
    scalingConfig: pulumi.Input<inputs.eks.NodeGroupScalingConfig>;
    /**
     * Identifiers of EC2 Subnets to associate with the EKS Node Group.
     *
     * The following arguments are optional:
     */
    subnetIds: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The Kubernetes taints to be applied to the nodes in the node group. Maximum of 50 taints per node group. See taint below for details.
     */
    taints?: pulumi.Input<pulumi.Input<inputs.eks.NodeGroupTaint>[]>;
    /**
     * Configuration block with update settings. See `updateConfig` below for details.
     */
    updateConfig?: pulumi.Input<inputs.eks.NodeGroupUpdateConfig>;
    /**
     * Kubernetes version. Defaults to EKS Cluster Kubernetes version. The provider will only perform drift detection if a configuration value is provided.
     */
    version?: pulumi.Input<string>;
}
