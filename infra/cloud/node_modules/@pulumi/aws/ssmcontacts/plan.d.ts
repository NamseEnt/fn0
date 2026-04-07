import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS SSM Contact Plan.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.ssmcontacts.Plan("example", {
 *     contactId: "arn:aws:ssm-contacts:us-west-2:123456789012:contact/contactalias",
 *     stages: [{
 *         durationInMinutes: 1,
 *     }],
 * });
 * ```
 *
 * ### Usage with SSM Contact
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const contact = new aws.ssmcontacts.Contact("contact", {
 *     alias: "alias",
 *     type: "PERSONAL",
 * });
 * const plan = new aws.ssmcontacts.Plan("plan", {
 *     contactId: contact.arn,
 *     stages: [{
 *         durationInMinutes: 1,
 *     }],
 * });
 * ```
 *
 * ### Usage With All Fields
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const escalationPlan = new aws.ssmcontacts.Contact("escalation_plan", {
 *     alias: "escalation-plan-alias",
 *     type: "ESCALATION",
 * });
 * const contactOne = new aws.ssmcontacts.Contact("contact_one", {
 *     alias: "alias",
 *     type: "PERSONAL",
 * });
 * const contactTwo = new aws.ssmcontacts.Contact("contact_two", {
 *     alias: "alias",
 *     type: "PERSONAL",
 * });
 * const test = new aws.ssmcontacts.Plan("test", {
 *     contactId: escalationPlan.arn,
 *     stages: [{
 *         durationInMinutes: 0,
 *         targets: [
 *             {
 *                 contactTargetInfo: {
 *                     isEssential: false,
 *                     contactId: contactOne.arn,
 *                 },
 *             },
 *             {
 *                 contactTargetInfo: {
 *                     isEssential: true,
 *                     contactId: contactTwo.arn,
 *                 },
 *             },
 *             {
 *                 channelTargetInfo: {
 *                     retryIntervalInMinutes: 2,
 *                     contactChannelId: channel.arn,
 *                 },
 *             },
 *         ],
 *     }],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import SSM Contact Plan using the Contact ARN. For example:
 *
 * ```sh
 * $ pulumi import aws:ssmcontacts/plan:Plan example {ARNValue}
 * ```
 */
export declare class Plan extends pulumi.CustomResource {
    /**
     * Get an existing Plan resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: PlanState, opts?: pulumi.CustomResourceOptions): Plan;
    /**
     * Returns true if the given object is an instance of Plan.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Plan;
    /**
     * The Amazon Resource Name (ARN) of the contact or escalation plan.
     */
    readonly contactId: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * One or more configuration blocks for specifying a list of stages that the escalation plan or engagement plan uses to engage contacts and contact methods. See Stage below for more details.
     */
    readonly stages: pulumi.Output<outputs.ssmcontacts.PlanStage[]>;
    /**
     * Create a Plan resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: PlanArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Plan resources.
 */
export interface PlanState {
    /**
     * The Amazon Resource Name (ARN) of the contact or escalation plan.
     */
    contactId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * One or more configuration blocks for specifying a list of stages that the escalation plan or engagement plan uses to engage contacts and contact methods. See Stage below for more details.
     */
    stages?: pulumi.Input<pulumi.Input<inputs.ssmcontacts.PlanStage>[]>;
}
/**
 * The set of arguments for constructing a Plan resource.
 */
export interface PlanArgs {
    /**
     * The Amazon Resource Name (ARN) of the contact or escalation plan.
     */
    contactId: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * One or more configuration blocks for specifying a list of stages that the escalation plan or engagement plan uses to engage contacts and contact methods. See Stage below for more details.
     */
    stages: pulumi.Input<pulumi.Input<inputs.ssmcontacts.PlanStage>[]>;
}
