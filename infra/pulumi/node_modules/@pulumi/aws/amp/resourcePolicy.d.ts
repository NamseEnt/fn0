import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages an Amazon Managed Service for Prometheus (AMP) Resource Policy.
 *
 * Resource-based policies allow you to grant permissions to other AWS accounts or services to access your Prometheus workspace. This enables cross-account access and fine-grained permissions for workspace sharing.
 *
 * ## Example Usage
 *
 * ### Basic Resource Policy
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const exampleWorkspace = new aws.amp.Workspace("example", {alias: "example-workspace"});
 * const current = aws.getCallerIdentity({});
 * const example = pulumi.all([current, exampleWorkspace.arn]).apply(([current, arn]) => aws.iam.getPolicyDocumentOutput({
 *     statements: [{
 *         effect: "Allow",
 *         principals: [{
 *             type: "AWS",
 *             identifiers: [current.accountId],
 *         }],
 *         actions: [
 *             "aps:RemoteWrite",
 *             "aps:QueryMetrics",
 *             "aps:GetSeries",
 *             "aps:GetLabels",
 *             "aps:GetMetricMetadata",
 *         ],
 *         resources: [arn],
 *     }],
 * }));
 * const exampleResourcePolicy = new aws.amp.ResourcePolicy("example", {
 *     workspaceId: exampleWorkspace.id,
 *     policyDocument: example.apply(example => example.json),
 * });
 * ```
 *
 * ### Cross-Account Access
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.amp.Workspace("example", {alias: "example-workspace"});
 * const crossAccount = aws.iam.getPolicyDocumentOutput({
 *     statements: [{
 *         effect: "Allow",
 *         principals: [{
 *             type: "AWS",
 *             identifiers: ["arn:aws:iam::123456789012:root"],
 *         }],
 *         actions: [
 *             "aps:RemoteWrite",
 *             "aps:QueryMetrics",
 *         ],
 *         resources: [example.arn],
 *     }],
 * });
 * const crossAccountResourcePolicy = new aws.amp.ResourcePolicy("cross_account", {
 *     workspaceId: example.id,
 *     policyDocument: crossAccount.apply(crossAccount => crossAccount.json),
 * });
 * ```
 *
 * ### Service-Specific Access
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.amp.Workspace("example", {alias: "example-workspace"});
 * const serviceAccess = aws.iam.getPolicyDocumentOutput({
 *     statements: [{
 *         effect: "Allow",
 *         principals: [{
 *             type: "Service",
 *             identifiers: ["grafana.amazonaws.com"],
 *         }],
 *         actions: [
 *             "aps:QueryMetrics",
 *             "aps:GetSeries",
 *             "aps:GetLabels",
 *             "aps:GetMetricMetadata",
 *         ],
 *         resources: [example.arn],
 *     }],
 * });
 * const serviceAccessResourcePolicy = new aws.amp.ResourcePolicy("service_access", {
 *     workspaceId: example.id,
 *     policyDocument: serviceAccess.apply(serviceAccess => serviceAccess.json),
 * });
 * ```
 *
 * ## Supported Actions
 *
 * The following actions are supported in resource policies for Prometheus workspaces:
 *
 * * `aps:RemoteWrite` - Allows writing metrics to the workspace
 * * `aps:QueryMetrics` - Allows querying metrics from the workspace
 * * `aps:GetSeries` - Allows retrieving time series data
 * * `aps:GetLabels` - Allows retrieving label names and values
 * * `aps:GetMetricMetadata` - Allows retrieving metric metadata
 *
 * ## Notes
 *
 * * Only Prometheus-compatible APIs can be used for workspace sharing. Non-Prometheus-compatible APIs added to the policy will be ignored.
 * * If your workspace uses customer-managed KMS keys for encryption, you must grant the principals in your resource-based policy access to those KMS keys through KMS grants.
 * * The resource ARN in the policy document must match the workspace ARN that the policy is being attached to.
 * * Resource policies enable cross-account access and fine-grained permissions for Prometheus workspaces.
 *
 * ## Import
 *
 * Using `pulumi import`, import AMP Resource Policies using the workspace ID. For example:
 *
 * ```sh
 * $ pulumi import aws:amp/resourcePolicy:ResourcePolicy example ws-12345678-90ab-cdef-1234-567890abcdef
 * ```
 */
export declare class ResourcePolicy extends pulumi.CustomResource {
    /**
     * Get an existing ResourcePolicy resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ResourcePolicyState, opts?: pulumi.CustomResourceOptions): ResourcePolicy;
    /**
     * Returns true if the given object is an instance of ResourcePolicy.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ResourcePolicy;
    /**
     * The JSON policy document to use as the resource-based policy. This policy defines the permissions that other AWS accounts or services have to access your workspace.
     *
     * The following arguments are optional:
     */
    readonly policyDocument: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The revision ID of the current resource-based policy.
     */
    readonly revisionId: pulumi.Output<string>;
    readonly timeouts: pulumi.Output<outputs.amp.ResourcePolicyTimeouts | undefined>;
    /**
     * The ID of the workspace to attach the resource-based policy to.
     */
    readonly workspaceId: pulumi.Output<string>;
    /**
     * Create a ResourcePolicy resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ResourcePolicyArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ResourcePolicy resources.
 */
export interface ResourcePolicyState {
    /**
     * The JSON policy document to use as the resource-based policy. This policy defines the permissions that other AWS accounts or services have to access your workspace.
     *
     * The following arguments are optional:
     */
    policyDocument?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The revision ID of the current resource-based policy.
     */
    revisionId?: pulumi.Input<string>;
    timeouts?: pulumi.Input<inputs.amp.ResourcePolicyTimeouts>;
    /**
     * The ID of the workspace to attach the resource-based policy to.
     */
    workspaceId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a ResourcePolicy resource.
 */
export interface ResourcePolicyArgs {
    /**
     * The JSON policy document to use as the resource-based policy. This policy defines the permissions that other AWS accounts or services have to access your workspace.
     *
     * The following arguments are optional:
     */
    policyDocument: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The revision ID of the current resource-based policy.
     */
    revisionId?: pulumi.Input<string>;
    timeouts?: pulumi.Input<inputs.amp.ResourcePolicyTimeouts>;
    /**
     * The ID of the workspace to attach the resource-based policy to.
     */
    workspaceId: pulumi.Input<string>;
}
