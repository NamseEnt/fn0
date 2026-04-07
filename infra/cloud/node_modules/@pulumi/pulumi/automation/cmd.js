"use strict";
// Copyright 2016-2020, Pulumi Corporation.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
var __importStar = (this && this.__importStar) || function (mod) {
    if (mod && mod.__esModule) return mod;
    var result = {};
    if (mod != null) for (var k in mod) if (Object.hasOwnProperty.call(mod, k)) result[k] = mod[k];
    result["default"] = mod;
    return result;
};
Object.defineProperty(exports, "__esModule", { value: true });
const execa_1 = __importDefault(require("execa"));
const fs = __importStar(require("fs"));
const got_1 = __importDefault(require("got"));
const os = __importStar(require("os"));
const path = __importStar(require("path"));
const semver = __importStar(require("semver"));
const tmp = __importStar(require("tmp"));
const version_1 = require("../version");
const minimumVersion_1 = require("./minimumVersion");
const errors_1 = require("./errors");
const SKIP_VERSION_CHECK_VAR = "PULUMI_AUTOMATION_API_SKIP_VERSION_CHECK";
/**
 * @internal
 */
class CommandResult {
    constructor(stdout, stderr, code, err) {
        this.stdout = stdout;
        this.stderr = stderr;
        this.code = code;
        this.err = err;
    }
    toString() {
        let errStr = "";
        if (this.err) {
            errStr = this.err.toString();
        }
        return `code: ${this.code}\n stdout: ${this.stdout}\n stderr: ${this.stderr}\n err?: ${errStr}\n`;
    }
}
exports.CommandResult = CommandResult;
class PulumiCommand {
    constructor(command, version) {
        this.command = command;
        this.version = version;
    }
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
    static async get(opts) {
        const command = opts?.root ? path.resolve(path.join(opts.root, "bin/pulumi")) : "pulumi";
        const { stdout } = await exec(command, ["version"], undefined, { PULUMI_SKIP_UPDATE_CHECK: "true" });
        const skipVersionCheck = !!opts?.skipVersionCheck || !!process.env[SKIP_VERSION_CHECK_VAR];
        let min = minimumVersion_1.minimumVersion;
        if (opts?.version && semver.gt(opts.version, minimumVersion_1.minimumVersion)) {
            min = opts.version;
        }
        const version = parseAndValidatePulumiVersion(min, stdout.trim(), skipVersionCheck);
        return new PulumiCommand(command, version);
    }
    /**
     * Installs the Pulumi CLI.
     */
    static async install(opts) {
        const optsWithDefaults = withDefaults(opts);
        try {
            return await PulumiCommand.get(optsWithDefaults);
        }
        catch (err) {
            // ignore
        }
        if (process.platform === "win32") {
            await PulumiCommand.installWindows(optsWithDefaults);
        }
        else {
            await PulumiCommand.installPosix(optsWithDefaults);
        }
        return await PulumiCommand.get(optsWithDefaults);
    }
    static async installWindows(opts) {
        const response = await got_1.default("https://get.pulumi.com/install.ps1");
        const script = await writeTempFile(response.body, { extension: ".ps1" });
        try {
            const command = process.env.SystemRoot
                ? path.join(process.env.SystemRoot, "System32", "WindowsPowerShell", "v1.0", "powershell.exe")
                : "powershell.exe";
            const args = [
                "-NoProfile",
                "-InputFormat",
                "None",
                "-ExecutionPolicy",
                "Bypass",
                "-File",
                script.path,
                "-NoEditPath",
                "-InstallRoot",
                opts.root,
                "-Version",
                `${opts.version}`,
            ];
            await exec(command, args);
        }
        finally {
            script.cleanup();
        }
    }
    static async installPosix(opts) {
        const response = await got_1.default("https://get.pulumi.com/install.sh");
        const script = await writeTempFile(response.body);
        try {
            const args = [script.path, "--no-edit-path", "--install-root", opts.root, "--version", `${opts.version}`];
            await exec("/bin/sh", args);
        }
        finally {
            script.cleanup();
        }
    }
    /** @internal */
    run(args, cwd, additionalEnv, onOutput, onError, signal) {
        // all commands should be run in non-interactive mode.
        // this causes commands to fail rather than prompting for input (and thus hanging indefinitely)
        if (!args.includes("--non-interactive")) {
            args.push("--non-interactive");
        }
        const env = { ...additionalEnv };
        // Prepend the folder where the CLI is installed to the path to ensure
        // we pickup the matching bundled plugins.
        if (path.isAbsolute(this.command)) {
            const pulumiBin = path.dirname(this.command);
            const sep = os.platform() === "win32" ? ";" : ":";
            const envPath = pulumiBin + sep + (additionalEnv["PATH"] || process.env.PATH);
            env["PATH"] = envPath;
        }
        env["PULUMI_AUTOMATION_API"] = "true";
        return exec(this.command, args, cwd, env, onOutput, onError, signal);
    }
}
exports.PulumiCommand = PulumiCommand;
async function exec(command, args, cwd, additionalEnv, onOutput, onError, signal) {
    const unknownErrCode = -2;
    const env = additionalEnv ? { ...additionalEnv } : undefined;
    try {
        const proc = execa_1.default(command, args, { env, cwd });
        if (onError && proc.stderr) {
            proc.stderr.on("data", (data) => {
                if (data?.toString) {
                    data = data.toString();
                }
                onError(data);
            });
        }
        if (onOutput && proc.stdout) {
            proc.stdout.on("data", (data) => {
                if (data?.toString) {
                    data = data.toString();
                }
                onOutput(data);
            });
        }
        if (signal) {
            signal.addEventListener("abort", () => {
                proc.kill("SIGINT", { forceKillAfterTimeout: false });
            });
        }
        const { stdout, stderr, exitCode } = await proc;
        const commandResult = new CommandResult(stdout, stderr, exitCode);
        if (exitCode !== 0) {
            throw errors_1.createCommandError(commandResult);
        }
        return commandResult;
    }
    catch (err) {
        const error = err;
        throw errors_1.createCommandError(new CommandResult("", error.message, unknownErrCode, error));
    }
}
function withDefaults(opts) {
    let version = opts?.version;
    if (!version) {
        version = new semver.SemVer(version_1.version);
    }
    let root = opts?.root;
    if (!root) {
        root = path.join(os.homedir(), ".pulumi", "versions", `${version}`);
    }
    const skipVersionCheck = opts?.skipVersionCheck !== undefined ? opts.skipVersionCheck : false;
    return { version, root, skipVersionCheck };
}
function writeTempFile(contents, options) {
    return new Promise((resolve, reject) => {
        tmp.file({
            // Powershell requires a `.ps1` extension.
            postfix: options?.extension,
            // Powershell won't execute the script if the file descriptor is open.
            discardDescriptor: true,
        }, (tmpErr, tmpPath, _fd, cleanup) => {
            if (tmpErr) {
                reject(tmpErr);
            }
            else {
                fs.writeFile(tmpPath, contents, (writeErr) => {
                    if (writeErr) {
                        cleanup();
                        reject(writeErr);
                    }
                    else {
                        resolve({ path: tmpPath, cleanup });
                    }
                });
            }
        });
    });
}
/**
 * @internal
 * Throws an error if the Pulumi CLI version is not valid.
 *
 * @param minVersion The minimum acceptable version of the Pulumi CLI.
 * @param currentVersion The currently known version. `null` indicates that the current version is unknown.
 * @param optOut If the user has opted out of the version check.
 */
function parseAndValidatePulumiVersion(minVersion, currentVersion, optOut) {
    const version = semver.parse(currentVersion);
    if (optOut) {
        return version;
    }
    if (version == null) {
        throw new Error(`Failed to parse Pulumi CLI version. This is probably an internal error. You can override this by setting "${SKIP_VERSION_CHECK_VAR}" to "true".`);
    }
    if (minVersion.major < version.major) {
        throw new Error(`Major version mismatch. You are using Pulumi CLI version ${currentVersion} with Automation SDK v${minVersion.major}. Please update the SDK.`);
    }
    if (minVersion.compare(version) === 1) {
        throw new Error(`Minimum version requirement failed. The minimum CLI version requirement is ${minVersion.toString()}, your current CLI version is ${currentVersion}. Please update the Pulumi CLI.`);
    }
    return version;
}
exports.parseAndValidatePulumiVersion = parseAndValidatePulumiVersion;
//# sourceMappingURL=cmd.js.map