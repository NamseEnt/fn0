import * as pulumi from "@pulumi/pulumi";
/**
 * Resource for managing an AWS WorkSpaces Web Session Logger Association.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const examplePortal = new aws.workspacesweb.Portal("example", {displayName: "example"});
 * const exampleBucket = new aws.s3.Bucket("example", {
 *     bucket: "example-session-logs",
 *     forceDestroy: true,
 * });
 * const example = aws.iam.getPolicyDocumentOutput({
 *     statements: [{
 *         effect: "Allow",
 *         principals: [{
 *             type: "Service",
 *             identifiers: ["workspaces-web.amazonaws.com"],
 *         }],
 *         actions: ["s3:PutObject"],
 *         resources: [pulumi.interpolate`${exampleBucket.arn}/*`],
 *     }],
 * });
 * const exampleBucketPolicy = new aws.s3.BucketPolicy("example", {
 *     bucket: exampleBucket.id,
 *     policy: example.apply(example => example.json),
 * });
 * const exampleSessionLogger = new aws.workspacesweb.SessionLogger("example", {
 *     displayName: "example",
 *     eventFilter: {
 *         all: {}[0],
 *     },
 *     logConfiguration: {
 *         s3: {
 *             bucket: exampleBucket.id,
 *             folderStructure: "Flat",
 *             logFileFormat: "Json",
 *         },
 *     },
 * }, {
 *     dependsOn: [exampleBucketPolicy],
 * });
 * const exampleSessionLoggerAssociation = new aws.workspacesweb.SessionLoggerAssociation("example", {
 *     portalArn: examplePortal.portalArn,
 *     sessionLoggerArn: exampleSessionLogger.sessionLoggerArn,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import WorkSpaces Web Session Logger Association using the `session_logger_arn,portal_arn`. For example:
 *
 * ```sh
 * $ pulumi import aws:workspacesweb/sessionLoggerAssociation:SessionLoggerAssociation example arn:aws:workspaces-web:us-west-2:123456789012:sessionLogger/session_logger-id-12345678,arn:aws:workspaces-web:us-west-2:123456789012:portal/portal-id-12345678
 * ```
 */
export declare class SessionLoggerAssociation extends pulumi.CustomResource {
    /**
     * Get an existing SessionLoggerAssociation resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: SessionLoggerAssociationState, opts?: pulumi.CustomResourceOptions): SessionLoggerAssociation;
    /**
     * Returns true if the given object is an instance of SessionLoggerAssociation.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is SessionLoggerAssociation;
    /**
     * ARN of the web portal.
     */
    readonly portalArn: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * ARN of the session logger.
     *
     * The following arguments are optional:
     */
    readonly sessionLoggerArn: pulumi.Output<string>;
    /**
     * Create a SessionLoggerAssociation resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: SessionLoggerAssociationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering SessionLoggerAssociation resources.
 */
export interface SessionLoggerAssociationState {
    /**
     * ARN of the web portal.
     */
    portalArn?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ARN of the session logger.
     *
     * The following arguments are optional:
     */
    sessionLoggerArn?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a SessionLoggerAssociation resource.
 */
export interface SessionLoggerAssociationArgs {
    /**
     * ARN of the web portal.
     */
    portalArn: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ARN of the session logger.
     *
     * The following arguments are optional:
     */
    sessionLoggerArn: pulumi.Input<string>;
}
