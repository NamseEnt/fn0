import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS DataZone Domain.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const domainExecutionRole = new aws.iam.Role("domain_execution_role", {
 *     name: "my_domain_execution_role",
 *     assumeRolePolicy: JSON.stringify({
 *         Version: "2012-10-17",
 *         Statement: [
 *             {
 *                 Action: [
 *                     "sts:AssumeRole",
 *                     "sts:TagSession",
 *                 ],
 *                 Effect: "Allow",
 *                 Principal: {
 *                     Service: "datazone.amazonaws.com",
 *                 },
 *             },
 *             {
 *                 Action: [
 *                     "sts:AssumeRole",
 *                     "sts:TagSession",
 *                 ],
 *                 Effect: "Allow",
 *                 Principal: {
 *                     Service: "cloudformation.amazonaws.com",
 *                 },
 *             },
 *         ],
 *     }),
 * });
 * const domainExecutionRoleRolePolicy = new aws.iam.RolePolicy("domain_execution_role", {
 *     role: domainExecutionRole.name,
 *     policy: JSON.stringify({
 *         Version: "2012-10-17",
 *         Statement: [{
 *             Action: [
 *                 "datazone:*",
 *                 "ram:*",
 *                 "sso:*",
 *                 "kms:*",
 *             ],
 *             Effect: "Allow",
 *             Resource: "*",
 *         }],
 *     }),
 * });
 * const example = new aws.datazone.Domain("example", {
 *     name: "example",
 *     domainExecutionRole: domainExecutionRole.arn,
 * });
 * ```
 *
 * ### V2 Domain
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const current = aws.getCallerIdentity({});
 * // IAM role for Domain Execution
 * const assumeRoleDomainExecution = current.then(current => aws.iam.getPolicyDocument({
 *     statements: [{
 *         actions: [
 *             "sts:AssumeRole",
 *             "sts:TagSession",
 *             "sts:SetContext",
 *         ],
 *         principals: [{
 *             type: "Service",
 *             identifiers: ["datazone.amazonaws.com"],
 *         }],
 *         conditions: [
 *             {
 *                 test: "StringEquals",
 *                 values: [current.accountId],
 *                 variable: "aws:SourceAccount",
 *             },
 *             {
 *                 test: "ForAllValues:StringLike",
 *                 values: ["datazone*"],
 *                 variable: "aws:TagKeys",
 *             },
 *         ],
 *     }],
 * }));
 * const domainExecution = new aws.iam.Role("domain_execution", {
 *     assumeRolePolicy: assumeRoleDomainExecution.then(assumeRoleDomainExecution => assumeRoleDomainExecution.json),
 *     name: "example-domain-execution-role",
 * });
 * const domainExecutionRole = aws.iam.getPolicy({
 *     name: "SageMakerStudioDomainExecutionRolePolicy",
 * });
 * const domainExecutionRolePolicyAttachment = new aws.iam.RolePolicyAttachment("domain_execution", {
 *     policyArn: domainExecutionRole.then(domainExecutionRole => domainExecutionRole.arn),
 *     role: domainExecution.name,
 * });
 * // IAM role for Domain Service
 * const assumeRoleDomainService = current.then(current => aws.iam.getPolicyDocument({
 *     statements: [{
 *         actions: ["sts:AssumeRole"],
 *         principals: [{
 *             type: "Service",
 *             identifiers: ["datazone.amazonaws.com"],
 *         }],
 *         conditions: [{
 *             test: "StringEquals",
 *             values: [current.accountId],
 *             variable: "aws:SourceAccount",
 *         }],
 *     }],
 * }));
 * const domainService = new aws.iam.Role("domain_service", {
 *     assumeRolePolicy: assumeRoleDomainService.then(assumeRoleDomainService => assumeRoleDomainService.json),
 *     name: "example-domain-service-role",
 * });
 * const domainServiceRole = aws.iam.getPolicy({
 *     name: "SageMakerStudioDomainServiceRolePolicy",
 * });
 * const domainServiceRolePolicyAttachment = new aws.iam.RolePolicyAttachment("domain_service", {
 *     policyArn: domainServiceRole.then(domainServiceRole => domainServiceRole.arn),
 *     role: domainService.name,
 * });
 * // DataZone Domain V2
 * const example = new aws.datazone.Domain("example", {
 *     name: "example-domain",
 *     domainExecutionRole: domainExecution.arn,
 *     domainVersion: "V2",
 *     serviceRole: domainService.arn,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import DataZone Domain using the `domain_id`. For example:
 *
 * ```sh
 * $ pulumi import aws:datazone/domain:Domain example domain-id-12345678
 * ```
 */
export declare class Domain extends pulumi.CustomResource {
    /**
     * Get an existing Domain resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: DomainState, opts?: pulumi.CustomResourceOptions): Domain;
    /**
     * Returns true if the given object is an instance of Domain.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Domain;
    /**
     * ARN of the Domain.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Description of the Domain.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * ARN of the role used by DataZone to configure the Domain.
     *
     * The following arguments are optional:
     */
    readonly domainExecutionRole: pulumi.Output<string>;
    /**
     * Version of the Domain. Valid values are `V1` and `V2`. Defaults to `V1`.
     */
    readonly domainVersion: pulumi.Output<string>;
    /**
     * ARN of the KMS key used to encrypt the Amazon DataZone domain, metadata and reporting data.
     */
    readonly kmsKeyIdentifier: pulumi.Output<string | undefined>;
    /**
     * Name of the Domain.
     */
    readonly name: pulumi.Output<string>;
    /**
     * URL of the data portal for the Domain.
     */
    readonly portalUrl: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * ARN of the service role used by DataZone. Required when `domainVersion` is set to `V2`.
     */
    readonly serviceRole: pulumi.Output<string | undefined>;
    /**
     * Single sign on options, used to [enable AWS IAM Identity Center](https://docs.aws.amazon.com/datazone/latest/userguide/enable-IAM-identity-center-for-datazone.html) for DataZone.
     */
    readonly singleSignOn: pulumi.Output<outputs.datazone.DomainSingleSignOn | undefined>;
    /**
     * Whether to skip the deletion check for the Domain.
     */
    readonly skipDeletionCheck: pulumi.Output<boolean | undefined>;
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * Map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    readonly timeouts: pulumi.Output<outputs.datazone.DomainTimeouts | undefined>;
    /**
     * Create a Domain resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: DomainArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Domain resources.
 */
export interface DomainState {
    /**
     * ARN of the Domain.
     */
    arn?: pulumi.Input<string>;
    /**
     * Description of the Domain.
     */
    description?: pulumi.Input<string>;
    /**
     * ARN of the role used by DataZone to configure the Domain.
     *
     * The following arguments are optional:
     */
    domainExecutionRole?: pulumi.Input<string>;
    /**
     * Version of the Domain. Valid values are `V1` and `V2`. Defaults to `V1`.
     */
    domainVersion?: pulumi.Input<string>;
    /**
     * ARN of the KMS key used to encrypt the Amazon DataZone domain, metadata and reporting data.
     */
    kmsKeyIdentifier?: pulumi.Input<string>;
    /**
     * Name of the Domain.
     */
    name?: pulumi.Input<string>;
    /**
     * URL of the data portal for the Domain.
     */
    portalUrl?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ARN of the service role used by DataZone. Required when `domainVersion` is set to `V2`.
     */
    serviceRole?: pulumi.Input<string>;
    /**
     * Single sign on options, used to [enable AWS IAM Identity Center](https://docs.aws.amazon.com/datazone/latest/userguide/enable-IAM-identity-center-for-datazone.html) for DataZone.
     */
    singleSignOn?: pulumi.Input<inputs.datazone.DomainSingleSignOn>;
    /**
     * Whether to skip the deletion check for the Domain.
     */
    skipDeletionCheck?: pulumi.Input<boolean>;
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    timeouts?: pulumi.Input<inputs.datazone.DomainTimeouts>;
}
/**
 * The set of arguments for constructing a Domain resource.
 */
export interface DomainArgs {
    /**
     * Description of the Domain.
     */
    description?: pulumi.Input<string>;
    /**
     * ARN of the role used by DataZone to configure the Domain.
     *
     * The following arguments are optional:
     */
    domainExecutionRole: pulumi.Input<string>;
    /**
     * Version of the Domain. Valid values are `V1` and `V2`. Defaults to `V1`.
     */
    domainVersion?: pulumi.Input<string>;
    /**
     * ARN of the KMS key used to encrypt the Amazon DataZone domain, metadata and reporting data.
     */
    kmsKeyIdentifier?: pulumi.Input<string>;
    /**
     * Name of the Domain.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ARN of the service role used by DataZone. Required when `domainVersion` is set to `V2`.
     */
    serviceRole?: pulumi.Input<string>;
    /**
     * Single sign on options, used to [enable AWS IAM Identity Center](https://docs.aws.amazon.com/datazone/latest/userguide/enable-IAM-identity-center-for-datazone.html) for DataZone.
     */
    singleSignOn?: pulumi.Input<inputs.datazone.DomainSingleSignOn>;
    /**
     * Whether to skip the deletion check for the Domain.
     */
    skipDeletionCheck?: pulumi.Input<boolean>;
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    timeouts?: pulumi.Input<inputs.datazone.DomainTimeouts>;
}
