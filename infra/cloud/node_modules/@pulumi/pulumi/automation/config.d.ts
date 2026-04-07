/**
 * An input to/output from a `pulumi config` command.
 */
export interface ConfigValue {
    /**
     * The underlying configuration value.
     */
    value: string;
    /**
     * True if and only if this configuration value is a secret.
     */
    secret?: boolean;
}
/**
 * A map of configuration values.
 */
export declare type ConfigMap = {
    [key: string]: ConfigValue;
};
