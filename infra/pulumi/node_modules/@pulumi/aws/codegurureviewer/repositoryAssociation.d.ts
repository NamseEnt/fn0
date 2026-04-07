import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS CodeGuru Reviewer Repository Association.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.kms.Key("example", {});
 * const exampleRepository = new aws.codecommit.Repository("example", {repositoryName: "example-repo"});
 * const exampleRepositoryAssociation = new aws.codegurureviewer.RepositoryAssociation("example", {
 *     repository: {
 *         codecommit: {
 *             name: exampleRepository.repositoryName,
 *         },
 *     },
 *     kmsKeyDetails: {
 *         encryptionOption: "CUSTOMER_MANAGED_CMK",
 *         kmsKeyId: example.keyId,
 *     },
 * });
 * ```
 */
export declare class RepositoryAssociation extends pulumi.CustomResource {
    /**
     * Get an existing RepositoryAssociation resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: RepositoryAssociationState, opts?: pulumi.CustomResourceOptions): RepositoryAssociation;
    /**
     * Returns true if the given object is an instance of RepositoryAssociation.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is RepositoryAssociation;
    /**
     * The Amazon Resource Name (ARN) identifying the repository association.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The ID of the repository association.
     */
    readonly associationId: pulumi.Output<string>;
    /**
     * The Amazon Resource Name (ARN) of an AWS CodeStar Connections connection.
     */
    readonly connectionArn: pulumi.Output<string>;
    /**
     * An object describing the KMS key to asssociate. Block is documented below.
     */
    readonly kmsKeyDetails: pulumi.Output<outputs.codegurureviewer.RepositoryAssociationKmsKeyDetails | undefined>;
    /**
     * The name of the repository.
     */
    readonly name: pulumi.Output<string>;
    /**
     * The owner of the repository.
     */
    readonly owner: pulumi.Output<string>;
    /**
     * The provider type of the repository association.
     */
    readonly providerType: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * An object describing the repository to associate. Valid values: `bitbucket`, `codecommit`, `githubEnterpriseServer`, or `s3Bucket`. Block is documented below. Note: for repositories that leverage CodeStar connections (ex. `bitbucket`, `githubEnterpriseServer`) the connection must be in `Available` status prior to creating this resource.
     *
     * The following arguments are optional:
     */
    readonly repository: pulumi.Output<outputs.codegurureviewer.RepositoryAssociationRepository>;
    readonly s3RepositoryDetails: pulumi.Output<outputs.codegurureviewer.RepositoryAssociationS3RepositoryDetail[]>;
    /**
     * The state of the repository association.
     */
    readonly state: pulumi.Output<string>;
    /**
     * A description of why the repository association is in the current state.
     */
    readonly stateReason: pulumi.Output<string>;
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * Create a RepositoryAssociation resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: RepositoryAssociationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering RepositoryAssociation resources.
 */
export interface RepositoryAssociationState {
    /**
     * The Amazon Resource Name (ARN) identifying the repository association.
     */
    arn?: pulumi.Input<string>;
    /**
     * The ID of the repository association.
     */
    associationId?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) of an AWS CodeStar Connections connection.
     */
    connectionArn?: pulumi.Input<string>;
    /**
     * An object describing the KMS key to asssociate. Block is documented below.
     */
    kmsKeyDetails?: pulumi.Input<inputs.codegurureviewer.RepositoryAssociationKmsKeyDetails>;
    /**
     * The name of the repository.
     */
    name?: pulumi.Input<string>;
    /**
     * The owner of the repository.
     */
    owner?: pulumi.Input<string>;
    /**
     * The provider type of the repository association.
     */
    providerType?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * An object describing the repository to associate. Valid values: `bitbucket`, `codecommit`, `githubEnterpriseServer`, or `s3Bucket`. Block is documented below. Note: for repositories that leverage CodeStar connections (ex. `bitbucket`, `githubEnterpriseServer`) the connection must be in `Available` status prior to creating this resource.
     *
     * The following arguments are optional:
     */
    repository?: pulumi.Input<inputs.codegurureviewer.RepositoryAssociationRepository>;
    s3RepositoryDetails?: pulumi.Input<pulumi.Input<inputs.codegurureviewer.RepositoryAssociationS3RepositoryDetail>[]>;
    /**
     * The state of the repository association.
     */
    state?: pulumi.Input<string>;
    /**
     * A description of why the repository association is in the current state.
     */
    stateReason?: pulumi.Input<string>;
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
/**
 * The set of arguments for constructing a RepositoryAssociation resource.
 */
export interface RepositoryAssociationArgs {
    /**
     * An object describing the KMS key to asssociate. Block is documented below.
     */
    kmsKeyDetails?: pulumi.Input<inputs.codegurureviewer.RepositoryAssociationKmsKeyDetails>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * An object describing the repository to associate. Valid values: `bitbucket`, `codecommit`, `githubEnterpriseServer`, or `s3Bucket`. Block is documented below. Note: for repositories that leverage CodeStar connections (ex. `bitbucket`, `githubEnterpriseServer`) the connection must be in `Available` status prior to creating this resource.
     *
     * The following arguments are optional:
     */
    repository: pulumi.Input<inputs.codegurureviewer.RepositoryAssociationRepository>;
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
