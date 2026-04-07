import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS DataZone Glossary Term.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.iam.Role("example", {
 *     name: "example",
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
 *         name: "example",
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
 * const exampleDomain = new aws.datazone.Domain("example", {
 *     name: "example_name",
 *     domainExecutionRole: example.arn,
 * });
 * const exampleSecurityGroup = new aws.ec2.SecurityGroup("example", {name: "example_name"});
 * const exampleProject = new aws.datazone.Project("example", {
 *     domainIdentifier: exampleDomain.id,
 *     glossaryTerms: ["2N8w6XJCwZf"],
 *     name: "example",
 *     skipDeletionCheck: true,
 * });
 * const exampleGlossary = new aws.datazone.Glossary("example", {
 *     description: "description",
 *     name: "example",
 *     owningProjectIdentifier: exampleProject.id,
 *     status: "ENABLED",
 *     domainIdentifier: exampleProject.domainIdentifier,
 * });
 * const exampleGlossaryTerm = new aws.datazone.GlossaryTerm("example", {
 *     domainIdentifier: exampleDomain.id,
 *     glossaryIdentifier: exampleGlossary.id,
 *     name: "example",
 *     status: "ENABLED",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import DataZone Glossary Term using a comma-delimited string combining the `domain_identifier`, `id`, and the `glossary_identifier`. For example:
 *
 * ```sh
 * $ pulumi import aws:datazone/glossaryTerm:GlossaryTerm example domain-id,glossary-term-id,glossary-id
 * ```
 */
export declare class GlossaryTerm extends pulumi.CustomResource {
    /**
     * Get an existing GlossaryTerm resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: GlossaryTermState, opts?: pulumi.CustomResourceOptions): GlossaryTerm;
    /**
     * Returns true if the given object is an instance of GlossaryTerm.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is GlossaryTerm;
    /**
     * Time of glossary term creation.
     */
    readonly createdAt: pulumi.Output<string>;
    /**
     * Creator of glossary term.
     */
    readonly createdBy: pulumi.Output<string>;
    /**
     * Identifier of domain.
     */
    readonly domainIdentifier: pulumi.Output<string | undefined>;
    /**
     * Identifier of glossary.
     */
    readonly glossaryIdentifier: pulumi.Output<string>;
    /**
     * Long description of entry.
     */
    readonly longDescription: pulumi.Output<string | undefined>;
    /**
     * Name of glossary term.
     *
     * The following arguments are optional:
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Short description of entry.
     */
    readonly shortDescription: pulumi.Output<string | undefined>;
    /**
     * If glossary term is ENABLED or DISABLED.
     */
    readonly status: pulumi.Output<string | undefined>;
    /**
     * Object classifying the term relations through the following attributes:
     */
    readonly termRelations: pulumi.Output<outputs.datazone.GlossaryTermTermRelations | undefined>;
    readonly timeouts: pulumi.Output<outputs.datazone.GlossaryTermTimeouts | undefined>;
    /**
     * Create a GlossaryTerm resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: GlossaryTermArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering GlossaryTerm resources.
 */
export interface GlossaryTermState {
    /**
     * Time of glossary term creation.
     */
    createdAt?: pulumi.Input<string>;
    /**
     * Creator of glossary term.
     */
    createdBy?: pulumi.Input<string>;
    /**
     * Identifier of domain.
     */
    domainIdentifier?: pulumi.Input<string>;
    /**
     * Identifier of glossary.
     */
    glossaryIdentifier?: pulumi.Input<string>;
    /**
     * Long description of entry.
     */
    longDescription?: pulumi.Input<string>;
    /**
     * Name of glossary term.
     *
     * The following arguments are optional:
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Short description of entry.
     */
    shortDescription?: pulumi.Input<string>;
    /**
     * If glossary term is ENABLED or DISABLED.
     */
    status?: pulumi.Input<string>;
    /**
     * Object classifying the term relations through the following attributes:
     */
    termRelations?: pulumi.Input<inputs.datazone.GlossaryTermTermRelations>;
    timeouts?: pulumi.Input<inputs.datazone.GlossaryTermTimeouts>;
}
/**
 * The set of arguments for constructing a GlossaryTerm resource.
 */
export interface GlossaryTermArgs {
    /**
     * Identifier of domain.
     */
    domainIdentifier?: pulumi.Input<string>;
    /**
     * Identifier of glossary.
     */
    glossaryIdentifier: pulumi.Input<string>;
    /**
     * Long description of entry.
     */
    longDescription?: pulumi.Input<string>;
    /**
     * Name of glossary term.
     *
     * The following arguments are optional:
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Short description of entry.
     */
    shortDescription?: pulumi.Input<string>;
    /**
     * If glossary term is ENABLED or DISABLED.
     */
    status?: pulumi.Input<string>;
    /**
     * Object classifying the term relations through the following attributes:
     */
    termRelations?: pulumi.Input<inputs.datazone.GlossaryTermTermRelations>;
    timeouts?: pulumi.Input<inputs.datazone.GlossaryTermTimeouts>;
}
