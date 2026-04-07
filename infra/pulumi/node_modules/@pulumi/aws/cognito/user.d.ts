import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a Cognito User Resource.
 *
 * ## Example Usage
 *
 * ### Basic configuration
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.cognito.UserPool("example", {name: "MyExamplePool"});
 * const exampleUser = new aws.cognito.User("example", {
 *     userPoolId: example.id,
 *     username: "example",
 * });
 * ```
 *
 * ### Setting user attributes
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.cognito.UserPool("example", {
 *     name: "mypool",
 *     schemas: [
 *         {
 *             name: "example",
 *             attributeDataType: "Boolean",
 *             mutable: false,
 *             required: false,
 *             developerOnlyAttribute: false,
 *         },
 *         {
 *             name: "foo",
 *             attributeDataType: "String",
 *             mutable: false,
 *             required: false,
 *             developerOnlyAttribute: false,
 *             stringAttributeConstraints: {},
 *         },
 *     ],
 * });
 * const exampleUser = new aws.cognito.User("example", {
 *     userPoolId: example.id,
 *     username: "example",
 *     attributes: {
 *         example: "true",
 *         foo: "bar",
 *         email: "no-reply@example.com",
 *         email_verified: "true",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Cognito User using the `user_pool_id`/`name` attributes concatenated. For example:
 *
 * ```sh
 * $ pulumi import aws:cognito/user:User user us-east-1_vG78M4goG/user
 * ```
 */
export declare class User extends pulumi.CustomResource {
    /**
     * Get an existing User resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: UserState, opts?: pulumi.CustomResourceOptions): User;
    /**
     * Returns true if the given object is an instance of User.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is User;
    /**
     * A map that contains user attributes and attribute values to be set for the user.
     */
    readonly attributes: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * A map of custom key-value pairs that you can provide as input for any custom workflows that user creation triggers. Amazon Cognito does not store the `clientMetadata` value. This data is available only to Lambda triggers that are assigned to a user pool to support custom workflows. If your user pool configuration does not include triggers, the ClientMetadata parameter serves no purpose. For more information, see [Customizing User Pool Workflows with Lambda Triggers](https://docs.aws.amazon.com/cognito/latest/developerguide/cognito-user-identity-pools-working-with-aws-lambda-triggers.html).
     */
    readonly clientMetadata: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    readonly creationDate: pulumi.Output<string>;
    /**
     * A list of mediums to the welcome message will be sent through. Allowed values are `EMAIL` and `SMS`. If it's provided, make sure you have also specified `email` attribute for the `EMAIL` medium and `phoneNumber` for the `SMS`. More than one value can be specified. Amazon Cognito does not store the `desiredDeliveryMediums` value. Defaults to `["SMS"]`.
     */
    readonly desiredDeliveryMediums: pulumi.Output<string[] | undefined>;
    /**
     * Specifies whether the user should be enabled after creation. The welcome message will be sent regardless of the `enabled` value. The behavior can be changed with `messageAction` argument. Defaults to `true`.
     */
    readonly enabled: pulumi.Output<boolean | undefined>;
    /**
     * If this parameter is set to True and the `phoneNumber` or `email` address specified in the `attributes` parameter already exists as an alias with a different user, Amazon Cognito will migrate the alias from the previous user to the newly created user. The previous user will no longer be able to log in using that alias. Amazon Cognito does not store the `forceAliasCreation` value. Defaults to `false`.
     */
    readonly forceAliasCreation: pulumi.Output<boolean | undefined>;
    readonly lastModifiedDate: pulumi.Output<string>;
    /**
     * Set to `RESEND` to resend the invitation message to a user that already exists and reset the expiration limit on the user's account. Set to `SUPPRESS` to suppress sending the message. Only one value can be specified. Amazon Cognito does not store the `messageAction` value.
     */
    readonly messageAction: pulumi.Output<string | undefined>;
    readonly mfaSettingLists: pulumi.Output<string[]>;
    /**
     * The user's permanent password. This password must conform to the password policy specified by user pool the user belongs to. The welcome message always contains only `temporaryPassword` value. You can suppress sending the welcome message with the `messageAction` argument. Amazon Cognito does not store the `password` value. Conflicts with `temporaryPassword`.
     */
    readonly password: pulumi.Output<string | undefined>;
    readonly preferredMfaSetting: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * current user status.
     */
    readonly status: pulumi.Output<string>;
    /**
     * unique user id that is never reassignable to another user.
     */
    readonly sub: pulumi.Output<string>;
    /**
     * The user's temporary password. Conflicts with `password`.
     */
    readonly temporaryPassword: pulumi.Output<string | undefined>;
    /**
     * The user pool ID for the user pool where the user will be created.
     */
    readonly userPoolId: pulumi.Output<string>;
    /**
     * The username for the user. Must be unique within the user pool. Must be a UTF-8 string between 1 and 128 characters. After the user is created, the username cannot be changed.
     *
     * The following arguments are optional:
     */
    readonly username: pulumi.Output<string>;
    /**
     * The user's validation data. This is an array of name-value pairs that contain user attributes and attribute values that you can use for custom validation, such as restricting the types of user accounts that can be registered. Amazon Cognito does not store the `validationData` value. For more information, see [Customizing User Pool Workflows with Lambda Triggers](https://docs.aws.amazon.com/cognito/latest/developerguide/cognito-user-identity-pools-working-with-aws-lambda-triggers.html).
     *
     * > **NOTE:** Clearing `password` or `temporaryPassword` does not reset user's password in Cognito.
     */
    readonly validationData: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * Create a User resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: UserArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering User resources.
 */
export interface UserState {
    /**
     * A map that contains user attributes and attribute values to be set for the user.
     */
    attributes?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * A map of custom key-value pairs that you can provide as input for any custom workflows that user creation triggers. Amazon Cognito does not store the `clientMetadata` value. This data is available only to Lambda triggers that are assigned to a user pool to support custom workflows. If your user pool configuration does not include triggers, the ClientMetadata parameter serves no purpose. For more information, see [Customizing User Pool Workflows with Lambda Triggers](https://docs.aws.amazon.com/cognito/latest/developerguide/cognito-user-identity-pools-working-with-aws-lambda-triggers.html).
     */
    clientMetadata?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    creationDate?: pulumi.Input<string>;
    /**
     * A list of mediums to the welcome message will be sent through. Allowed values are `EMAIL` and `SMS`. If it's provided, make sure you have also specified `email` attribute for the `EMAIL` medium and `phoneNumber` for the `SMS`. More than one value can be specified. Amazon Cognito does not store the `desiredDeliveryMediums` value. Defaults to `["SMS"]`.
     */
    desiredDeliveryMediums?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Specifies whether the user should be enabled after creation. The welcome message will be sent regardless of the `enabled` value. The behavior can be changed with `messageAction` argument. Defaults to `true`.
     */
    enabled?: pulumi.Input<boolean>;
    /**
     * If this parameter is set to True and the `phoneNumber` or `email` address specified in the `attributes` parameter already exists as an alias with a different user, Amazon Cognito will migrate the alias from the previous user to the newly created user. The previous user will no longer be able to log in using that alias. Amazon Cognito does not store the `forceAliasCreation` value. Defaults to `false`.
     */
    forceAliasCreation?: pulumi.Input<boolean>;
    lastModifiedDate?: pulumi.Input<string>;
    /**
     * Set to `RESEND` to resend the invitation message to a user that already exists and reset the expiration limit on the user's account. Set to `SUPPRESS` to suppress sending the message. Only one value can be specified. Amazon Cognito does not store the `messageAction` value.
     */
    messageAction?: pulumi.Input<string>;
    mfaSettingLists?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The user's permanent password. This password must conform to the password policy specified by user pool the user belongs to. The welcome message always contains only `temporaryPassword` value. You can suppress sending the welcome message with the `messageAction` argument. Amazon Cognito does not store the `password` value. Conflicts with `temporaryPassword`.
     */
    password?: pulumi.Input<string>;
    preferredMfaSetting?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * current user status.
     */
    status?: pulumi.Input<string>;
    /**
     * unique user id that is never reassignable to another user.
     */
    sub?: pulumi.Input<string>;
    /**
     * The user's temporary password. Conflicts with `password`.
     */
    temporaryPassword?: pulumi.Input<string>;
    /**
     * The user pool ID for the user pool where the user will be created.
     */
    userPoolId?: pulumi.Input<string>;
    /**
     * The username for the user. Must be unique within the user pool. Must be a UTF-8 string between 1 and 128 characters. After the user is created, the username cannot be changed.
     *
     * The following arguments are optional:
     */
    username?: pulumi.Input<string>;
    /**
     * The user's validation data. This is an array of name-value pairs that contain user attributes and attribute values that you can use for custom validation, such as restricting the types of user accounts that can be registered. Amazon Cognito does not store the `validationData` value. For more information, see [Customizing User Pool Workflows with Lambda Triggers](https://docs.aws.amazon.com/cognito/latest/developerguide/cognito-user-identity-pools-working-with-aws-lambda-triggers.html).
     *
     * > **NOTE:** Clearing `password` or `temporaryPassword` does not reset user's password in Cognito.
     */
    validationData?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
/**
 * The set of arguments for constructing a User resource.
 */
export interface UserArgs {
    /**
     * A map that contains user attributes and attribute values to be set for the user.
     */
    attributes?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * A map of custom key-value pairs that you can provide as input for any custom workflows that user creation triggers. Amazon Cognito does not store the `clientMetadata` value. This data is available only to Lambda triggers that are assigned to a user pool to support custom workflows. If your user pool configuration does not include triggers, the ClientMetadata parameter serves no purpose. For more information, see [Customizing User Pool Workflows with Lambda Triggers](https://docs.aws.amazon.com/cognito/latest/developerguide/cognito-user-identity-pools-working-with-aws-lambda-triggers.html).
     */
    clientMetadata?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * A list of mediums to the welcome message will be sent through. Allowed values are `EMAIL` and `SMS`. If it's provided, make sure you have also specified `email` attribute for the `EMAIL` medium and `phoneNumber` for the `SMS`. More than one value can be specified. Amazon Cognito does not store the `desiredDeliveryMediums` value. Defaults to `["SMS"]`.
     */
    desiredDeliveryMediums?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Specifies whether the user should be enabled after creation. The welcome message will be sent regardless of the `enabled` value. The behavior can be changed with `messageAction` argument. Defaults to `true`.
     */
    enabled?: pulumi.Input<boolean>;
    /**
     * If this parameter is set to True and the `phoneNumber` or `email` address specified in the `attributes` parameter already exists as an alias with a different user, Amazon Cognito will migrate the alias from the previous user to the newly created user. The previous user will no longer be able to log in using that alias. Amazon Cognito does not store the `forceAliasCreation` value. Defaults to `false`.
     */
    forceAliasCreation?: pulumi.Input<boolean>;
    /**
     * Set to `RESEND` to resend the invitation message to a user that already exists and reset the expiration limit on the user's account. Set to `SUPPRESS` to suppress sending the message. Only one value can be specified. Amazon Cognito does not store the `messageAction` value.
     */
    messageAction?: pulumi.Input<string>;
    /**
     * The user's permanent password. This password must conform to the password policy specified by user pool the user belongs to. The welcome message always contains only `temporaryPassword` value. You can suppress sending the welcome message with the `messageAction` argument. Amazon Cognito does not store the `password` value. Conflicts with `temporaryPassword`.
     */
    password?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The user's temporary password. Conflicts with `password`.
     */
    temporaryPassword?: pulumi.Input<string>;
    /**
     * The user pool ID for the user pool where the user will be created.
     */
    userPoolId: pulumi.Input<string>;
    /**
     * The username for the user. Must be unique within the user pool. Must be a UTF-8 string between 1 and 128 characters. After the user is created, the username cannot be changed.
     *
     * The following arguments are optional:
     */
    username: pulumi.Input<string>;
    /**
     * The user's validation data. This is an array of name-value pairs that contain user attributes and attribute values that you can use for custom validation, such as restricting the types of user accounts that can be registered. Amazon Cognito does not store the `validationData` value. For more information, see [Customizing User Pool Workflows with Lambda Triggers](https://docs.aws.amazon.com/cognito/latest/developerguide/cognito-user-identity-pools-working-with-aws-lambda-triggers.html).
     *
     * > **NOTE:** Clearing `password` or `temporaryPassword` does not reset user's password in Cognito.
     */
    validationData?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
