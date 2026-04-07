import * as pulumi from "@pulumi/pulumi";
/**
 * Resource for managing an AWS DataZone Glossary.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const domainExecutionRole = new aws.iam.Role("domain_execution_role", {
 *     name: "example_name",
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
 *         name: "example_name",
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
 *     name: "example_name",
 *     domainExecutionRole: domainExecutionRole.arn,
 * });
 * const testSecurityGroup = new aws.ec2.SecurityGroup("test", {name: "example_name"});
 * const testProject = new aws.datazone.Project("test", {
 *     domainIdentifier: test.id,
 *     glossaryTerms: ["2N8w6XJCwZf"],
 *     name: "example_name",
 *     description: "desc",
 *     skipDeletionCheck: true,
 * });
 * const testGlossary = new aws.datazone.Glossary("test", {
 *     description: "description",
 *     name: "example_name",
 *     owningProjectIdentifier: testProject.id,
 *     status: "DISABLED",
 *     domainIdentifier: testProject.domainIdentifier,
 * });
 * ```
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = new aws.datazone.Glossary("test", {
 *     description: "description",
 *     name: "example_name",
 *     owningProjectIdentifier: testAwsDatazoneProject.id,
 *     status: "DISABLED",
 *     domainIdentifier: testAwsDatazoneProject.domainIdentifier,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import DataZone Glossary using the import Datazone Glossary using a comma-delimited string combining the domain id, glossary id, and the id of the project it's under. For example:
 *
 * ```sh
 * $ pulumi import aws:datazone/glossary:Glossary example domain-id,glossary-id,owning-project-identifier
 * ```
 */
export declare class Glossary extends pulumi.CustomResource {
    /**
     * Get an existing Glossary resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: GlossaryState, opts?: pulumi.CustomResourceOptions): Glossary;
    /**
     * Returns true if the given object is an instance of Glossary.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Glossary;
    /**
     * Description of the glossary. Must have a length between 0 and 4096.
     */
    readonly description: pulumi.Output<string | undefined>;
    readonly domainIdentifier: pulumi.Output<string>;
    /**
     * Name of the glossary. Must have length between 1 and 256.
     */
    readonly name: pulumi.Output<string>;
    /**
     * ID of the project that owns business glossary. Must follow regex of ^[a-zA-Z0-9_-]{1,36}$.
     *
     * The following arguments are optional:
     */
    readonly owningProjectIdentifier: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Status of business glossary. Valid values are DISABLED and ENABLED.
     */
    readonly status: pulumi.Output<string | undefined>;
    /**
     * Create a Glossary resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: GlossaryArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Glossary resources.
 */
export interface GlossaryState {
    /**
     * Description of the glossary. Must have a length between 0 and 4096.
     */
    description?: pulumi.Input<string>;
    domainIdentifier?: pulumi.Input<string>;
    /**
     * Name of the glossary. Must have length between 1 and 256.
     */
    name?: pulumi.Input<string>;
    /**
     * ID of the project that owns business glossary. Must follow regex of ^[a-zA-Z0-9_-]{1,36}$.
     *
     * The following arguments are optional:
     */
    owningProjectIdentifier?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Status of business glossary. Valid values are DISABLED and ENABLED.
     */
    status?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a Glossary resource.
 */
export interface GlossaryArgs {
    /**
     * Description of the glossary. Must have a length between 0 and 4096.
     */
    description?: pulumi.Input<string>;
    domainIdentifier: pulumi.Input<string>;
    /**
     * Name of the glossary. Must have length between 1 and 256.
     */
    name?: pulumi.Input<string>;
    /**
     * ID of the project that owns business glossary. Must follow regex of ^[a-zA-Z0-9_-]{1,36}$.
     *
     * The following arguments are optional:
     */
    owningProjectIdentifier: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Status of business glossary. Valid values are DISABLED and ENABLED.
     */
    status?: pulumi.Input<string>;
}
