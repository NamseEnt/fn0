use bytes::Bytes;
use fn0_ski::Request;
use http_body_util::{BodyExt, Empty};

fn user_js() -> &'static str {
    r#"
globalThis.handler = async () => {
  const randomBytes = crypto.getRandomValues(new Uint8Array(32));
  if (randomBytes.length !== 32) throw new Error("getRandomValues length");
  if (randomBytes.every((byte) => byte === 0)) {
    throw new Error("getRandomValues returned all zeros");
  }

  const uuid = crypto.randomUUID();
  const uuidV4Pattern =
    /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/;
  if (!uuidV4Pattern.test(uuid)) throw new Error("randomUUID format: " + uuid);

  const digest = await crypto.subtle.digest(
    "SHA-256",
    new TextEncoder().encode("hello"),
  );
  const digestHex = Array.from(new Uint8Array(digest))
    .map((byte) => byte.toString(16).padStart(2, "0"))
    .join("");
  const expectedSha256 =
    "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824";
  if (digestHex !== expectedSha256) throw new Error("sha256: " + digestHex);

  const hmacKey = await crypto.subtle.generateKey(
    { name: "HMAC", hash: "SHA-256" },
    false,
    ["sign", "verify"],
  );
  const message = new TextEncoder().encode("message");
  const signature = await crypto.subtle.sign("HMAC", hmacKey, message);
  const verified = await crypto.subtle.verify("HMAC", hmacKey, signature, message);
  if (!verified) throw new Error("hmac verify failed");

  const aesKey = await crypto.subtle.generateKey(
    { name: "AES-GCM", length: 256 },
    false,
    ["encrypt", "decrypt"],
  );
  const initializationVector = crypto.getRandomValues(new Uint8Array(12));
  const plaintext = new TextEncoder().encode("secret");
  const ciphertext = await crypto.subtle.encrypt(
    { name: "AES-GCM", iv: initializationVector },
    aesKey,
    plaintext,
  );
  const decrypted = await crypto.subtle.decrypt(
    { name: "AES-GCM", iv: initializationVector },
    aesKey,
    ciphertext,
  );
  if (new TextDecoder().decode(decrypted) !== "secret") {
    throw new Error("aes-gcm round trip failed");
  }

  return new Response("ok");
};
"#
}

fn empty_request() -> Request {
    let body = http_body_util::combinators::UnsyncBoxBody::new(
        Empty::<Bytes>::new().map_err(|never| match never {}),
    );
    hyper::Request::builder()
        .method("GET")
        .uri("http://localhost/")
        .body(body)
        .unwrap()
}

#[tokio::test]
async fn web_crypto_api_round_trips() {
    let response = fn0_ski::run(user_js(), "/web_crypto_test.js", empty_request(), None)
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(&body[..], b"ok");
}
