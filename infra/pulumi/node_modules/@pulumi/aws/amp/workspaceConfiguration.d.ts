import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages an AWS Managed Service for Prometheus Workspace Configuration.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.amp.Workspace("example", {});
 * const exampleWorkspaceConfiguration = new aws.amp.WorkspaceConfiguration("example", {
 *     workspaceId: example.id,
 *     retentionPeriodInDays: 60,
 *     limitsPerLabelSets: [
 *         {
 *             labelSet: {
 *                 env: "dev",
 *             },
 *             limits: {
 *                 maxSeries: 100000,
 *             },
 *         },
 *         {
 *             labelSet: {
 *                 env: "prod",
 *             },
 *             limits: {
 *                 maxSeries: 400000,
 *             },
 *         },
 *     ],
 * });
 * ```
 *
 * ### Setting up default bucket
 *
 * The default bucket limit is the maximum number of active time series that can be
 * ingested in the workspace, counting only time series that don’t match a defined
 * label set.
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.amp.Workspace("example", {});
 * const exampleWorkspaceConfiguration = new aws.amp.WorkspaceConfiguration("example", {
 *     workspaceId: example.id,
 *     limitsPerLabelSets: [{
 *         labelSet: {},
 *         limits: {
 *             maxSeries: 50000,
 *         },
 *     }],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import AMP (Managed Prometheus) Workspace Configuration using the `workspace_id`. For example
 *
 * ```sh
 * $ pulumi import aws:amp/workspaceConfiguration:WorkspaceConfiguration example ws-12345678-abcd-1234-abcd-123456789012
 * ```
 */
export declare class WorkspaceConfiguration extends pulumi.CustomResource {
    /**
     * Get an existing WorkspaceConfiguration resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: WorkspaceConfigurationState, opts?: pulumi.CustomResourceOptions): WorkspaceConfiguration;
    /**
     * Returns true if the given object is an instance of WorkspaceConfiguration.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is WorkspaceConfiguration;
    /**
     * Configuration block for setting limits on metrics with specific label sets. Detailed below.
     */
    readonly limitsPerLabelSets: pulumi.Output<outputs.amp.WorkspaceConfigurationLimitsPerLabelSet[] | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Number of days to retain metric data in the workspace.
     */
    readonly retentionPeriodInDays: pulumi.Output<number>;
    readonly timeouts: pulumi.Output<outputs.amp.WorkspaceConfigurationTimeouts | undefined>;
    /**
     * ID of the workspace to configure.
     *
     * The following arguments are optional:
     */
    readonly workspaceId: pulumi.Output<string>;
    /**
     * Create a WorkspaceConfiguration resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: WorkspaceConfigurationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering WorkspaceConfiguration resources.
 */
export interface WorkspaceConfigurationState {
    /**
     * Configuration block for setting limits on metrics with specific label sets. Detailed below.
     */
    limitsPerLabelSets?: pulumi.Input<pulumi.Input<inputs.amp.WorkspaceConfigurationLimitsPerLabelSet>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Number of days to retain metric data in the workspace.
     */
    retentionPeriodInDays?: pulumi.Input<number>;
    timeouts?: pulumi.Input<inputs.amp.WorkspaceConfigurationTimeouts>;
    /**
     * ID of the workspace to configure.
     *
     * The following arguments are optional:
     */
    workspaceId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a WorkspaceConfiguration resource.
 */
export interface WorkspaceConfigurationArgs {
    /**
     * Configuration block for setting limits on metrics with specific label sets. Detailed below.
     */
    limitsPerLabelSets?: pulumi.Input<pulumi.Input<inputs.amp.WorkspaceConfigurationLimitsPerLabelSet>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Number of days to retain metric data in the workspace.
     */
    retentionPeriodInDays?: pulumi.Input<number>;
    timeouts?: pulumi.Input<inputs.amp.WorkspaceConfigurationTimeouts>;
    /**
     * ID of the workspace to configure.
     *
     * The following arguments are optional:
     */
    workspaceId: pulumi.Input<string>;
}
