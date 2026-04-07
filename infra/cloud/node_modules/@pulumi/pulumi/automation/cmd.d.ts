import * as semver from "semver";
export interface PulumiCommandOptions {
    /**
     * The version of the CLI to use. Defaults to the CLI version matching the SDK version.
     */
    version?: semver.SemVer;
    /**
     * The directory to install the CLI in or where to look for an existing
     * installation. Defaults to $HOME/.pulumi/versions/$VERSION.
     */
    root?: string;
    /**
     * Skips the minimum CLI version check, see `PULUMI_AUTOMATION_API_SKIP_VERSION_CHECK`.
     */
    skipVersionCheck?: boolean;
}
export declare class PulumiCommand {
    readonly command: string;
    readonly version: semver.SemVer | null;
    private constructor();
    /**
     * Get a new Pulumi instance that uses the installation in `opts.root`.
     * Defaults to using the pulumi binary found in $PATH if no installation
     * root is specified.  If `opts.version` is specified, it validates that
     * the CLI is compatible with the requested version and throws an error if
     * not. This validation can be skipped by setting the environment variable
     * `PULUMI_AUTOMATION_API_SKIP_VERSION_CHECK` or setting
     * `opts.skipVersionCheck` to `true`. Note that the environment variable
     * always takes precedence. If it is set it is not possible to re-enable
     * the validation with `opts.skipVersionCheck`.
     */
    static get(opts?: PulumiCommandOptions): Promise<PulumiCommand>;
    /**
     * Installs the Pulumi CLI.
     */
    static install(opts?: PulumiCommandOptions): Promise<PulumiCommand>;
    private static installWindows;
    private static installPosix;
}
