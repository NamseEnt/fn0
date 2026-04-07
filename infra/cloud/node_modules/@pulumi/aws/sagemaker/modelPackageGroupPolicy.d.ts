import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a SageMaker AI Model Package Group Policy resource.
 *
 * ## Example Usage
 *
 * ### Basic usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 * import * as std from "@pulumi/std";
 *
 * const current = aws.getCallerIdentity({});
 * const exampleModelPackageGroup = new aws.sagemaker.ModelPackageGroup("example", {modelPackageGroupName: "example"});
 * const example = pulumi.all([exampleModelPackageGroup.arn, current]).apply(([arn, current]) => aws.iam.getPolicyDocumentOutput({
 *     statements: [{
 *         sid: "AddPermModelPackageGroup",
 *         actions: [
 *             "sagemaker:DescribeModelPackage",
 *             "sagemaker:ListModelPackages",
 *         ],
 *         resources: [arn],
 *         principals: [{
 *             identifiers: [current.accountId],
 *             type: "AWS",
 *         }],
 *     }],
 * }));
 * const exampleModelPackageGroupPolicy = new aws.sagemaker.ModelPackageGroupPolicy("example", {
 *     modelPackageGroupName: exampleModelPackageGroup.modelPackageGroupName,
 *     resourcePolicy: pulumi.jsonStringify(example.apply(example => std.jsondecodeOutput({
 *         input: example.json,
 *     })).apply(invoke => invoke.result)),
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import SageMaker AI Model Package Groups using the `name`. For example:
 *
 * ```sh
 * $ pulumi import aws:sagemaker/modelPackageGroupPolicy:ModelPackageGroupPolicy example example
 * ```
 */
export declare class ModelPackageGroupPolicy extends pulumi.CustomResource {
    /**
     * Get an existing ModelPackageGroupPolicy resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ModelPackageGroupPolicyState, opts?: pulumi.CustomResourceOptions): ModelPackageGroupPolicy;
    /**
     * Returns true if the given object is an instance of ModelPackageGroupPolicy.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ModelPackageGroupPolicy;
    /**
     * The name of the model package group.
     */
    readonly modelPackageGroupName: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    readonly resourcePolicy: pulumi.Output<string>;
    /**
     * Create a ModelPackageGroupPolicy resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ModelPackageGroupPolicyArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ModelPackageGroupPolicy resources.
 */
export interface ModelPackageGroupPolicyState {
    /**
     * The name of the model package group.
     */
    modelPackageGroupName?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    resourcePolicy?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a ModelPackageGroupPolicy resource.
 */
export interface ModelPackageGroupPolicyArgs {
    /**
     * The name of the model package group.
     */
    modelPackageGroupName: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    resourcePolicy: pulumi.Input<string>;
}
