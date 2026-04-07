import * as pulumi from "@pulumi/pulumi";
/**
 * > **NOTE:**: To reliably detect drift between customer managed inline policies listed in this resource and actual policies attached to the role in the cloud, you currently need to run Pulumi with `pulumi up --refresh`. See [#4766](https://github.com/pulumi/pulumi-aws/issues/4766) for tracking making this work with regular `pulumi up` invocations.
 *
 * Resource for maintaining exclusive management of inline policies assigned to an AWS IAM (Identity & Access Management) role.
 *
 * !> This resource takes exclusive ownership over inline policies assigned to a role. This includes removal of inline policies which are not explicitly configured. To prevent persistent drift, ensure any `aws.iam.RolePolicy` resources managed alongside this resource are included in the `policyNames` argument.
 *
 * > Destruction of this resource means Pulumi will no longer manage reconciliation of the configured inline policy assignments. It __will not__ delete the configured policies from the role.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.iam.RolePoliciesExclusive("example", {
 *     roleName: exampleAwsIamRole.name,
 *     policyNames: [exampleAwsIamRolePolicy.name],
 * });
 * ```
 *
 * ### Disallow Inline Policies
 *
 * To automatically remove any configured inline policies, set the `policyNames` argument to an empty list.
 *
 * > This will not __prevent__ inline policies from being assigned to a role via Pulumi (or any other interface). This resource enables bringing inline policy assignments into a configured state, however, this reconciliation happens only when `apply` is proactively run.
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.iam.RolePoliciesExclusive("example", {
 *     roleName: exampleAwsIamRole.name,
 *     policyNames: [],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import exclusive management of inline policy assignments using the `role_name`. For example:
 *
 * ```sh
 * $ pulumi import aws:iam/rolePoliciesExclusive:RolePoliciesExclusive example MyRole
 * ```
 */
export declare class RolePoliciesExclusive extends pulumi.CustomResource {
    /**
     * Get an existing RolePoliciesExclusive resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: RolePoliciesExclusiveState, opts?: pulumi.CustomResourceOptions): RolePoliciesExclusive;
    /**
     * Returns true if the given object is an instance of RolePoliciesExclusive.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is RolePoliciesExclusive;
    /**
     * A list of inline policy names to be assigned to the role. Policies attached to this role but not configured in this argument will be removed.
     */
    readonly policyNames: pulumi.Output<string[]>;
    /**
     * IAM role name.
     */
    readonly roleName: pulumi.Output<string>;
    /**
     * Create a RolePoliciesExclusive resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: RolePoliciesExclusiveArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering RolePoliciesExclusive resources.
 */
export interface RolePoliciesExclusiveState {
    /**
     * A list of inline policy names to be assigned to the role. Policies attached to this role but not configured in this argument will be removed.
     */
    policyNames?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * IAM role name.
     */
    roleName?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a RolePoliciesExclusive resource.
 */
export interface RolePoliciesExclusiveArgs {
    /**
     * A list of inline policy names to be assigned to the role. Policies attached to this role but not configured in this argument will be removed.
     */
    policyNames: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * IAM role name.
     */
    roleName: pulumi.Input<string>;
}
