import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a CodeBuild Resource Policy Resource.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.codebuild.ReportGroup("example", {
 *     name: "example",
 *     type: "TEST",
 *     exportConfig: {
 *         type: "NO_EXPORT",
 *     },
 * });
 * const current = aws.getPartition({});
 * const currentGetCallerIdentity = aws.getCallerIdentity({});
 * const exampleResourcePolicy = new aws.codebuild.ResourcePolicy("example", {
 *     resourceArn: example.arn,
 *     policy: pulumi.jsonStringify({
 *         Version: "2012-10-17",
 *         Id: "default",
 *         Statement: [{
 *             Sid: "default",
 *             Effect: "Allow",
 *             Principal: {
 *                 AWS: Promise.all([current, currentGetCallerIdentity]).then(([current, currentGetCallerIdentity]) => `arn:${current.partition}:iam::${currentGetCallerIdentity.accountId}:root`),
 *             },
 *             Action: [
 *                 "codebuild:BatchGetReportGroups",
 *                 "codebuild:BatchGetReports",
 *                 "codebuild:ListReportsForReportGroup",
 *                 "codebuild:DescribeTestCases",
 *             ],
 *             Resource: example.arn,
 *         }],
 *     }),
 * });
 * ```
 *
 * ## Import
 *
 * ### Identity Schema
 *
 * #### Required
 *
 * - `arn` (String) Amazon Resource Name (ARN) of the CodeBuild resource.
 *
 * Using `pulumi import`, import CodeBuild Resource Policy using the CodeBuild Resource Policy arn. For example:
 *
 * console
 *
 * % pulumi import aws_codebuild_resource_policy.example arn:aws:codebuild:us-west-2:123456789:report-group/report-group-name
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
     * A JSON-formatted resource policy. For more information, see [Sharing a Projec](https://docs.aws.amazon.com/codebuild/latest/userguide/project-sharing.html#project-sharing-share) and [Sharing a Report Group](https://docs.aws.amazon.com/codebuild/latest/userguide/report-groups-sharing.html#report-groups-sharing-share).
     */
    readonly policy: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The ARN of the Project or ReportGroup resource you want to associate with a resource policy.
     */
    readonly resourceArn: pulumi.Output<string>;
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
     * A JSON-formatted resource policy. For more information, see [Sharing a Projec](https://docs.aws.amazon.com/codebuild/latest/userguide/project-sharing.html#project-sharing-share) and [Sharing a Report Group](https://docs.aws.amazon.com/codebuild/latest/userguide/report-groups-sharing.html#report-groups-sharing-share).
     */
    policy?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The ARN of the Project or ReportGroup resource you want to associate with a resource policy.
     */
    resourceArn?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a ResourcePolicy resource.
 */
export interface ResourcePolicyArgs {
    /**
     * A JSON-formatted resource policy. For more information, see [Sharing a Projec](https://docs.aws.amazon.com/codebuild/latest/userguide/project-sharing.html#project-sharing-share) and [Sharing a Report Group](https://docs.aws.amazon.com/codebuild/latest/userguide/report-groups-sharing.html#report-groups-sharing-share).
     */
    policy: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The ARN of the Project or ReportGroup resource you want to associate with a resource policy.
     */
    resourceArn: pulumi.Input<string>;
}
