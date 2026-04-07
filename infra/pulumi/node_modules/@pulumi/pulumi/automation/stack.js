"use strict";
// Copyright 2016-2022, Pulumi Corporation.
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
var __importStar = (this && this.__importStar) || function (mod) {
    if (mod && mod.__esModule) return mod;
    var result = {};
    if (mod != null) for (var k in mod) if (Object.hasOwnProperty.call(mod, k)) result[k] = mod[k];
    result["default"] = mod;
    return result;
};
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const fs = __importStar(require("fs"));
const os = __importStar(require("os"));
const pathlib = __importStar(require("path"));
const readline = __importStar(require("readline"));
const semver = __importStar(require("semver"));
const upath = __importStar(require("upath"));
const grpc = __importStar(require("@grpc/grpc-js"));
const tail_file_1 = __importDefault(require("@logdna/tail-file"));
const log = __importStar(require("../log"));
const errors_1 = require("./errors");
const localWorkspace_1 = require("./localWorkspace");
const server_1 = require("./server");
const empty_pb_1 = require("google-protobuf/google/protobuf/empty_pb");
const eventsrpc = __importStar(require("../proto/events_grpc_pb"));
const langrpc = __importStar(require("../proto/language_grpc_pb"));
const runtime_1 = require("../runtime");
/**
 * {@link Stack} is an isolated, independently configurable instance of a Pulumi
 * program. {@link Stack} exposes methods for the full Pulumi lifecycle
 * (up/preview/refresh/destroy), as well as managing configuration. Multiple
 * {@link Stacks} are commonly used to denote different phases of development
 * (such as development, staging and production) or feature branches (such as
 * feature-x-dev, jane-feature-x-dev).
 *
 * @alpha
 */
class Stack {
    constructor(name, workspace, mode) {
        this.name = name;
        this.workspace = workspace;
        switch (mode) {
            case "create":
                this.ready = workspace.createStack(name);
                return this;
            case "select":
                this.ready = workspace.selectStack(name);
                return this;
            case "createOrSelect":
                this.ready = workspace.selectStack(name).catch((err) => {
                    if (err instanceof errors_1.StackNotFoundError) {
                        return workspace.createStack(name);
                    }
                    throw err;
                });
                return this;
            default:
                throw new Error(`unexpected Stack creation mode: ${mode}`);
        }
    }
    /**
     * Creates a new stack using the given workspace, and stack name.
     * It fails if a stack with that name already exists
     *
     * @param name
     *  The name identifying the Stack.
     * @param workspace
     *  The Workspace the Stack was created from.
     */
    static async create(name, workspace) {
        const stack = new Stack(name, workspace, "create");
        await stack.ready;
        return stack;
    }
    /**
     * Selects stack using the given workspace and stack name. It returns an
     * error if the given stack does not exist.
     *
     * @param name
     *  The name identifying the Stack.
     * @param workspace
     *  The {@link Workspace} the stack will be created from.
     */
    static async select(name, workspace) {
        const stack = new Stack(name, workspace, "select");
        await stack.ready;
        return stack;
    }
    /**
     * Creates a new stack using the given workspace and stack name if the stack
     * does not already exist, or falls back to selecting the existing stack. If
     * the stack does not exist, it will be created and selected.
     *
     * @param name
     *  The name identifying the Stack.
     * @param workspace
     *  The {@link Workspace} the stack will be created from.
     */
    static async createOrSelect(name, workspace) {
        const stack = new Stack(name, workspace, "createOrSelect");
        await stack.ready;
        return stack;
    }
    async setupEventLog(command, onEvent, pulumiVersion) {
        const ver = semver.parse(pulumiVersion) ?? semver.parse("3.0.0");
        if (semver.gt(ver, "3.205.0")) {
            const eventsServer = new grpc.Server({
                ...runtime_1.grpcChannelOptions,
            });
            const eventsService = new EventsServer(onEvent);
            eventsServer.addService(eventsrpc.EventsService, eventsService);
            const port = await new Promise((resolve, reject) => {
                eventsServer.bindAsync(`127.0.0.1:0`, grpc.ServerCredentials.createInsecure(), (err, p) => {
                    if (err) {
                        reject(err);
                    }
                    else {
                        resolve(p);
                    }
                });
            });
            const file = `tcp://127.0.0.1:${port}`;
            return { logFile: file, logPromise: undefined, server: eventsServer };
        }
        else {
            const file = createLogFile(command);
            const logPromise = this.readLines(file, onEvent);
            return { logFile: file, logPromise: logPromise };
        }
    }
    async readLines(logPath, callback) {
        const eventLogTail = new tail_file_1.default(logPath, { startPos: 0, pollFileIntervalMs: 200 }).on("tail_error", (err) => {
            throw err;
        });
        await eventLogTail.start();
        const lineSplitter = readline.createInterface({ input: eventLogTail });
        lineSplitter.on("line", (line) => {
            let event;
            try {
                event = JSON.parse(line);
                callback(event);
            }
            catch (e) {
                log.warn(`Failed to parse engine event
If you're seeing this warning, please comment on https://github.com/pulumi/pulumi/issues/6768 with the event and any
details about your environment.
Event: ${line}\n${e.toString()}`);
            }
        });
        return {
            tail: eventLogTail,
            rl: lineSplitter,
        };
    }
    /**
     * Creates or updates the resources in a stack by executing the program in
     * the {@link Workspace.}
     *
     * @param opts
     *  Options to customize the behavior of the update.
     *
     * @see https://www.pulumi.com/docs/cli/commands/pulumi_up/
     */
    async up(opts) {
        const args = ["up", "--yes", "--skip-preview"];
        let kind = execKind.local;
        let program = this.workspace.program;
        args.push(...this.remoteArgs());
        if (opts) {
            if (opts.program) {
                program = opts.program;
            }
            if (opts.message) {
                args.push("--message", opts.message);
            }
            if (opts.expectNoChanges) {
                args.push("--expect-no-changes");
            }
            if (opts.refresh) {
                args.push("--refresh");
            }
            if (opts.diff) {
                args.push("--diff");
            }
            if (opts.replace) {
                for (const rURN of opts.replace) {
                    args.push("--replace", rURN);
                }
            }
            if (opts.exclude) {
                for (const eURN of opts.exclude) {
                    args.push("--exclude", eURN);
                }
            }
            if (opts.target) {
                for (const tURN of opts.target) {
                    args.push("--target", tURN);
                }
            }
            if (opts.policyPacks) {
                for (const pack of opts.policyPacks) {
                    args.push("--policy-pack", pack);
                }
            }
            if (opts.policyPackConfigs) {
                for (const packConfig of opts.policyPackConfigs) {
                    args.push("--policy-pack-config", packConfig);
                }
            }
            if (opts.excludeDependents) {
                args.push("--exclude-dependents");
            }
            if (opts.targetDependents) {
                args.push("--target-dependents");
            }
            if (opts.parallel) {
                args.push("--parallel", opts.parallel.toString());
            }
            if (opts.userAgent) {
                args.push("--exec-agent", opts.userAgent);
            }
            if (opts.plan) {
                args.push("--plan", opts.plan);
            }
            if (opts.continueOnError) {
                args.push("--continue-on-error");
            }
            if (opts.attachDebugger) {
                args.push("--attach-debugger");
            }
            if (opts.runProgram !== undefined) {
                if (opts.runProgram) {
                    args.push("--run-program=true");
                }
                else {
                    args.push("--run-program=false");
                }
            }
            applyGlobalOpts(opts, args);
        }
        let onExit = (hasError) => {
            return;
        };
        let didError = false;
        if (program) {
            kind = execKind.inline;
            const server = new grpc.Server({
                ...runtime_1.grpcChannelOptions,
            });
            const languageServer = new server_1.LanguageServer(program);
            server.addService(langrpc.LanguageRuntimeService, languageServer);
            const port = await new Promise((resolve, reject) => {
                server.bindAsync(`127.0.0.1:0`, grpc.ServerCredentials.createInsecure(), (err, p) => {
                    if (err) {
                        reject(err);
                    }
                    else {
                        resolve(p);
                    }
                });
            });
            onExit = (hasError) => {
                languageServer.onPulumiExit(hasError);
                server.forceShutdown();
            };
            args.push(`--client=127.0.0.1:${port}`);
        }
        args.push("--exec-kind", kind);
        let logPromise;
        let logFile;
        let eventsServer;
        // Set up event log tailing
        if (opts?.onEvent) {
            ({
                logFile,
                logPromise,
                server: eventsServer,
            } = await this.setupEventLog("up", opts.onEvent, this.workspace.pulumiVersion));
            args.push("--event-log", logFile);
        }
        let upResult;
        try {
            upResult = await this.runPulumiCmd(args, opts?.onOutput, opts?.onError, opts?.signal);
        }
        catch (e) {
            didError = true;
            throw e;
        }
        finally {
            onExit(didError);
            await cleanUp(logFile, await logPromise, eventsServer);
        }
        // TODO: do this in parallel after this is fixed https://github.com/pulumi/pulumi/issues/6050
        const outputs = await this.outputs();
        // If it's a remote workspace, explicitly set showSecrets to false to prevent attempting to
        // load the project file.
        const summary = await this.info(!this.isRemote && opts?.showSecrets);
        return {
            stdout: upResult.stdout,
            stderr: upResult.stderr,
            summary: summary,
            outputs: outputs,
        };
    }
    /**
     * Performs a dry-run update to a stack, returning pending changes.
     *
     * @param opts Options to customize the behavior of the preview.
     *
     * @see https://www.pulumi.com/docs/cli/commands/pulumi_preview/
     */
    async preview(opts) {
        const args = ["preview"];
        let kind = execKind.local;
        let program = this.workspace.program;
        args.push(...this.remoteArgs());
        if (opts) {
            if (opts.program) {
                program = opts.program;
            }
            if (opts.message) {
                args.push("--message", opts.message);
            }
            if (opts.expectNoChanges) {
                args.push("--expect-no-changes");
            }
            if (opts.refresh) {
                args.push("--refresh");
            }
            if (opts.diff) {
                args.push("--diff");
            }
            if (opts.replace) {
                for (const rURN of opts.replace) {
                    args.push("--replace", rURN);
                }
            }
            if (opts.exclude) {
                for (const eURN of opts.exclude) {
                    args.push("--exclude", eURN);
                }
            }
            if (opts.target) {
                for (const tURN of opts.target) {
                    args.push("--target", tURN);
                }
            }
            if (opts.policyPacks) {
                for (const pack of opts.policyPacks) {
                    args.push("--policy-pack", pack);
                }
            }
            if (opts.policyPackConfigs) {
                for (const packConfig of opts.policyPackConfigs) {
                    args.push("--policy-pack-config", packConfig);
                }
            }
            if (opts.excludeDependents) {
                args.push("--exclude-dependents");
            }
            if (opts.targetDependents) {
                args.push("--target-dependents");
            }
            if (opts.parallel) {
                args.push("--parallel", opts.parallel.toString());
            }
            if (opts.userAgent) {
                args.push("--exec-agent", opts.userAgent);
            }
            if (opts.plan) {
                args.push("--save-plan", opts.plan);
            }
            if (opts.importFile) {
                args.push("--import-file", opts.importFile);
            }
            if (opts.attachDebugger) {
                args.push("--attach-debugger");
            }
            if (opts.runProgram !== undefined) {
                if (opts.runProgram) {
                    args.push("--run-program=true");
                }
                else {
                    args.push("--run-program=false");
                }
            }
            applyGlobalOpts(opts, args);
        }
        let onExit = (hasError) => {
            return;
        };
        let didError = false;
        if (program) {
            kind = execKind.inline;
            const server = new grpc.Server({
                ...runtime_1.grpcChannelOptions,
            });
            const languageServer = new server_1.LanguageServer(program);
            server.addService(langrpc.LanguageRuntimeService, languageServer);
            const port = await new Promise((resolve, reject) => {
                server.bindAsync(`127.0.0.1:0`, grpc.ServerCredentials.createInsecure(), (err, p) => {
                    if (err) {
                        reject(err);
                    }
                    else {
                        resolve(p);
                    }
                });
            });
            onExit = (hasError) => {
                languageServer.onPulumiExit(hasError);
                server.forceShutdown();
            };
            args.push(`--client=127.0.0.1:${port}`);
        }
        args.push("--exec-kind", kind);
        let summaryEvent;
        const onEvent = (event) => {
            if (event.summaryEvent) {
                summaryEvent = event.summaryEvent;
            }
            if (opts?.onEvent) {
                opts.onEvent(event);
            }
        };
        const { logFile, logPromise, server: eventsServer, } = await this.setupEventLog("preview", onEvent, this.workspace.pulumiVersion);
        args.push("--event-log", logFile);
        let previewResult;
        try {
            previewResult = await this.runPulumiCmd(args, opts?.onOutput, opts?.onError, opts?.signal);
        }
        catch (e) {
            didError = true;
            throw e;
        }
        finally {
            onExit(didError);
            await cleanUp(logFile, await logPromise, eventsServer);
        }
        if (!summaryEvent) {
            log.warn("Failed to parse summary event, but preview succeeded. PreviewResult `changeSummary` will be empty.");
        }
        return {
            stdout: previewResult.stdout,
            stderr: previewResult.stderr,
            changeSummary: summaryEvent?.resourceChanges || {},
        };
    }
    /**
     * Check the installed version of the Pulumi CLI supports inline programs for refresh and destroy operations.
     */
    checkInlineSupport() {
        const ver = semver.parse(this.workspace.pulumiVersion) ?? semver.parse("3.0.0");
        // 3.181 added support for --client (https://github.com/pulumi/pulumi/releases/tag/v3.181.0)
        if (semver.lt(ver, "3.181.0")) {
            throw new Error(`destroy with inline programs requires Pulumi version >= 3.181.0`);
        }
    }
    /**
     * Compares the current stack’s resource state with the state known to exist
     * in the actual cloud provider. Any such changes are adopted into the
     * current stack.
     *
     * @param opts
     *  Options to customize the behavior of the refresh.
     */
    async refresh(opts) {
        const args = ["refresh"];
        if (opts?.previewOnly) {
            args.push("--preview-only");
        }
        else {
            args.push("--skip-preview", "--yes");
        }
        args.push(...this.remoteArgs());
        if (opts) {
            if (opts.message) {
                args.push("--message", opts.message);
            }
            if (opts.expectNoChanges) {
                args.push("--expect-no-changes");
            }
            if (opts.clearPendingCreates) {
                args.push("--clear-pending-creates");
            }
            if (opts.exclude) {
                for (const eURN of opts.exclude) {
                    args.push("--exclude", eURN);
                }
            }
            if (opts.excludeDependents) {
                args.push("--exclude-dependents");
            }
            if (opts.target) {
                for (const tURN of opts.target) {
                    args.push("--target", tURN);
                }
            }
            if (opts.targetDependents) {
                args.push("--target-dependents");
            }
            if (opts.parallel) {
                args.push("--parallel", opts.parallel.toString());
            }
            if (opts.userAgent) {
                args.push("--exec-agent", opts.userAgent);
            }
            if (opts.runProgram !== undefined) {
                if (opts.runProgram) {
                    args.push("--run-program=true");
                }
                else {
                    args.push("--run-program=false");
                }
            }
            applyGlobalOpts(opts, args);
        }
        let logPromise;
        let logFile;
        let eventsServer;
        // Set up event log tailing
        if (opts?.onEvent) {
            ({
                logFile,
                logPromise,
                server: eventsServer,
            } = await this.setupEventLog("refresh", opts.onEvent, this.workspace.pulumiVersion));
            args.push("--event-log", logFile);
        }
        let onExit = (hasError) => {
            return;
        };
        let didError = false;
        let kind = execKind.local;
        if (this.workspace.program !== undefined) {
            this.checkInlineSupport();
            kind = execKind.inline;
            const server = new grpc.Server({
                ...runtime_1.grpcChannelOptions,
            });
            const languageServer = new server_1.LanguageServer(this.workspace.program);
            server.addService(langrpc.LanguageRuntimeService, languageServer);
            const port = await new Promise((resolve, reject) => {
                server.bindAsync(`127.0.0.1:0`, grpc.ServerCredentials.createInsecure(), (err, p) => {
                    if (err) {
                        reject(err);
                    }
                    else {
                        resolve(p);
                    }
                });
            });
            onExit = (hasError) => {
                languageServer.onPulumiExit(hasError);
                server.forceShutdown();
            };
            args.push(`--client=127.0.0.1:${port}`);
        }
        args.push("--exec-kind", kind);
        let refResult;
        try {
            refResult = await this.runPulumiCmd(args, opts?.onOutput, opts?.onError, opts?.signal);
        }
        catch (e) {
            didError = true;
            throw e;
        }
        finally {
            onExit(didError);
            await cleanUp(logFile, await logPromise, eventsServer);
        }
        // If it's a remote workspace, explicitly set showSecrets to false to prevent attempting to
        // load the project file.
        const summary = await this.info(!this.isRemote && opts?.showSecrets);
        return {
            stdout: refResult.stdout,
            stderr: refResult.stderr,
            summary: summary,
        };
    }
    /**
     * Performs a dry-run refresh of the stack, returning pending changes.
     *
     * @param opts
     *  Options to customize the behavior of the refresh.
     */
    async previewRefresh(opts) {
        const args = ["refresh", "--preview-only"];
        args.push(...this.remoteArgs());
        if (opts) {
            if (opts.message) {
                args.push("--message", opts.message);
            }
            if (opts.expectNoChanges) {
                args.push("--expect-no-changes");
            }
            if (opts.clearPendingCreates) {
                args.push("--clear-pending-creates");
            }
            if (opts.exclude) {
                for (const eURN of opts.exclude) {
                    args.push("--exclude", eURN);
                }
            }
            if (opts.excludeDependents) {
                args.push("--exclude-dependents");
            }
            if (opts.target) {
                for (const tURN of opts.target) {
                    args.push("--target", tURN);
                }
            }
            if (opts.targetDependents) {
                args.push("--target-dependents");
            }
            if (opts.parallel) {
                args.push("--parallel", opts.parallel.toString());
            }
            if (opts.userAgent) {
                args.push("--exec-agent", opts.userAgent);
            }
            if (opts.runProgram !== undefined) {
                if (opts.runProgram) {
                    args.push("--run-program=true");
                }
                else {
                    args.push("--run-program=false");
                }
            }
            applyGlobalOpts(opts, args);
        }
        args.push("--exec-kind", execKind.local);
        let summaryEvent;
        const onEvent = (event) => {
            if (event.summaryEvent) {
                summaryEvent = event.summaryEvent;
            }
            if (opts?.onEvent) {
                opts.onEvent(event);
            }
        };
        const { logFile, logPromise, server } = await this.setupEventLog("preview-refresh", onEvent, this.workspace.pulumiVersion);
        args.push("--event-log", logFile);
        let previewResult;
        try {
            previewResult = await this.runPulumiCmd(args, opts?.onOutput, opts?.onError, opts?.signal);
        }
        catch (e) {
            throw e;
        }
        finally {
            await cleanUp(logFile, await logPromise, server);
        }
        if (!summaryEvent) {
            log.warn("Failed to parse summary event, but preview succeeded. PreviewResult `changeSummary` will be empty.");
        }
        return {
            stdout: previewResult.stdout,
            stderr: previewResult.stderr,
            changeSummary: summaryEvent?.resourceChanges || {},
        };
    }
    /**
     * Deletes all resources in a stack. By default, this method will leave all
     * history and configuration intact. If `opts.remove` is set, the entire
     * stack and its configuration will also be deleted.
     *
     * @param opts
     *  Options to customize the behavior of the destroy.
     */
    async destroy(opts) {
        const args = ["destroy"];
        if (opts?.previewOnly) {
            args.push("--preview-only");
        }
        else {
            args.push("--yes", "--skip-preview");
        }
        args.push(...this.remoteArgs());
        if (opts) {
            if (opts.message) {
                args.push("--message", opts.message);
            }
            if (opts.exclude) {
                for (const eURN of opts.exclude) {
                    args.push("--exclude", eURN);
                }
            }
            if (opts.target) {
                for (const tURN of opts.target) {
                    args.push("--target", tURN);
                }
            }
            if (opts.excludeDependents) {
                args.push("--exclude-dependents");
            }
            if (opts.targetDependents) {
                args.push("--target-dependents");
            }
            if (opts.excludeProtected) {
                args.push("--exclude-protected");
            }
            if (opts.continueOnError) {
                args.push("--continue-on-error");
            }
            if (opts.parallel) {
                args.push("--parallel", opts.parallel.toString());
            }
            if (opts.userAgent) {
                args.push("--exec-agent", opts.userAgent);
            }
            if (opts.refresh) {
                args.push("--refresh");
            }
            if (opts.runProgram !== undefined) {
                if (opts.runProgram) {
                    args.push("--run-program=true");
                }
                else {
                    args.push("--run-program=false");
                }
            }
            applyGlobalOpts(opts, args);
        }
        let logPromise;
        let logFile;
        let eventsServer;
        // Set up event log tailing
        if (opts?.onEvent) {
            ({
                logFile,
                logPromise,
                server: eventsServer,
            } = await this.setupEventLog("destroy", opts.onEvent, this.workspace.pulumiVersion));
            args.push("--event-log", logFile);
        }
        let onExit = (hasError) => {
            return;
        };
        let didError = false;
        let kind = execKind.local;
        if (this.workspace.program !== undefined) {
            this.checkInlineSupport();
            kind = execKind.inline;
            const server = new grpc.Server({
                ...runtime_1.grpcChannelOptions,
            });
            const languageServer = new server_1.LanguageServer(this.workspace.program);
            server.addService(langrpc.LanguageRuntimeService, languageServer);
            const port = await new Promise((resolve, reject) => {
                server.bindAsync(`127.0.0.1:0`, grpc.ServerCredentials.createInsecure(), (err, p) => {
                    if (err) {
                        reject(err);
                    }
                    else {
                        resolve(p);
                    }
                });
            });
            onExit = (hasError) => {
                languageServer.onPulumiExit(hasError);
                server.forceShutdown();
            };
            args.push(`--client=127.0.0.1:${port}`);
        }
        args.push("--exec-kind", kind);
        let desResult;
        try {
            desResult = await this.runPulumiCmd(args, opts?.onOutput, opts?.onError, opts?.signal);
        }
        catch (e) {
            didError = true;
            throw e;
        }
        finally {
            onExit(didError);
            await cleanUp(logFile, await logPromise, eventsServer);
        }
        // If it's a remote workspace, explicitly set showSecrets to false to prevent attempting to
        // load the project file.
        const summary = await this.info(!this.isRemote && opts?.showSecrets);
        // If `opts.remove` was set, remove the stack now. We take this approach
        // rather than passing `--remove` to `pulumi destroy` because the latter
        // would make it impossible for us to retrieve a summary of the
        // operation above for returning to the caller.
        if (opts?.remove) {
            await this.workspace.removeStack(this.name);
        }
        return {
            stdout: desResult.stdout,
            stderr: desResult.stderr,
            summary: summary,
        };
    }
    /**
     * Performs a dry-run destroy of the stack, returning pending changes.
     *
     * @param opts
     *  Options to customize the behavior of the destroy.
     */
    async previewDestroy(opts) {
        const args = ["destroy", "--preview-only"];
        args.push(...this.remoteArgs());
        if (opts) {
            if (opts.message) {
                args.push("--message", opts.message);
            }
            if (opts.exclude) {
                for (const eURN of opts.exclude) {
                    args.push("--exclude", eURN);
                }
            }
            if (opts.target) {
                for (const tURN of opts.target) {
                    args.push("--target", tURN);
                }
            }
            if (opts.excludeDependents) {
                args.push("--exclude-dependents");
            }
            if (opts.targetDependents) {
                args.push("--target-dependents");
            }
            if (opts.excludeProtected) {
                args.push("--exclude-protected");
            }
            if (opts.continueOnError) {
                args.push("--continue-on-error");
            }
            if (opts.parallel) {
                args.push("--parallel", opts.parallel.toString());
            }
            if (opts.userAgent) {
                args.push("--exec-agent", opts.userAgent);
            }
            if (opts.refresh) {
                args.push("--refresh");
            }
            if (opts.runProgram !== undefined) {
                if (opts.runProgram) {
                    args.push("--run-program=true");
                }
                else {
                    args.push("--run-program=false");
                }
            }
            applyGlobalOpts(opts, args);
        }
        let onExit = (hasError) => {
            return;
        };
        let didError = false;
        let kind = execKind.local;
        if (this.workspace.program !== undefined) {
            this.checkInlineSupport();
            kind = execKind.inline;
            const server = new grpc.Server({
                ...runtime_1.grpcChannelOptions,
            });
            const languageServer = new server_1.LanguageServer(this.workspace.program);
            server.addService(langrpc.LanguageRuntimeService, languageServer);
            const port = await new Promise((resolve, reject) => {
                server.bindAsync(`127.0.0.1:0`, grpc.ServerCredentials.createInsecure(), (err, p) => {
                    if (err) {
                        reject(err);
                    }
                    else {
                        resolve(p);
                    }
                });
            });
            onExit = (hasError) => {
                languageServer.onPulumiExit(hasError);
                server.forceShutdown();
            };
            args.push(`--client=127.0.0.1:${port}`);
        }
        args.push("--exec-kind", kind);
        let summaryEvent;
        const onEvent = (event) => {
            if (event.summaryEvent) {
                summaryEvent = event.summaryEvent;
            }
            if (opts?.onEvent) {
                opts.onEvent(event);
            }
        };
        const { logFile, logPromise, server: eventsServer, } = await this.setupEventLog("preview-destroy", onEvent, this.workspace.pulumiVersion);
        args.push("--event-log", logFile);
        let previewResult;
        try {
            previewResult = await this.runPulumiCmd(args, opts?.onOutput, opts?.onError, opts?.signal);
        }
        catch (e) {
            didError = true;
            throw e;
        }
        finally {
            onExit(didError);
            await cleanUp(logFile, await logPromise, eventsServer);
        }
        if (!summaryEvent) {
            log.warn("Failed to parse summary event, but preview succeeded. PreviewResult `changeSummary` will be empty.");
        }
        return {
            stdout: previewResult.stdout,
            stderr: previewResult.stderr,
            changeSummary: summaryEvent?.resourceChanges || {},
        };
    }
    /**
     * Rename an existing stack
     */
    async rename(options) {
        const args = ["stack", "rename", options.stackName];
        args.push(...this.remoteArgs());
        applyGlobalOpts(options, args);
        const renameResult = await this.runPulumiCmd(args, options?.onOutput, options?.onError, options?.signal);
        if (this.isRemote && options?.showSecrets) {
            throw new Error("can't enable `showSecrets` for remote workspaces");
        }
        this.name = options.stackName;
        const summary = await this.info(!this.isRemote && options?.showSecrets);
        return {
            stdout: renameResult.stdout,
            stderr: renameResult.stderr,
            summary: summary,
        };
    }
    /**
     * Import resources into the stack
     *
     * @param options Options to specify resources and customize the behavior of the import.
     */
    async import(options) {
        const args = ["import", "--yes", "--skip-preview"];
        if (options.message) {
            args.push("--message", options.message);
        }
        applyGlobalOpts(options, args);
        let importResult;
        // for import operations, generate a temporary directory to store the following:
        //   - the import file when the user specifies resources to import
        //   - the output file for the generated code
        // we use the import file as input to the import command
        // we the output file to read the generated code and return it to the user
        let tempDir = "";
        try {
            tempDir = await fs.promises.mkdtemp(pathlib.join(os.tmpdir(), "pulumi-import-"));
            const tempGeneratedCodeFile = pathlib.join(tempDir, "generated-code.txt");
            if (options.resources) {
                // user has specified resources to import, write them to a temp import file
                const tempImportFile = pathlib.join(tempDir, "import.json");
                await fs.promises.writeFile(tempImportFile, JSON.stringify({
                    nameTable: options.nameTable,
                    resources: options.resources,
                }));
                args.push("--file", tempImportFile);
            }
            if (options.generateCode === false) {
                // --generate-code is set to true by default
                // only use --generate-code=false if the user explicitly sets it to false
                args.push("--generate-code=false");
            }
            else {
                args.push("--out", tempGeneratedCodeFile);
            }
            if (options.protect === false) {
                // --protect is set to true by default
                // only use --protect=false if the user explicitly sets it to false
                args.push(`--protect=false`);
            }
            if (options.converter) {
                // if the user specifies a converter, pass it to `--from <converter>` argument of import
                args.push("--from", options.converter);
                if (options.converterArgs) {
                    // pass any additional arguments to the converter
                    // for example using {
                    //   converter: "terraform"
                    //   converterArgs: "./tfstate.json"
                    // }
                    // would be equivalent to `pulumi import --from terraform ./tfstate.json`
                    args.push("--");
                    args.push(...options.converterArgs);
                }
            }
            importResult = await this.runPulumiCmd(args, options.onOutput);
            const summary = await this.info(!this.isRemote && options.showSecrets);
            let generatedCode = "";
            if (options.generateCode !== false) {
                generatedCode = await fs.promises.readFile(tempGeneratedCodeFile, "utf8");
            }
            return {
                stdout: importResult.stdout,
                stderr: importResult.stderr,
                generatedCode: generatedCode,
                summary: summary,
            };
        }
        finally {
            if (tempDir !== "") {
                // clean up temp directory we used for the import file
                await fs.promises.rm(tempDir, { recursive: true });
            }
        }
    }
    /**
     * Adds environments to the end of a stack's import list. Imported
     * environments are merged in order per the ESC merge rules. The list of
     * environments behaves as if it were the import list in an anonymous
     * environment.
     *
     * @param environments
     *  The names of the environments to add to the stack's configuration
     */
    async addEnvironments(...environments) {
        await this.workspace.addEnvironments(this.name, ...environments);
    }
    /**
     * Returns the list of environments currently in the stack's import list.
     */
    async listEnvironments() {
        return this.workspace.listEnvironments(this.name);
    }
    /**
     * Removes an environment from a stack's import list.
     *
     * @param environment
     *  The name of the environment to remove from the stack's configuration
     */
    async removeEnvironment(environment) {
        await this.workspace.removeEnvironment(this.name, environment);
    }
    /**
     * Returns the config value associated with the specified key.
     *
     * @param key
     *  The key to use for the config lookup
     * @param path
     *  The key contains a path to a property in a map or list to get
     */
    async getConfig(key, path) {
        return this.workspace.getConfig(this.name, key, path);
    }
    /**
     * Returns the full config map associated with the stack in the workspace.
     */
    async getAllConfig() {
        return this.workspace.getAllConfig(this.name);
    }
    /**
     * Sets a config key-value pair on the stack in the associated Workspace.
     *
     * @param key
     *  The key to set.
     * @param value
     *  The config value to set.
     * @param path
     *  The key contains a path to a property in a map or list to set.
     */
    async setConfig(key, value, path) {
        return this.workspace.setConfig(this.name, key, value, path);
    }
    /**
     * Sets all specified config values on the stack in the associated
     * workspace.
     *
     * @param config
     *  The map of config key-value pairs to set.
     * @param path
     *  The keys contain a path to a property in a map or list to set.
     */
    async setAllConfig(config, path) {
        return this.workspace.setAllConfig(this.name, config, path);
    }
    /**
     * Sets all config values from a JSON string for the stack in the associated workspace.
     * The JSON string should be in the format produced by "pulumi config --json".
     *
     * @param configJson
     *  A JSON string containing the configuration values to set
     */
    async setAllConfigJson(configJson) {
        return this.workspace.setAllConfigJson(this.name, configJson);
    }
    /**
     * Removes the specified config key from the stack in the associated workspace.
     *
     * @param key
     *  The config key to remove.
     * @param path
     *  The key contains a path to a property in a map or list to remove.
     */
    async removeConfig(key, path) {
        return this.workspace.removeConfig(this.name, key, path);
    }
    /**
     * Removes the specified config keys from the stack in the associated workspace.
     *
     * @param keys
     *  The config keys to remove.
     * @param path
     *  The keys contain a path to a property in a map or list to remove.
     */
    async removeAllConfig(keys, path) {
        return this.workspace.removeAllConfig(this.name, keys, path);
    }
    /**
     * Gets and sets the config map used with the last update.
     */
    async refreshConfig() {
        return this.workspace.refreshConfig(this.name);
    }
    /**
     * Returns the tag value associated with specified key.
     *
     * @param key The key to use for the tag lookup.
     */
    async getTag(key) {
        return this.workspace.getTag(this.name, key);
    }
    /**
     * Sets a tag key-value pair on the stack in the associated workspace.
     *
     * @param key
     *  The tag key to set.
     * @param value
     *  The tag value to set.
     */
    async setTag(key, value) {
        await this.workspace.setTag(this.name, key, value);
    }
    /**
     * Removes the specified tag key-value pair from the stack in the associated
     * workspace.
     *
     * @param key The tag key to remove.
     */
    async removeTag(key) {
        await this.workspace.removeTag(this.name, key);
    }
    /**
     * Returns the full tag map associated with the stack in the workspace.
     */
    async listTags() {
        return this.workspace.listTags(this.name);
    }
    /**
     * Gets the current set of stack outputs from the last {@link Stack.up}.
     */
    async outputs() {
        return this.workspace.stackOutputs(this.name);
    }
    /**
     * Returns a list summarizing all previous and current results from Stack
     * lifecycle operations (up/preview/refresh/destroy).
     */
    async history(pageSize, page, showSecrets) {
        const args = ["stack", "history", "--json"];
        if (showSecrets ?? true) {
            args.push("--show-secrets");
        }
        if (pageSize) {
            if (!page || page < 1) {
                page = 1;
            }
            args.push("--page-size", Math.floor(pageSize).toString(), "--page", Math.floor(page).toString());
        }
        const result = await this.runPulumiCmd(args);
        return JSON.parse(result.stdout, (key, value) => {
            if (key === "startTime" || key === "endTime") {
                return new Date(value);
            }
            return value;
        });
    }
    async info(showSecrets) {
        const history = await this.history(1 /*pageSize*/, undefined, showSecrets);
        if (!history || history.length === 0) {
            return undefined;
        }
        return history[0];
    }
    /**
     * Stops a stack's currently running update. It returns an error if no
     * update is currently running. Note that this operation is _very
     * dangerous_, and may leave the stack in an inconsistent state if a
     * resource operation was pending when the update was canceled.
     */
    async cancel() {
        await this.runPulumiCmd(["cancel", "--yes"]);
    }
    /**
     * Exports the deployment state of the stack. This can be combined with
     * {@link Stack.importStack} to edit a stack's state (such as recovery from
     * failed deployments).
     */
    async exportStack() {
        return this.workspace.exportStack(this.name);
    }
    /**
     * Imports the specified deployment state into a pre-existing stack. This
     * can be combined with {@link Stack.exportStack} to edit a stack's state
     * (such as recovery from failed deployments).
     *
     * @param state
     *  The stack state to import.
     */
    async importStack(state) {
        return this.workspace.importStack(this.name, state);
    }
    async runPulumiCmd(args, onOutput, onError, signal) {
        let envs = {
            PULUMI_DEBUG_COMMANDS: "true",
        };
        if (this.isRemote) {
            envs["PULUMI_EXPERIMENTAL"] = "true";
        }
        const pulumiHome = this.workspace.pulumiHome;
        if (pulumiHome) {
            envs["PULUMI_HOME"] = pulumiHome;
        }
        envs = { ...envs, ...this.workspace.envVars };
        const additionalArgs = await this.workspace.serializeArgsForOp(this.name);
        args = [...args, "--stack", this.name, ...additionalArgs];
        const result = await this.workspace.pulumiCommand.run(args, this.workspace.workDir, envs, onOutput, onError, signal);
        await this.workspace.postCommandCallback(this.name);
        return result;
    }
    get isRemote() {
        const ws = this.workspace;
        return ws instanceof localWorkspace_1.LocalWorkspace ? ws.isRemote : false;
    }
    remoteArgs() {
        const ws = this.workspace;
        return ws instanceof localWorkspace_1.LocalWorkspace ? ws.remoteArgs() : [];
    }
}
exports.Stack = Stack;
function applyGlobalOpts(opts, args) {
    if (opts.color) {
        args.push("--color", opts.color);
    }
    if (opts.logFlow) {
        args.push("--logflow");
    }
    if (opts.logVerbosity) {
        args.push("--verbose", opts.logVerbosity.toString());
    }
    if (opts.logToStdErr) {
        args.push("--logtostderr");
    }
    if (opts.tracing) {
        args.push("--tracing", opts.tracing);
    }
    if (opts.debug) {
        args.push("--debug");
    }
    if (opts.suppressOutputs) {
        args.push("--suppress-outputs");
    }
    if (opts.suppressProgress) {
        args.push("--suppress-progress");
    }
    if (opts.configFile) {
        args.push("--config-file", opts.configFile);
    }
}
/**
 * Returns a stack name formatted with the greatest possible specificity:
 * `org/project/stack` or `user/project/stack` Using this format avoids
 * ambiguity in stack identity guards creating or selecting the wrong stack.
 *
 * Note: legacy DIY backends (local file, S3, Azure Blob) do not support
 * stack names in this format, and instead only use the stack name without an
 * org/user or project to qualify it.
 *
 * See: https://github.com/pulumi/pulumi/issues/2522
 *
 * Non-legacy DIY backends do support the `org/project/stack` format, but `org`
 * must be set to "organization".
 *
 * @param org
 *  The org (or user) that contains the Stack.
 * @param project
 *  The project that parents the Stack.
 * @param stack
 *  The name of the Stack.
 */
function fullyQualifiedStackName(org, project, stack) {
    return `${org}/${project}/${stack}`;
}
exports.fullyQualifiedStackName = fullyQualifiedStackName;
class EventsServer {
    constructor(onEvent) {
        this.onEvent = onEvent;
        this.onEvent = onEvent;
    }
    streamEvents(call, callback) {
        call.on("data", (request) => {
            const eventStr = request.getEvent();
            try {
                const event = JSON.parse(eventStr);
                this.onEvent(event);
            }
            catch (e) {
                log.warn(`Failed to parse engine event: ${e.toString()}`);
            }
        });
        call.on("end", () => {
            callback(null, new empty_pb_1.Empty());
        });
        call.on("error", (err) => {
            log.warn(`Error in event stream: ${err.toString()}`);
            callback(err, null);
        });
    }
}
const execKind = {
    local: "auto.local",
    inline: "auto.inline",
};
const createLogFile = (command) => {
    const logDir = fs.mkdtempSync(upath.joinSafe(os.tmpdir(), `automation-logs-${command}-`));
    const logFile = upath.joinSafe(logDir, "eventlog.txt");
    // just open/close the file to make sure it exists when we start polling.
    fs.closeSync(fs.openSync(logFile, "w"));
    return logFile;
};
const cleanUp = async (logFile, rl, server) => {
    if (rl) {
        // stop tailing
        await rl.tail.quit();
        // close the readline interface
        rl.rl.close();
    }
    if (server) {
        server.forceShutdown();
    }
    if (logFile) {
        // remove the logfile
        if (fs.rm) {
            // remove with Node JS 15.X+
            fs.rm(pathlib.dirname(logFile), { recursive: true }, () => {
                return;
            });
        }
        else {
            // remove with Node JS 14.X
            fs.rmdir(pathlib.dirname(logFile), { recursive: true }, () => {
                return;
            });
        }
    }
};
//# sourceMappingURL=stack.js.map