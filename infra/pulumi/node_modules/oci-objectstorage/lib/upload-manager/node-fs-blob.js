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
Object.defineProperty(exports, "__esModule", { value: true });
exports.NodeFSBlob = void 0;
const fs_1 = require("fs");
const crypto_1 = require("crypto");
/*
 * NodeFSBlob to support Upload manager in Node
 */
class NodeFSBlob {
    constructor(filePath, highWaterMark, start, end, fileSize) {
        this.filePath = filePath;
        this.highWaterMark = highWaterMark;
        this.type = "";
        this.fileSize = fileSize || fs_1.statSync(this.filePath).size;
        this.start = start || 0;
        this.end = end || this.fileSize;
        if (this.end > this.fileSize) {
            this.end = this.fileSize;
        }
        this.size = this.end - this.start;
    }
    /*
     * @Summary Get Data for the given NodeFSBlob
     * returns stream.Readable object
     */
    getData() {
        return __awaiter(this, void 0, void 0, function* () {
            const end = this.fileSize === 0 ? this.end : this.end - 1;
            return fs_1.createReadStream(this.filePath, {
                start: this.start,
                // As per https://nodejs.org/api/fs.html#fs_fs_createreadstream_path_options, end is inclusive.
                // However, as per https://developer.mozilla.org/en-US/docs/Web/API/Blob/slice, end is non-inclusive.
                // Read till end - 1 to match Blob's behavior
                // Note: if this.end = 0, and end: -1, we will have an issue giving The value of "end" is out of range. It must be >= 0 and <= 2 ** 53 - 1. Received -1
                end: end,
                highWaterMark: this.highWaterMark
            });
        });
    }
    /*
     * Slice NodeFSBlob in to smaller chunk
     * @param start start byte of NodeFSBlob
     * @param end end byte of NodeFSBlob
     * @param cntentType contentType of NodeFSBlob
     * @returns NodeFSBlob Chunk
     */
    slice(start, end, contentType) {
        return new NodeFSBlob(this.filePath, this.highWaterMark, start, end, this.fileSize);
    }
    /*
     * Get MD5 hash of the NodeFSBlob
     * @return Md5 hash string
     */
    getMD5Hash() {
        return __awaiter(this, void 0, void 0, function* () {
            return new Promise((resolve) => __awaiter(this, void 0, void 0, function* () {
                const md5Hash = crypto_1.createHash("md5");
                const dataStream = yield this.getData();
                dataStream
                    .on("data", data => md5Hash.update(data))
                    .on("end", () => resolve(md5Hash.digest("base64")));
            }));
        });
    }
    /*
     * @Summary Get text for the given NodeFsBlob
     * returns ArrayBuffer object
     */
    arrayBuffer() {
        return __awaiter(this, void 0, void 0, function* () {
            return new Promise(resolve => {
                resolve(fs_1.readFileSync(this.filePath));
            });
        });
    }
    /*
     * @Summary Get text for the given NodeFsBlob
     * returns string object
     */
    text() {
        return __awaiter(this, void 0, void 0, function* () {
            const data = yield this.getData();
            return this.streamToString(data);
        });
    }
    streamToString(stream) {
        return __awaiter(this, void 0, void 0, function* () {
            const chunks = [];
            return new Promise((resolve, reject) => {
                stream.on("data", chunk => chunks.push(chunk));
                stream.on("error", reject);
                stream.on("end", () => resolve(Buffer.concat(chunks).toString("utf8")));
            });
        });
    }
    /*
     * @Summary Get stream for the given NodeFsBlob
     * returns ReadableStream object
     */
    stream() {
        throw "ReadableStream not supported in NodeJs. Use getData() to fetch Stream";
    }
}
exports.NodeFSBlob = NodeFSBlob;
//# sourceMappingURL=node-fs-blob.js.map