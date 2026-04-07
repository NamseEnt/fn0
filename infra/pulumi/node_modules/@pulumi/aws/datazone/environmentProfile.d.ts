import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS DataZone Environment Profile.
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
 *     name: "example-name",
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
 *         name: "example-name",
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
 * const testDomain = new aws.datazone.Domain("test", {
 *     name: "example-name",
 *     domainExecutionRole: domainExecutionRole.arn,
 * });
 * const testSecurityGroup = new aws.ec2.SecurityGroup("test", {name: "example-name"});
 * const testProject = new aws.datazone.Project("test", {
 *     domainIdentifier: testDomain.id,
 *     glossaryTerms: ["2N8w6XJCwZf"],
 *     name: "example-name",
 *     description: "desc",
 *     skipDeletionCheck: true,
 * });
 * const test = aws.getCallerIdentity({});
 * const testGetRegion = aws.getRegion({});
 * const testGetEnvironmentBlueprint = aws.datazone.getEnvironmentBlueprintOutput({
 *     domainId: testDomain.id,
 *     name: "DefaultDataLake",
 *     managed: true,
 * });
 * const testEnvironmentBlueprintConfiguration = new aws.datazone.EnvironmentBlueprintConfiguration("test", {
 *     domainId: testDomain.id,
 *     environmentBlueprintId: testGetEnvironmentBlueprint.apply(testGetEnvironmentBlueprint => testGetEnvironmentBlueprint.id),
 *     provisioningRoleArn: domainExecutionRole.arn,
 *     enabledRegions: [testGetRegion.then(testGetRegion => testGetRegion.name)],
 * });
 * const testEnvironmentProfile = new aws.datazone.EnvironmentProfile("test", {
 *     awsAccountId: test.then(test => test.accountId),
 *     awsAccountRegion: testGetRegion.then(testGetRegion => testGetRegion.name),
 *     description: "description",
 *     environmentBlueprintIdentifier: testGetEnvironmentBlueprint.apply(testGetEnvironmentBlueprint => testGetEnvironmentBlueprint.id),
 *     name: "example-name",
 *     projectIdentifier: testProject.id,
 *     domainIdentifier: testDomain.id,
 *     userParameters: [{
 *         name: "consumerGlueDbName",
 *         value: "value",
 *     }],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import DataZone Environment Profile using a comma-delimited string combining `id` and `domain_identifier`. For example:
 *
 * ```sh
 * $ pulumi import aws:datazone/environmentProfile:EnvironmentProfile example environment_profile-id-12345678,domain-id-12345678
 * ```
 */
export declare class EnvironmentProfile extends pulumi.CustomResource {
    /**
     * Get an existing EnvironmentProfile resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: EnvironmentProfileState, opts?: pulumi.CustomResourceOptions): EnvironmentProfile;
    /**
     * Returns true if the given object is an instance of EnvironmentProfile.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is EnvironmentProfile;
    /**
     * Id of the AWS account being used.
     */
    readonly awsAccountId: pulumi.Output<string>;
    /**
     * Desired region for environment profile.
     */
    readonly awsAccountRegion: pulumi.Output<string>;
    /**
     * Creation time of environment profile.
     */
    readonly createdAt: pulumi.Output<string>;
    /**
     * Creator of environment profile.
     */
    readonly createdBy: pulumi.Output<string>;
    /**
     * Description of environment profile.
     */
    readonly description: pulumi.Output<string>;
    /**
     * Domain Identifier for environment profile.
     */
    readonly domainIdentifier: pulumi.Output<string>;
    /**
     * ID of the blueprint which the environment will be created with.
     */
    readonly environmentBlueprintIdentifier: pulumi.Output<string>;
    /**
     * Name of the environment profile.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Project identifier for environment profile.
     */
    readonly projectIdentifier: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Time of last update to environment profile.
     */
    readonly updatedAt: pulumi.Output<string>;
    /**
     * Array of user parameters of the environment profile with the following attributes:
     */
    readonly userParameters: pulumi.Output<outputs.datazone.EnvironmentProfileUserParameter[] | undefined>;
    /**
     * Create a EnvironmentProfile resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: EnvironmentProfileArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering EnvironmentProfile resources.
 */
export interface EnvironmentProfileState {
    /**
     * Id of the AWS account being used.
     */
    awsAccountId?: pulumi.Input<string>;
    /**
     * Desired region for environment profile.
     */
    awsAccountRegion?: pulumi.Input<string>;
    /**
     * Creation time of environment profile.
     */
    createdAt?: pulumi.Input<string>;
    /**
     * Creator of environment profile.
     */
    createdBy?: pulumi.Input<string>;
    /**
     * Description of environment profile.
     */
    description?: pulumi.Input<string>;
    /**
     * Domain Identifier for environment profile.
     */
    domainIdentifier?: pulumi.Input<string>;
    /**
     * ID of the blueprint which the environment will be created with.
     */
    environmentBlueprintIdentifier?: pulumi.Input<string>;
    /**
     * Name of the environment profile.
     */
    name?: pulumi.Input<string>;
    /**
     * Project identifier for environment profile.
     */
    projectIdentifier?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Time of last update to environment profile.
     */
    updatedAt?: pulumi.Input<string>;
    /**
     * Array of user parameters of the environment profile with the following attributes:
     */
    userParameters?: pulumi.Input<pulumi.Input<inputs.datazone.EnvironmentProfileUserParameter>[]>;
}
/**
 * The set of arguments for constructing a EnvironmentProfile resource.
 */
export interface EnvironmentProfileArgs {
    /**
     * Id of the AWS account being used.
     */
    awsAccountId?: pulumi.Input<string>;
    /**
     * Desired region for environment profile.
     */
    awsAccountRegion: pulumi.Input<string>;
    /**
     * Description of environment profile.
     */
    description?: pulumi.Input<string>;
    /**
     * Domain Identifier for environment profile.
     */
    domainIdentifier: pulumi.Input<string>;
    /**
     * ID of the blueprint which the environment will be created with.
     */
    environmentBlueprintIdentifier: pulumi.Input<string>;
    /**
     * Name of the environment profile.
     */
    name?: pulumi.Input<string>;
    /**
     * Project identifier for environment profile.
     */
    projectIdentifier: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Array of user parameters of the environment profile with the following attributes:
     */
    userParameters?: pulumi.Input<pulumi.Input<inputs.datazone.EnvironmentProfileUserParameter>[]>;
}
