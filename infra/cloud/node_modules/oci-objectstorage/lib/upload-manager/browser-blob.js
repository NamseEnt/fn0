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
exports.BrowserBlob = void 0;
const crypto_1 = require("crypto");
/*
 * BrowserBlob to support Upload manager in Browser
 */
class BrowserBlob {
    constructor(blob, chunkSize) {
        this.size = blob.size;
        this.blob = blob;
        this.chunkSize = chunkSize;
        this.type = blob.type;
    }
    /*
     * @Summary Get Data for the given BrowserBlob
     * returns UploadableBlob object
     */
    getData() {
        return __awaiter(this, void 0, void 0, function* () {
            return new Promise(resolve => {
                resolve(this.blob);
            });
        });
    }
    /*
     * @Summary Get text for the given BrowserBlob
     * returns string object
     */
    text() {
        return __awaiter(this, void 0, void 0, function* () {
            return this.blob.text();
        });
    }
    /*
     * @Summary Get array buffer for the given BrowserBlob
     * returns ArrayBuffer object
     */
    arrayBuffer() {
        return __awaiter(this, void 0, void 0, function* () {
            return this.blob.arrayBuffer();
        });
    }
    /*
     * @Summary Get stream for the given BrowserBlob
     * returns Readable object
     */
    stream() {
        return this.blob.stream();
    }
    /*
     * Get MD5 hash of the BrowserBlob
     * @return Md5 hash string
     */
    getMD5Hash() {
        return __awaiter(this, void 0, void 0, function* () {
            return new Promise(resolve => {
                const blobSize = this.size - 1;
                const chunkSize = this.chunkSize;
                const md5 = crypto_1.createHash("md5");
                const fileReader = new FileReader();
                var updateMD5 = function () {
                    md5.update(fileReader.result);
                };
                // Read blob in smaller chunks of size this.chunkSize specified by user as UploadOptions
                // to avoid loading whole blob in memory.
                for (var startPos = 0; startPos < blobSize; startPos += chunkSize) {
                    ((blb, start) => {
                        const browserBlob = blb.slice(start, chunkSize + start);
                        fileReader.onload = updateMD5;
                        fileReader.readAsBinaryString(browserBlob.blob);
                    })(this, startPos);
                }
                resolve(md5.digest("base64"));
            });
        });
    }
    /*
     * Slice BrowserBlob in to smaller chunk
     * @param start start byte of BrowserBlob
     * @param end end byte of BrowserBlob
     * @param cntentType contentType of BrowserBlob
     * @returns BrowserBlob Chunk
     */
    slice(start, end, contentType) {
        return new BrowserBlob(this.blob.slice(start, end, contentType), this.chunkSize);
    }
}
exports.BrowserBlob = BrowserBlob;
//# sourceMappingURL=browser-blob.js.map