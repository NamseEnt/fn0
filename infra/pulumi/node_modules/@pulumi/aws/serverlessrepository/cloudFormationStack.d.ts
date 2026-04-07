import * as pulumi from "@pulumi/pulumi";
/**
 * Deploys an Application CloudFormation Stack from the Serverless Application Repository.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const current = aws.getPartition({});
 * const currentGetRegion = aws.getRegion({});
 * const postgres_rotator = new aws.serverlessrepository.CloudFormationStack("postgres-rotator", {
 *     name: "postgres-rotator",
 *     applicationId: "arn:aws:serverlessrepo:us-east-1:297356227824:applications/SecretsManagerRDSPostgreSQLRotationSingleUser",
 *     capabilities: [
 *         "CAPABILITY_IAM",
 *         "CAPABILITY_RESOURCE_POLICY",
 *     ],
 *     parameters: {
 *         functionName: "func-postgres-rotator",
 *         endpoint: Promise.all([currentGetRegion, current]).then(([currentGetRegion, current]) => `secretsmanager.${currentGetRegion.region}.${current.dnsSuffix}`),
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Serverless Application Repository Stack using the CloudFormation Stack name (with or without the `serverlessrepo-` prefix) or the CloudFormation Stack ID. For example:
 *
 * ```sh
 * $ pulumi import aws:serverlessrepository/cloudFormationStack:CloudFormationStack example serverlessrepo-postgres-rotator
 * ```
 */
export declare class CloudFormationStack extends pulumi.CustomResource {
    /**
     * Get an existing CloudFormationStack resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: CloudFormationStackState, opts?: pulumi.CustomResourceOptions): CloudFormationStack;
    /**
     * Returns true if the given object is an instance of CloudFormationStack.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is CloudFormationStack;
    /**
     * The ARN of the application from the Serverless Application Repository.
     */
    readonly applicationId: pulumi.Output<string>;
    /**
     * A list of capabilities. Valid values are `CAPABILITY_IAM`, `CAPABILITY_NAMED_IAM`, `CAPABILITY_RESOURCE_POLICY`, or `CAPABILITY_AUTO_EXPAND`
     */
    readonly capabilities: pulumi.Output<string[]>;
    /**
     * The name of the stack to create. The resource deployed in AWS will be prefixed with `serverlessrepo-`
     */
    readonly name: pulumi.Output<string>;
    /**
     * A map of outputs from the stack.
     */
    readonly outputs: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * A map of Parameter structures that specify input parameters for the stack.
     */
    readonly parameters: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The version of the application to deploy. If not supplied, deploys the latest version.
     */
    readonly semanticVersion: pulumi.Output<string>;
    /**
     * A list of tags to associate with this stack. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Create a CloudFormationStack resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: CloudFormationStackArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering CloudFormationStack resources.
 */
export interface CloudFormationStackState {
    /**
     * The ARN of the application from the Serverless Application Repository.
     */
    applicationId?: pulumi.Input<string>;
    /**
     * A list of capabilities. Valid values are `CAPABILITY_IAM`, `CAPABILITY_NAMED_IAM`, `CAPABILITY_RESOURCE_POLICY`, or `CAPABILITY_AUTO_EXPAND`
     */
    capabilities?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The name of the stack to create. The resource deployed in AWS will be prefixed with `serverlessrepo-`
     */
    name?: pulumi.Input<string>;
    /**
     * A map of outputs from the stack.
     */
    outputs?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * A map of Parameter structures that specify input parameters for the stack.
     */
    parameters?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The version of the application to deploy. If not supplied, deploys the latest version.
     */
    semanticVersion?: pulumi.Input<string>;
    /**
     * A list of tags to associate with this stack. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
}
/**
 * The set of arguments for constructing a CloudFormationStack resource.
 */
export interface CloudFormationStackArgs {
    /**
     * The ARN of the application from the Serverless Application Repository.
     */
    applicationId: pulumi.Input<string>;
    /**
     * A list of capabilities. Valid values are `CAPABILITY_IAM`, `CAPABILITY_NAMED_IAM`, `CAPABILITY_RESOURCE_POLICY`, or `CAPABILITY_AUTO_EXPAND`
     */
    capabilities: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The name of the stack to create. The resource deployed in AWS will be prefixed with `serverlessrepo-`
     */
    name?: pulumi.Input<string>;
    /**
     * A map of Parameter structures that specify input parameters for the stack.
     */
    parameters?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The version of the application to deploy. If not supplied, deploys the latest version.
     */
    semanticVersion?: pulumi.Input<string>;
    /**
     * A list of tags to associate with this stack. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
