import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS DataZone Form Type.
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
 *     name: "example-role",
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
 *     inlinePolicies: [{
 *         name: "example-policy",
 *         policy: JSON.stringify({
 *             Version: "2012-10-17",
 *             Statement: [{
 *                 Action: [
 *                     "datazone:*",
 *                     "ram:*",
 *                     "sso:*",
 *                     "kms:*",
 *                 ],
 *                 Effect: "Allow",
 *                 Resource: "*",
 *             }],
 *         }),
 *     }],
 * });
 * const test = new aws.datazone.Domain("test", {
 *     name: "example",
 *     domainExecutionRole: domainExecutionRole.arn,
 * });
 * const testSecurityGroup = new aws.ec2.SecurityGroup("test", {name: "example"});
 * const testProject = new aws.datazone.Project("test", {
 *     domainIdentifier: test.id,
 *     glossaryTerms: ["2N8w6XJCwZf"],
 *     name: "example name",
 *     description: "desc",
 *     skipDeletionCheck: true,
 * });
 * const testFormType = new aws.datazone.FormType("test", {
 *     description: "desc",
 *     name: "SageMakerModelFormType",
 *     domainIdentifier: test.id,
 *     owningProjectIdentifier: testProject.id,
 *     status: "DISABLED",
 *     model: {
 *         smithy: `\\tstructure SageMakerModelFormType {
 * \\t\\t\\t@required
 * \\t\\t\\t@amazon.datazone#searchable
 * \\t\\t\\tmodelName: String
 *
 * \\t\\t\\t@required
 * \\t\\t\\tmodelArn: String
 *
 * \\t\\t\\t@required
 * \\t\\t\\tcreationTime: String
 * \\t\\t\\t}
 * `,
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import DataZone Form Type using a comma separated value of `domain_identifier`,`name`,`revision`. For example:
 *
 * ```sh
 * $ pulumi import aws:datazone/formType:FormType example domain_identifier,name,revision
 * ```
 */
export declare class FormType extends pulumi.CustomResource {
    /**
     * Get an existing FormType resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: FormTypeState, opts?: pulumi.CustomResourceOptions): FormType;
    /**
     * Returns true if the given object is an instance of FormType.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is FormType;
    /**
     * Creation time of the Form Type.
     */
    readonly createdAt: pulumi.Output<string>;
    /**
     * Creator of the Form Type.
     */
    readonly createdBy: pulumi.Output<string>;
    /**
     * Description of form type. Must have a length of between 1 and 2048 characters.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * Identifier of the domain.
     */
    readonly domainIdentifier: pulumi.Output<string>;
    readonly imports: pulumi.Output<outputs.datazone.FormTypeImport[]>;
    /**
     * Object of the model of the form type that contains the following attributes.
     */
    readonly model: pulumi.Output<outputs.datazone.FormTypeModel | undefined>;
    /**
     * Name of the form type. Must be the name of the structure in smithy document.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Origin domain id of the Form Type.
     */
    readonly originDomainId: pulumi.Output<string>;
    /**
     * Origin project id of the Form Type.
     */
    readonly originProjectId: pulumi.Output<string>;
    /**
     * Identifier of project that owns the form type. Must follow regex of ^[a-zA-Z0-9_-]{1,36}.
     */
    readonly owningProjectIdentifier: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Revision of the Form Type.
     */
    readonly revision: pulumi.Output<string>;
    readonly status: pulumi.Output<string>;
    readonly timeouts: pulumi.Output<outputs.datazone.FormTypeTimeouts | undefined>;
    /**
     * Create a FormType resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: FormTypeArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering FormType resources.
 */
export interface FormTypeState {
    /**
     * Creation time of the Form Type.
     */
    createdAt?: pulumi.Input<string>;
    /**
     * Creator of the Form Type.
     */
    createdBy?: pulumi.Input<string>;
    /**
     * Description of form type. Must have a length of between 1 and 2048 characters.
     */
    description?: pulumi.Input<string>;
    /**
     * Identifier of the domain.
     */
    domainIdentifier?: pulumi.Input<string>;
    imports?: pulumi.Input<pulumi.Input<inputs.datazone.FormTypeImport>[]>;
    /**
     * Object of the model of the form type that contains the following attributes.
     */
    model?: pulumi.Input<inputs.datazone.FormTypeModel>;
    /**
     * Name of the form type. Must be the name of the structure in smithy document.
     */
    name?: pulumi.Input<string>;
    /**
     * Origin domain id of the Form Type.
     */
    originDomainId?: pulumi.Input<string>;
    /**
     * Origin project id of the Form Type.
     */
    originProjectId?: pulumi.Input<string>;
    /**
     * Identifier of project that owns the form type. Must follow regex of ^[a-zA-Z0-9_-]{1,36}.
     */
    owningProjectIdentifier?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Revision of the Form Type.
     */
    revision?: pulumi.Input<string>;
    status?: pulumi.Input<string>;
    timeouts?: pulumi.Input<inputs.datazone.FormTypeTimeouts>;
}
/**
 * The set of arguments for constructing a FormType resource.
 */
export interface FormTypeArgs {
    /**
     * Description of form type. Must have a length of between 1 and 2048 characters.
     */
    description?: pulumi.Input<string>;
    /**
     * Identifier of the domain.
     */
    domainIdentifier: pulumi.Input<string>;
    /**
     * Object of the model of the form type that contains the following attributes.
     */
    model?: pulumi.Input<inputs.datazone.FormTypeModel>;
    /**
     * Name of the form type. Must be the name of the structure in smithy document.
     */
    name?: pulumi.Input<string>;
    /**
     * Identifier of project that owns the form type. Must follow regex of ^[a-zA-Z0-9_-]{1,36}.
     */
    owningProjectIdentifier: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    status?: pulumi.Input<string>;
    timeouts?: pulumi.Input<inputs.datazone.FormTypeTimeouts>;
}
