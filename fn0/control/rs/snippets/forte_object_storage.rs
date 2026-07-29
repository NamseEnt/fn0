let bucket = object_storage::private::bucket();

bucket.put("avatars/42.png", Some("image/png"), bytes).await?;

// hand the browser a direct download link
let url = bucket
    .presigned_get_url(
        "avatars/42.png",
        Duration::from_secs(3600),
    )
    .await?;
