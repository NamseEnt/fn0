/**
 * A description of a {@link Stack}'s configuration and encryption metadata.
 */
export interface StackSettings {
    secretsProvider?: string;
    encryptedKey?: string;
    encryptionSalt?: string;
    config?: {
        [key: string]: StackSettingsConfigValue;
    };
}
/**
 * A stack configuration entry.
 */
export declare type StackSettingsConfigValue = string | StackSettingsSecureConfigValue | any;
/**
 * A secret stack configuration entry.
 */
export interface StackSettingsSecureConfigValue {
    secure: string;
}
