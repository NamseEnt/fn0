import * as pulumi from "@pulumi/pulumi";
/**
 * Generate a random password.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = aws.secretsmanager.getRandomPassword({
 *     passwordLength: 50,
 *     excludeNumbers: true,
 * });
 * ```
 */
export declare function getRandomPassword(args?: GetRandomPasswordArgs, opts?: pulumi.InvokeOptions): Promise<GetRandomPasswordResult>;
/**
 * A collection of arguments for invoking getRandomPassword.
 */
export interface GetRandomPasswordArgs {
    /**
     * String of the characters that you don't want in the password.
     */
    excludeCharacters?: string;
    /**
     * Specifies whether to exclude lowercase letters from the password.
     */
    excludeLowercase?: boolean;
    /**
     * Specifies whether to exclude numbers from the password.
     */
    excludeNumbers?: boolean;
    /**
     * Specifies whether to exclude the following punctuation characters from the password: ``! " # $ % & ' ( ) * + , - . / : ; < = > ? @ [ \ ] ^ _ ` { | } ~ .``
     */
    excludePunctuation?: boolean;
    /**
     * Specifies whether to exclude uppercase letters from the password.
     */
    excludeUppercase?: boolean;
    /**
     * Specifies whether to include the space character.
     */
    includeSpace?: boolean;
    /**
     * Length of the password.
     */
    passwordLength?: number;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
    /**
     * Specifies whether to include at least one upper and lowercase letter, one number, and one punctuation.
     */
    requireEachIncludedType?: boolean;
}
/**
 * A collection of values returned by getRandomPassword.
 */
export interface GetRandomPasswordResult {
    readonly excludeCharacters?: string;
    readonly excludeLowercase?: boolean;
    readonly excludeNumbers?: boolean;
    readonly excludePunctuation?: boolean;
    readonly excludeUppercase?: boolean;
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly includeSpace?: boolean;
    readonly passwordLength?: number;
    /**
     * Random password.
     */
    readonly randomPassword: string;
    readonly region: string;
    readonly requireEachIncludedType?: boolean;
}
/**
 * Generate a random password.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = aws.secretsmanager.getRandomPassword({
 *     passwordLength: 50,
 *     excludeNumbers: true,
 * });
 * ```
 */
export declare function getRandomPasswordOutput(args?: GetRandomPasswordOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetRandomPasswordResult>;
/**
 * A collection of arguments for invoking getRandomPassword.
 */
export interface GetRandomPasswordOutputArgs {
    /**
     * String of the characters that you don't want in the password.
     */
    excludeCharacters?: pulumi.Input<string>;
    /**
     * Specifies whether to exclude lowercase letters from the password.
     */
    excludeLowercase?: pulumi.Input<boolean>;
    /**
     * Specifies whether to exclude numbers from the password.
     */
    excludeNumbers?: pulumi.Input<boolean>;
    /**
     * Specifies whether to exclude the following punctuation characters from the password: ``! " # $ % & ' ( ) * + , - . / : ; < = > ? @ [ \ ] ^ _ ` { | } ~ .``
     */
    excludePunctuation?: pulumi.Input<boolean>;
    /**
     * Specifies whether to exclude uppercase letters from the password.
     */
    excludeUppercase?: pulumi.Input<boolean>;
    /**
     * Specifies whether to include the space character.
     */
    includeSpace?: pulumi.Input<boolean>;
    /**
     * Length of the password.
     */
    passwordLength?: pulumi.Input<number>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Specifies whether to include at least one upper and lowercase letter, one number, and one punctuation.
     */
    requireEachIncludedType?: pulumi.Input<boolean>;
}
