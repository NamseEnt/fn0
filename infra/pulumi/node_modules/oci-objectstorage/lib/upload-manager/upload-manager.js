"use strict";
/**
 * Copyright (c) 2020, 2021 Oracle and/or its affiliates.  All rights reserved.
 * This software is dual-licensed to you under the Universal Permissive License (UPL) 1.0 as shown at https://oss.oracle.com/licenses/upl or Apache License 2.0 as shown at http://www.apache.org/licenses/LICENSE-2.0. You may choose either license.
 */
var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
var __asyncValues = (this && this.__asyncValues) || function (o) {
    if (!Symbol.asyncIterator) throw new TypeError("Symbol.asyncIterator is not defined.");
    var m = o[Symbol.asyncIterator], i;
    return m ? m.call(o) : (o = typeof __values === "function" ? __values(o) : o[Symbol.iterator](), i = {}, verb("next"), verb("throw"), verb("return"), i[Symbol.asyncIterator] = function () { return this; }, i);
    function verb(n) { i[n] = o[n] && function (v) { return new Promise(function (resolve, reject) { v = o[n](v), settle(resolve, reject, v.done, v.value); }); }; }
    function settle(resolve, reject, d, v) { Promise.resolve(v).then(function(v) { resolve({ value: v, done: d }); }, reject); }
};
var __await = (this && this.__await) || function (v) { return this instanceof __await ? (this.v = v, this) : new __await(v); }
var __asyncGenerator = (this && this.__asyncGenerator) || function (thisArg, _arguments, generator) {
    if (!Symbol.asyncIterator) throw new TypeError("Symbol.asyncIterator is not defined.");
    var g = generator.apply(thisArg, _arguments || []), i, q = [];
    return i = {}, verb("next"), verb("throw"), verb("return"), i[Symbol.asyncIterator] = function () { return this; }, i;
    function verb(n) { if (g[n]) i[n] = function (v) { return new Promise(function (a, b) { q.push([n, v, a, b]) > 1 || resume(n, v); }); }; }
    function resume(n, v) { try { step(g[n](v)); } catch (e) { settle(q[0][3], e); } }
    function step(r) { r.value instanceof __await ? Promise.resolve(r.value.v).then(fulfill, reject) : settle(q[0][2], r); }
    function fulfill(value) { resume("next", value); }
    function reject(value) { resume("throw", value); }
    function settle(f, v) { if (f(v), q.shift(), q.length) resume(q[0][0], q[0][1]); }
};
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.UploadManager = void 0;
const index_1 = require("../../index");
const package_json_1 = require("../../package.json");
const os_1 = __importDefault(require("os"));
const oci_common_1 = require("oci-common");
const await_semaphore_1 = require("await-semaphore");
const blob_factory_1 = require("./blob-factory");
const node_fs_blob_1 = require("./node-fs-blob");
const CLIENT_VERSION = `Oracle-TypeScriptSDK/${package_json_1.version}`;
const OS_VERSION = `${os_1.default.type()} ${os_1.default.release()} ${os_1.default.platform()}`;
const UPLOAD_MANAGER_DEBUG_INFORMATION_LOG = `Client Version: ${CLIENT_VERSION}, OS Version: ${OS_VERSION}, See https://docs.oracle.com/iaas/Content/API/Concepts/sdk_troubleshooting.htm for common issues and steps to resolve them. If you need to contact support, or file a GitHub issue, please include this full error message.`;
class UploadManager {
    constructor(client, options) {
        this.client = client;
        // uploadSize will be a dictionary that keeps track of uploadSize per uploadId. This helps prevent mismatch uploadSize with
        // different upload when uploading multiple things in parallel.
        this.uploadSize = {};
        this.MAX_PARTS = 10000; // Object storage multipart upload does not allow more than 10000 parts.
        this.MAX_READ_SIZE = Number.MAX_SAFE_INTEGER;
        // numberOfRetries will be a dictionary that keeps track of numberOfRetries per uploadId. This helps prevent mismatch numberOfRetries with
        // different upload when uploading multiple things in parallel.
        this.numberOfRetries = {};
        this.uploadRetryConfiguration = { retryConfiguration: oci_common_1.NoRetryConfigurationDetails };
        this.numberOfSingleUploadRetry = 0;
        this.options = Object.assign(Object.assign({}, UploadManager.defaultUploadOptions), options);
    }
    get logger() {
        return oci_common_1.LOG.logger;
    }
    shouldUseMultipartUpload(content, singleUpload) {
        if (singleUpload || content.size == 0) {
            // Return false to force the upload to be a single upload,
            // multi-upload does not support sending a 0 sized part. Need to use single upload to handle 0 sized streams
            return false;
        }
        if (!content.size) {
            // Always use multiupload if content.size is not able to initially calculated.
            return true;
        }
        return content.size > this.options.partSize;
    }
    /**
     * Initiates a new upload request.  The upload manager will decide whether to use
     * a single PutObject call or multi-part uploads depending on the UploadOptions
     * specified.
     * <p>
     * Note, if a multi-part upload attempt fails, the UploadManager will attempt to
     * abort the upload to avoid leaving partially complete uploads and parts
     * (unless explicitly disabled via uploadOptions).
     *
     * @param request The upload request.
     * @return The UploadResponse.
     * @throws OciError if the upload fails for any reason.
     */
    upload(request, callback) {
        return __awaiter(this, void 0, void 0, function* () {
            const content = yield blob_factory_1.getContent(request.content, this.options);
            if (this.shouldUseMultipartUpload(content, request.singleUpload)) {
                return this.multiUpload(request.requestDetails, content, callback);
            }
            else {
                let body = yield content.getData();
                const dataFeeder = oci_common_1.getChunk(body, this.MAX_READ_SIZE);
                const dataPart = (yield dataFeeder.next()).value;
                return this.singleUpload(request.requestDetails, dataPart);
            }
        });
    }
    singleUpload(requestDetails, content) {
        return __awaiter(this, void 0, void 0, function* () {
            const contentDetails = {
                putObjectBody: content.data,
                contentLength: content.size
            };
            const contentMD5Hash = this.options.enforceMD5
                ? { contentMD5: content.md5Hash }
                : {};
            if (this.logger)
                this.logger.debug("uploading using single upload");
            try {
                const response = yield this.client.putObject(Object.assign(Object.assign(Object.assign(Object.assign({}, requestDetails), contentDetails), contentMD5Hash), this.uploadRetryConfiguration));
                this.numberOfSingleUploadRetry = 0;
                return {
                    eTag: response.eTag,
                    contentMd5: response.opcContentMd5,
                    opcRequestId: response.opcRequestId,
                    opcClientRequestId: response.opcClientRequestId
                };
            }
            catch (e) {
                if (this.numberOfSingleUploadRetry < 3) {
                    if (this.logger)
                        this.logger.error(`putObject failed, will retry. Last known error: ${e}`);
                    this.numberOfSingleUploadRetry += 1;
                    return yield this.singleUpload(requestDetails, content);
                }
                else {
                    if (this.logger)
                        this.logger.error(`putObject failed to retry ${this.numberOfSingleUploadRetry} times. Error: ${e}`);
                    const error = {
                        message: `putObject failed to retry ${this.numberOfSingleUploadRetry} times. Error: ${e}`,
                        troubleShootingInfo: UPLOAD_MANAGER_DEBUG_INFORMATION_LOG
                    };
                    throw error;
                }
            }
        });
    }
    triggerUploadPart(content, requestDetails, uploadId, uploadPartNum, semaphore, totalSize, callback) {
        return __awaiter(this, void 0, void 0, function* () {
            try {
                return yield semaphore.use(() => __awaiter(this, void 0, void 0, function* () {
                    let contentDetails = {};
                    let contentMD5Hash = {};
                    if (content instanceof node_fs_blob_1.NodeFSBlob) {
                        contentDetails = {
                            uploadPartBody: yield content.getData(),
                            contentLength: content.size
                        };
                        contentMD5Hash = this.options.enforceMD5
                            ? { contentMD5: yield content.getMD5Hash() }
                            : {};
                    }
                    else if ("data" in content) {
                        contentDetails = {
                            uploadPartBody: content.data,
                            contentLength: content.size
                        };
                        contentMD5Hash = this.options.enforceMD5 ? { contentMD5: content.md5Hash } : {};
                    }
                    const uploadPartDetails = {
                        uploadId: uploadId,
                        uploadPartNum: uploadPartNum
                    };
                    let response = yield this.client.uploadPart(Object.assign(Object.assign(Object.assign(Object.assign(Object.assign({}, requestDetails), uploadPartDetails), contentDetails), contentMD5Hash), this.uploadRetryConfiguration));
                    const uploadSize = (this.uploadSize[uploadId] += content.size);
                    const progress = (uploadSize / totalSize) * 100;
                    const result = {
                        etag: response.eTag,
                        partNum: uploadPartNum,
                        progress: progress.toFixed()
                    };
                    if (callback) {
                        callback(result);
                    }
                    return result;
                }));
            }
            catch (ex) {
                this.numberOfRetries[uploadId] = this.numberOfRetries[uploadId]
                    ? (this.numberOfRetries[uploadId] += 1)
                    : 1;
                if (this.numberOfRetries[uploadId] < 4) {
                    if (this.logger)
                        this.logger.error(`Upload part failed, will retry. Last known error: ${ex}`);
                    return yield this.triggerUploadPart(content, requestDetails, uploadId, uploadPartNum, semaphore, totalSize, callback);
                }
                else {
                    const error = {
                        message: `Upload part retried ${this.numberOfRetries[uploadId]} times and failed. Upload of part: ${uploadPartNum} failed due to ${ex}`,
                        troubleShootingInfo: UPLOAD_MANAGER_DEBUG_INFORMATION_LOG
                    };
                    throw error;
                }
            }
        });
    }
    pushUploadParts(totalSize, requestDetails, uploadId, content, callback) {
        var e_1, _a;
        return __awaiter(this, void 0, void 0, function* () {
            let uploadPartNum = 1;
            const semaphore = new await_semaphore_1.Semaphore(this.options.maxConcurrentUploads);
            const partUploadPromises = [];
            let dataFeeder;
            if (this.options.disableBufferingForFiles && content instanceof node_fs_blob_1.NodeFSBlob) {
                dataFeeder = getFileChunk(content, this.options.partSize);
            }
            else {
                let body = yield content.getData();
                dataFeeder = oci_common_1.getChunk(body, this.options.partSize);
            }
            try {
                for (var dataFeeder_1 = __asyncValues(dataFeeder), dataFeeder_1_1; dataFeeder_1_1 = yield dataFeeder_1.next(), !dataFeeder_1_1.done;) {
                    const dataPart = dataFeeder_1_1.value;
                    if (partUploadPromises.length > this.MAX_PARTS) {
                        throw new Error(`Exceeded ${this.MAX_PARTS} as part of the upload to ${requestDetails.bucketName}. ${UPLOAD_MANAGER_DEBUG_INFORMATION_LOG}`);
                    }
                    if (dataPart.size === 0) {
                        // If we have a 0 length part, we don't want to upload this.
                        continue;
                    }
                    // let partToUpload = new StreamBlob(dataPart.data, this.options.partSize);
                    partUploadPromises.push(this.triggerUploadPart(dataPart, requestDetails, uploadId, uploadPartNum, semaphore, totalSize, callback));
                    uploadPartNum++;
                }
            }
            catch (e_1_1) { e_1 = { error: e_1_1 }; }
            finally {
                try {
                    if (dataFeeder_1_1 && !dataFeeder_1_1.done && (_a = dataFeeder_1.return)) yield _a.call(dataFeeder_1);
                }
                finally { if (e_1) throw e_1.error; }
            }
            // }
            return partUploadPromises;
        });
    }
    multiUpload(requestDetails, content, callback) {
        return __awaiter(this, void 0, void 0, function* () {
            const timestamp = Date.now().toString();
            let opcMeta = {};
            if (requestDetails.opcMeta) {
                Object.entries(requestDetails.opcMeta).forEach(([key, value]) => {
                    Object.assign(opcMeta, { ["opc-meta-" + key]: value });
                });
            }
            const createUploadResponse = yield this.client.createMultipartUpload(Object.assign(Object.assign({}, UploadManager.composeRequestDetails(requestDetails)), { createMultipartUploadDetails: {
                    object: requestDetails.objectName,
                    storageTier: requestDetails.storageTier
                        ? requestDetails.storageTier
                        : index_1.models.StorageTier.Standard,
                    metadata: requestDetails.opcMeta ? opcMeta : undefined,
                    cacheControl: requestDetails.cacheControl,
                    contentType: requestDetails.contentType,
                    contentLanguage: requestDetails.contentLanguage,
                    contentEncoding: requestDetails.contentEncoding,
                    contentDisposition: requestDetails.contentDisposition
                } }));
            const uploadId = createUploadResponse.multipartUpload.uploadId;
            this.uploadSize[uploadId] = 0;
            try {
                const totalSize = content.size;
                const partUploadPromises = yield this.pushUploadParts(totalSize, requestDetails, uploadId, content, callback);
                const uploadPartDetails = yield Promise.all(partUploadPromises);
                const response = yield this.client.commitMultipartUpload(Object.assign(Object.assign({}, UploadManager.composeRequestDetails(requestDetails)), { commitMultipartUploadDetails: {
                        partsToCommit: uploadPartDetails
                    }, uploadId: uploadId }));
                return {
                    eTag: response.eTag,
                    multipartMd5: response.opcMultipartMd5,
                    opcRequestId: response.opcRequestId,
                    opcClientRequestId: response.opcClientRequestId
                };
            }
            catch (ex) {
                if (this.options.isDisableAutoAbort) {
                    if (this.logger)
                        this.logger.info(`Not aborting failed multipart upload as per configuration, client must manually abort it`);
                }
                else {
                    if (this.logger)
                        this.logger.error(`Aborting multi-part upload ${uploadId}`);
                    yield this.client.abortMultipartUpload(Object.assign(Object.assign({}, UploadManager.composeRequestDetails(requestDetails)), { uploadId: uploadId }));
                    if (this.logger)
                        this.logger.error(`Abort complete`);
                }
                if (ex instanceof oci_common_1.OciError)
                    throw ex;
                const error = {
                    message: `Failed to upload due to ${ex}`,
                    troubleShootingInfo: UPLOAD_MANAGER_DEBUG_INFORMATION_LOG
                };
                throw error;
            }
        });
    }
    static composeRequestDetails(requestDetails) {
        return {
            namespaceName: requestDetails.namespaceName,
            bucketName: requestDetails.bucketName,
            objectName: requestDetails.objectName,
            opcClientRequestId: requestDetails.opcClientRequestId
        };
    }
}
exports.UploadManager = UploadManager;
UploadManager.defaultUploadOptions = {
    partSize: 20 * 1024 * 1024,
    maxConcurrentUploads: 5,
    allowedMemoryUsage: 5 * 20 * 1024 * 1024,
    enforceMD5: false,
    isDisableAutoAbort: false,
    disableBufferingForFiles: true
};
function getFileChunk(content, partSize) {
    return FileChunk(content, partSize);
}
function FileChunk(content, partSize) {
    return __asyncGenerator(this, arguments, function* FileChunk_1() {
        let totalSize = content.size;
        for (let currentChunkStart = 0; currentChunkStart < totalSize; currentChunkStart += partSize) {
            yield yield __await(content.slice(currentChunkStart, currentChunkStart + partSize));
        }
    });
}
//# sourceMappingURL=upload-manager.js.map