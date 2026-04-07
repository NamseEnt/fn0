/**
 * Copyright (c) 2020, 2021 Oracle and/or its affiliates.  All rights reserved.
 * This software is dual-licensed to you under the Universal Permissive License (UPL) 1.0 as shown at https://oss.oracle.com/licenses/upl or Apache License 2.0 as shown at http://www.apache.org/licenses/LICENSE-2.0. You may choose either license.
 */
import { ObjectStorageClient } from "../client";
import { UploadResponse } from "./upload-response";
import { UploadRequest } from "./upload-request";
import { BinaryBody } from "./types";
import { UploadOptions } from "./upload-options";
/**
 * UploadManager simplifies interaction with the Object Storage service by abstracting away the method used
 * to upload objects.  Depending on the configuration parameters(UploadOptions), UploadManager may choose to do a single
 * PutObject request, or break up the upload into multiple parts and utilize multi-part uploads.
 * <p>
 * An advantage of using multi-part uploads is the ability to be able to upload parts in parallel to reduce upload time.
 * <p>
 * Callers still have full control over how the UploadManager decides to perform the upload using UploadOptions.
 * NodeJS V8 Engine have a buffer size limitation, 2GB for 64-bit machine and 1GB for 32-bit machine.
 * Do not make the partSize greater than the buffer size limitation.
 */
export interface RawData {
    size: number;
    data: BinaryBody;
    md5Hash: string;
}
export declare class UploadManager {
    private readonly client;
    private readonly options;
    private uploadSize;
    private MAX_PARTS;
    private MAX_READ_SIZE;
    private numberOfRetries;
    private uploadRetryConfiguration;
    private numberOfSingleUploadRetry;
    constructor(client: ObjectStorageClient, options?: Partial<UploadOptions>);
    get logger(): import("oci-common/lib/log").Logger;
    private static defaultUploadOptions;
    private shouldUseMultipartUpload;
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
    upload(request: UploadRequest, callback?: Function): Promise<UploadResponse>;
    private singleUpload;
    private triggerUploadPart;
    private pushUploadParts;
    private multiUpload;
    private static composeRequestDetails;
}
