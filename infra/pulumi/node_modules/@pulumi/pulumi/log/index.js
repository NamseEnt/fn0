"use strict";
// Copyright 2016-2018, Pulumi Corporation.
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
Object.defineProperty(exports, "__esModule", { value: true });
const engproto = __importStar(require("../proto/engine_pb"));
const settings_1 = require("../runtime/settings");
const state_1 = require("../runtime/state");
let lastLog = Promise.resolve();
const messageLevels = {
    [engproto.LogSeverity.DEBUG]: "debug",
    [engproto.LogSeverity.INFO]: "info",
    [engproto.LogSeverity.WARNING]: "warn",
    [engproto.LogSeverity.ERROR]: "error",
};
/**
 * Returns true if any errors have occurred in the program.
 */
function hasErrors() {
    return state_1.getStore().logErrorCount > 0;
}
exports.hasErrors = hasErrors;
/**
 * Logs a debug-level message that is generally hidden from end-users.
 */
function debug(msg, resource, streamId, ephemeral) {
    const engine = settings_1.getEngine();
    if (engine) {
        return log(engine, engproto.LogSeverity.DEBUG, msg, resource, streamId, ephemeral);
    }
    else {
        if (settings_1.excessiveDebugOutput) {
            console.log(`debug: [runtime] ${msg}`);
        }
        return Promise.resolve();
    }
}
exports.debug = debug;
/**
 * Logs an informational message that is generally printed to standard output
 * during resource operations.
 */
function info(msg, resource, streamId, ephemeral) {
    const engine = settings_1.getEngine();
    if (engine) {
        return log(engine, engproto.LogSeverity.INFO, msg, resource, streamId, ephemeral);
    }
    else {
        console.log(`info: [runtime] ${msg}`);
        return Promise.resolve();
    }
}
exports.info = info;
/**
 * Logs a warning to indicate that something went wrong, but not
 * catastrophically so.
 */
function warn(msg, resource, streamId, ephemeral) {
    const engine = settings_1.getEngine();
    if (engine) {
        return log(engine, engproto.LogSeverity.WARNING, msg, resource, streamId, ephemeral);
    }
    else {
        console.warn(`warning: [runtime] ${msg}`);
        return Promise.resolve();
    }
}
exports.warn = warn;
/**
 * Logs a fatal condition. Consider raising an exception after calling error to
 * stop the Pulumi program.
 */
function error(msg, resource, streamId, ephemeral) {
    state_1.getStore().logErrorCount++; // remember the error so we can suppress leaks.
    const engine = settings_1.getEngine();
    if (engine) {
        return log(engine, engproto.LogSeverity.ERROR, msg, resource, streamId, ephemeral);
    }
    else {
        console.error(`error: [runtime] ${msg}`);
        return Promise.resolve();
    }
}
exports.error = error;
function log(engine, sev, msg, resource, streamId, ephemeral) {
    // Ensure we log everything in serial order.
    const keepAlive = settings_1.rpcKeepAlive();
    const urnPromise = resource ? resource.urn.promise() : Promise.resolve("");
    lastLog = Promise.all([lastLog, urnPromise]).then(([_, urn]) => {
        return new Promise((resolve, reject) => {
            try {
                const req = new engproto.LogRequest();
                req.setSeverity(sev);
                req.setMessage(msg);
                req.setUrn(urn);
                req.setStreamid(streamId === undefined ? 0 : streamId);
                req.setEphemeral(ephemeral === true);
                engine.log(req, () => {
                    resolve(); // let the next log through
                    keepAlive(); // permit RPC channel tear-downs
                });
            }
            catch (err) {
                reject(err);
            }
        });
    });
    return lastLog.catch((err) => {
        // debug messages never go to stdout/err
        if (sev !== engproto.LogSeverity.DEBUG) {
            // if we're unable to deliver the log message, deliver to stderr instead
            console.error(`failed to deliver log message. \nerror: ${err} \noriginal message: ${msg}\n message severity: ${messageLevels[sev]}`);
        }
        // we still need to free up the outstanding promise chain, whether or not delivery succeeded.
        keepAlive();
    });
}
//# sourceMappingURL=index.js.map