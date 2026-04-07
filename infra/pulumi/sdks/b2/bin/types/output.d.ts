import * as outputs from "../types/output";
export interface BucketCorsRule {
    /**
     * If present, this is a list of headers that are allowed in a pre-flight OPTIONS's request's Access-Control-Request-Headers header value.
     */
    allowedHeaders?: string[];
    /**
     * A list specifying which operations the rule allows.
     */
    allowedOperations: string[];
    /**
     * A non-empty list specifying which origins the rule covers.
     */
    allowedOrigins: string[];
    /**
     * A name for humans to recognize the rule in a user interface.
     */
    corsRuleName: string;
    /**
     * If present, this is a list of headers that may be exposed to an application inside the client.
     */
    exposeHeaders?: string[];
    /**
     * This specifies the maximum number of seconds that a browser may cache the response to a preflight request.
     */
    maxAgeSeconds: number;
}
export interface BucketDefaultServerSideEncryption {
    /**
     * Server-side encryption algorithm. AES256 is the only one supported.
     */
    algorithm?: string;
    /**
     * Server-side encryption mode.
     */
    mode?: string;
}
export interface BucketFileLockConfiguration {
    /**
     * Default retention settings for files uploaded to this bucket.
     */
    defaultRetention?: outputs.BucketFileLockConfigurationDefaultRetention;
    /**
     * If present, the boolean value specifies whether bucket is File Lock-enabled. Defaults to <span pulumi-lang-nodejs="`false`" pulumi-lang-dotnet="`False`" pulumi-lang-go="`false`" pulumi-lang-python="`false`" pulumi-lang-yaml="`false`" pulumi-lang-java="`false`">`false`</span>. **Modifying this attribute will force creation of a new resource.**
     */
    isFileLockEnabled?: boolean;
}
export interface BucketFileLockConfigurationDefaultRetention {
    /**
     * Default retention mode (compliance|governance|none).
     */
    mode: string;
    /**
     * How long for to make files immutable.
     */
    period?: outputs.BucketFileLockConfigurationDefaultRetentionPeriod;
}
export interface BucketFileLockConfigurationDefaultRetentionPeriod {
    /**
     * Duration.
     */
    duration: number;
    /**
     * Unit for duration (days|years).
     */
    unit: string;
}
export interface BucketFileVersionServerSideEncryption {
    /**
     * Server-side encryption algorithm. AES256 is the only one supported.
     */
    algorithm?: string;
    /**
     * Key used in SSE-C mode.
     */
    key?: outputs.BucketFileVersionServerSideEncryptionKey;
    /**
     * Server-side encryption mode.
     */
    mode?: string;
}
export interface BucketFileVersionServerSideEncryptionKey {
    /**
     * Key identifier stored in file info metadata.
     */
    keyId?: string;
    /**
     * Secret key value, in standard Base 64 encoding (RFC 4648).
     */
    secretB64?: string;
}
export interface BucketLifecycleRule {
    /**
     * It says how long to keep file versions that are not the current version.
     */
    daysFromHidingToDeleting?: number;
    /**
     * It cancels any unfinished large file versions after a given number of days.
     */
    daysFromStartingToCancelingUnfinishedLargeFiles?: number;
    /**
     * It causes files to be hidden automatically after the given number of days.
     */
    daysFromUploadingToHiding?: number;
    /**
     * It specifies which files in the bucket it applies to.
     */
    fileNamePrefix: string;
}
export interface BucketNotificationRulesNotificationRule {
    /**
     * The list of event types for the event notification rule.
     */
    eventTypes: string[];
    /**
     * Whether the event notification rule is enabled. Defaults to <span pulumi-lang-nodejs="`true`" pulumi-lang-dotnet="`True`" pulumi-lang-go="`true`" pulumi-lang-python="`true`" pulumi-lang-yaml="`true`" pulumi-lang-java="`true`">`true`</span>.
     */
    isEnabled?: boolean;
    /**
     * Whether the event notification rule is suspended.
     */
    isSuspended: boolean;
    /**
     * A name for the event notification rule. The name must be unique among the bucket's notification rules.
     */
    name: string;
    /**
     * Specifies which object(s) in the bucket the event notification rule applies to.
     */
    objectNamePrefix?: string;
    /**
     * A brief description of why the event notification rule was suspended.
     */
    suspensionReason: string;
    /**
     * The target configuration for the event notification rule.
     */
    targetConfiguration: outputs.BucketNotificationRulesNotificationRuleTargetConfiguration;
}
export interface BucketNotificationRulesNotificationRuleTargetConfiguration {
    /**
     * When present, additional header name/value pairs to be sent on the webhook invocation.
     */
    customHeaders?: outputs.BucketNotificationRulesNotificationRuleTargetConfigurationCustomHeader[];
    /**
     * The signing secret for use in verifying the X-Bz-Event-Notification-Signature.
     */
    hmacSha256SigningSecret?: string;
    /**
     * The type of the target configuration, currently "webhook" only.
     */
    targetType: string;
    /**
     * The URL for the webhook.
     */
    url: string;
}
export interface BucketNotificationRulesNotificationRuleTargetConfigurationCustomHeader {
    /**
     * Name of the header.
     */
    name: string;
    /**
     * Value of the header.
     */
    value: string;
}
export interface GetAccountInfoAllowed {
    bucketId: string;
    bucketName: string;
    buckets: outputs.GetAccountInfoAllowedBucket[];
    capabilities: string[];
    namePrefix: string;
}
export interface GetAccountInfoAllowedBucket {
    id: string;
    name: string;
}
export interface GetBucketCorsRule {
    allowedHeaders: string[];
    allowedOperations: string[];
    allowedOrigins: string[];
    corsRuleName: string;
    exposeHeaders: string[];
    maxAgeSeconds: number;
}
export interface GetBucketDefaultServerSideEncryption {
    algorithm: string;
    mode: string;
}
export interface GetBucketFileFileVersion {
    action: string;
    bucketId: string;
    contentMd5: string;
    contentSha1: string;
    contentType: string;
    fileId: string;
    fileInfo: {
        [key: string]: string;
    };
    fileName: string;
    serverSideEncryptions: outputs.GetBucketFileFileVersionServerSideEncryption[];
    size: number;
    uploadTimestamp: number;
}
export interface GetBucketFileFileVersionServerSideEncryption {
    algorithm: string;
    mode: string;
}
export interface GetBucketFileLockConfiguration {
    defaultRetentions: outputs.GetBucketFileLockConfigurationDefaultRetention[];
    isFileLockEnabled: boolean;
}
export interface GetBucketFileLockConfigurationDefaultRetention {
    mode: string;
    periods: outputs.GetBucketFileLockConfigurationDefaultRetentionPeriod[];
}
export interface GetBucketFileLockConfigurationDefaultRetentionPeriod {
    duration: number;
    unit: string;
}
export interface GetBucketFilesFileVersion {
    action: string;
    bucketId: string;
    contentMd5: string;
    contentSha1: string;
    contentType: string;
    fileId: string;
    fileInfo: {
        [key: string]: string;
    };
    fileName: string;
    serverSideEncryptions: outputs.GetBucketFilesFileVersionServerSideEncryption[];
    size: number;
    uploadTimestamp: number;
}
export interface GetBucketFilesFileVersionServerSideEncryption {
    algorithm: string;
    mode: string;
}
export interface GetBucketLifecycleRule {
    daysFromHidingToDeleting: number;
    daysFromStartingToCancelingUnfinishedLargeFiles: number;
    daysFromUploadingToHiding: number;
    fileNamePrefix: string;
}
export interface GetBucketNotificationRulesNotificationRule {
    eventTypes: string[];
    isEnabled: boolean;
    isSuspended: boolean;
    name: string;
    objectNamePrefix: string;
    suspensionReason: string;
    targetConfigurations: outputs.GetBucketNotificationRulesNotificationRuleTargetConfiguration[];
}
export interface GetBucketNotificationRulesNotificationRuleTargetConfiguration {
    customHeaders: outputs.GetBucketNotificationRulesNotificationRuleTargetConfigurationCustomHeader[];
    hmacSha256SigningSecret: string;
    targetType: string;
    url: string;
}
export interface GetBucketNotificationRulesNotificationRuleTargetConfigurationCustomHeader {
    name: string;
    value: string;
}
