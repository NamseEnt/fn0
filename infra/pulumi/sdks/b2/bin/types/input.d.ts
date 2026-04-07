import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
export interface BucketCorsRule {
    /**
     * If present, this is a list of headers that are allowed in a pre-flight OPTIONS's request's Access-Control-Request-Headers header value.
     */
    allowedHeaders?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * A list specifying which operations the rule allows.
     */
    allowedOperations: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * A non-empty list specifying which origins the rule covers.
     */
    allowedOrigins: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * A name for humans to recognize the rule in a user interface.
     */
    corsRuleName: pulumi.Input<string>;
    /**
     * If present, this is a list of headers that may be exposed to an application inside the client.
     */
    exposeHeaders?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * This specifies the maximum number of seconds that a browser may cache the response to a preflight request.
     */
    maxAgeSeconds: pulumi.Input<number>;
}
export interface BucketDefaultServerSideEncryption {
    /**
     * Server-side encryption algorithm. AES256 is the only one supported.
     */
    algorithm?: pulumi.Input<string>;
    /**
     * Server-side encryption mode.
     */
    mode?: pulumi.Input<string>;
}
export interface BucketFileLockConfiguration {
    /**
     * Default retention settings for files uploaded to this bucket.
     */
    defaultRetention?: pulumi.Input<inputs.BucketFileLockConfigurationDefaultRetention>;
    /**
     * If present, the boolean value specifies whether bucket is File Lock-enabled. Defaults to <span pulumi-lang-nodejs="`false`" pulumi-lang-dotnet="`False`" pulumi-lang-go="`false`" pulumi-lang-python="`false`" pulumi-lang-yaml="`false`" pulumi-lang-java="`false`">`false`</span>. **Modifying this attribute will force creation of a new resource.**
     */
    isFileLockEnabled?: pulumi.Input<boolean>;
}
export interface BucketFileLockConfigurationDefaultRetention {
    /**
     * Default retention mode (compliance|governance|none).
     */
    mode: pulumi.Input<string>;
    /**
     * How long for to make files immutable.
     */
    period?: pulumi.Input<inputs.BucketFileLockConfigurationDefaultRetentionPeriod>;
}
export interface BucketFileLockConfigurationDefaultRetentionPeriod {
    /**
     * Duration.
     */
    duration: pulumi.Input<number>;
    /**
     * Unit for duration (days|years).
     */
    unit: pulumi.Input<string>;
}
export interface BucketFileVersionServerSideEncryption {
    /**
     * Server-side encryption algorithm. AES256 is the only one supported.
     */
    algorithm?: pulumi.Input<string>;
    /**
     * Key used in SSE-C mode.
     */
    key?: pulumi.Input<inputs.BucketFileVersionServerSideEncryptionKey>;
    /**
     * Server-side encryption mode.
     */
    mode?: pulumi.Input<string>;
}
export interface BucketFileVersionServerSideEncryptionKey {
    /**
     * Key identifier stored in file info metadata.
     */
    keyId?: pulumi.Input<string>;
    /**
     * Secret key value, in standard Base 64 encoding (RFC 4648).
     */
    secretB64?: pulumi.Input<string>;
}
export interface BucketLifecycleRule {
    /**
     * It says how long to keep file versions that are not the current version.
     */
    daysFromHidingToDeleting?: pulumi.Input<number>;
    /**
     * It cancels any unfinished large file versions after a given number of days.
     */
    daysFromStartingToCancelingUnfinishedLargeFiles?: pulumi.Input<number>;
    /**
     * It causes files to be hidden automatically after the given number of days.
     */
    daysFromUploadingToHiding?: pulumi.Input<number>;
    /**
     * It specifies which files in the bucket it applies to.
     */
    fileNamePrefix: pulumi.Input<string>;
}
export interface BucketNotificationRulesNotificationRule {
    /**
     * The list of event types for the event notification rule.
     */
    eventTypes: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Whether the event notification rule is enabled. Defaults to <span pulumi-lang-nodejs="`true`" pulumi-lang-dotnet="`True`" pulumi-lang-go="`true`" pulumi-lang-python="`true`" pulumi-lang-yaml="`true`" pulumi-lang-java="`true`">`true`</span>.
     */
    isEnabled?: pulumi.Input<boolean>;
    /**
     * Whether the event notification rule is suspended.
     */
    isSuspended?: pulumi.Input<boolean>;
    /**
     * A name for the event notification rule. The name must be unique among the bucket's notification rules.
     */
    name: pulumi.Input<string>;
    /**
     * Specifies which object(s) in the bucket the event notification rule applies to.
     */
    objectNamePrefix?: pulumi.Input<string>;
    /**
     * A brief description of why the event notification rule was suspended.
     */
    suspensionReason?: pulumi.Input<string>;
    /**
     * The target configuration for the event notification rule.
     */
    targetConfiguration: pulumi.Input<inputs.BucketNotificationRulesNotificationRuleTargetConfiguration>;
}
export interface BucketNotificationRulesNotificationRuleTargetConfiguration {
    /**
     * When present, additional header name/value pairs to be sent on the webhook invocation.
     */
    customHeaders?: pulumi.Input<pulumi.Input<inputs.BucketNotificationRulesNotificationRuleTargetConfigurationCustomHeader>[]>;
    /**
     * The signing secret for use in verifying the X-Bz-Event-Notification-Signature.
     */
    hmacSha256SigningSecret?: pulumi.Input<string>;
    /**
     * The type of the target configuration, currently "webhook" only.
     */
    targetType: pulumi.Input<string>;
    /**
     * The URL for the webhook.
     */
    url: pulumi.Input<string>;
}
export interface BucketNotificationRulesNotificationRuleTargetConfigurationCustomHeader {
    /**
     * Name of the header.
     */
    name: pulumi.Input<string>;
    /**
     * Value of the header.
     */
    value: pulumi.Input<string>;
}
