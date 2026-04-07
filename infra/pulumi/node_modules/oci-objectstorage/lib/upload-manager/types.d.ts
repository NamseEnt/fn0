/**
 * Copyright (c) 2020, 2021 Oracle and/or its affiliates.  All rights reserved.
 * This software is dual-licensed to you under the Universal Permissive License (UPL) 1.0 as shown at https://oss.oracle.com/licenses/upl or Apache License 2.0 as shown at http://www.apache.org/licenses/LICENSE-2.0. You may choose either license.
 */
/// <reference types="node" />
import { RequireOnlyOne } from "oci-common";
import { requests } from "oci-objectstorage";
import { Readable } from "stream";
export declare type BinaryBody = Uint8Array | Buffer | Readable | ReadableStream | string | Blob;
export interface ContentTypes {
    blob?: Blob;
    filePath?: string;
    stream?: BinaryBody;
}
declare type PutObjectContent = "putObjectBody" | "contentLength";
declare type UploadPartContent = "uploadPartBody" | "contentLength";
declare type ContentMD5HashDetails<T extends {
    contentMD5?: string;
}> = Pick<T, "contentMD5">;
declare type uploadPartData = "uploadId" | "uploadPartNum";
export declare type Content = RequireOnlyOne<ContentTypes>;
export declare type PutObjectContentDetails = Pick<requests.PutObjectRequest, PutObjectContent>;
export declare type UploadPartContentDetails = Pick<requests.UploadPartRequest, UploadPartContent>;
export declare type PutObjectContentMD5HashDetails = ContentMD5HashDetails<requests.PutObjectRequest>;
export declare type UploadPartContentMD5HashDetails = ContentMD5HashDetails<requests.UploadPartRequest>;
export declare type UploadPartDetails = Pick<requests.UploadPartRequest, uploadPartData>;
export declare type RequestDetails = Omit<requests.PutObjectRequest, PutObjectContent>;
export {};
