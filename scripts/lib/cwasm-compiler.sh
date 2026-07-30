# shellcheck shell=bash
# fn0-wasmtime cwasm-compiler lambda lifecycle. Source-only.
#
# ensure_cwasm_pending <new_fn0_wasmtime_version>
#   Idempotent. Builds (if needed) the cwasm-compiler lambda for <new_wasmtime>,
#   registers <new_wasmtime> as control's pending (or seeds active if none),
#   and synchronously compiles every R2 /original/ object that is not yet under
#   /compiled/<new_wasmtime>/. Re-running picks up where it left off.
#
# Caller must have sourced pulumi-outputs.sh and control-admin.sh, and called
# load_pulumi_outputs.

if [[ -n "${__FN0_CWASM_COMPILER_LOADED:-}" ]]; then
  return 0
fi
__FN0_CWASM_COMPILER_LOADED=1

# shellcheck source=r2.sh
source "$(dirname "${BASH_SOURCE[0]}")/r2.sh"

CWASM_COMPILER_PARALLEL="${CWASM_COMPILER_PARALLEL:-20}"

ensure_cwasm_lambda() {
  local new_fn0_wasmtime_version="$1"
  if [[ -z "$new_fn0_wasmtime_version" ]]; then
    echo "ensure_cwasm_lambda: missing version" >&2
    return 2
  fi
  local new_fn0_wasmtime_version_dash="${new_fn0_wasmtime_version//./-}"
  CWASM_LAMBDA_FUNCTION_NAME="fn0-cwasm-compiler-${new_fn0_wasmtime_version_dash}"

  local cwasm_region cwasm_ecr cwasm_role builder_ak builder_sk
  local r2_endpoint r2_bucket r2_ak r2_sk
  cwasm_region="$(pulumi_pick cwasmCompilerBucketRegion)"
  cwasm_ecr="$(pulumi_pick cwasmCompilerEcrRepository)"
  cwasm_role="$(pulumi_pick cwasmCompilerRoleArn)"
  builder_ak="$(pulumi_pick cwasmCompilerBuilderAccessKeyId)"
  builder_sk="$(pulumi_pick cwasmCompilerBuilderSecretAccessKey)"
  r2_endpoint="$(pulumi_pick bundleStoreR2Endpoint)"
  r2_bucket="$(pulumi_pick bundleStoreR2BucketName)"
  r2_ak="$(pulumi_pick bundleStoreR2AccessKeyId)"
  r2_sk="$(pulumi_pick bundleStoreR2SecretAccessKey)"
  local v
  for v in cwasm_region cwasm_ecr cwasm_role builder_ak builder_sk \
           r2_endpoint r2_bucket r2_ak r2_sk; do
    if [[ -z "${!v}" ]]; then
      echo "ensure_cwasm_lambda: missing pulumi output (${v})" >&2
      return 1
    fi
  done

  CWASM_LAMBDA_REGION="$cwasm_region"

  local work_dir build_ctx
  work_dir="$(mktemp -d)"
  build_ctx="$(mktemp -d)"
  trap 'rm -rf "$work_dir" "$build_ctx"' RETURN

  local image_uri="${cwasm_ecr}:${new_fn0_wasmtime_version_dash}"
  local function_name="$CWASM_LAMBDA_FUNCTION_NAME"

  echo ">> ensure cwasm-compiler image ${image_uri}"
  (
    set -e
    export AWS_ACCESS_KEY_ID="$builder_ak"
    export AWS_SECRET_ACCESS_KEY="$builder_sk"
    export AWS_DEFAULT_REGION="$cwasm_region"
    unset AWS_SESSION_TOKEN AWS_PROFILE

    local ecr_registry="${cwasm_ecr%%/*}"
    aws ecr get-login-password --region "$cwasm_region" \
      | docker login --username AWS --password-stdin "$ecr_registry"

    "${REPO_ROOT}/scripts/build-rust-linux-arm64-bin.sh" fn0-wasmtime "$build_ctx"
    cp "${REPO_ROOT}/cwasm-compiler/package.json" "${REPO_ROOT}/cwasm-compiler/handler.mjs" "$build_ctx/"

    docker buildx build \
      --platform linux/arm64 \
      --provenance=false \
      --sbom=false \
      --file "${REPO_ROOT}/cwasm-compiler/Dockerfile" \
      --tag "$image_uri" \
      --push \
      "$build_ctx"

    local env_vars="Variables={BUCKET=${r2_bucket},XDG_CACHE_HOME=/tmp,HOME=/tmp,R2_ENDPOINT=${r2_endpoint},R2_ACCESS_KEY_ID=${r2_ak},R2_SECRET_ACCESS_KEY=${r2_sk}}"

    if aws lambda get-function --region "$cwasm_region" --function-name "$function_name" >/dev/null 2>&1; then
      echo ">> cwasm-compiler lambda exists; updating code + config"
      aws lambda update-function-code \
        --region "$cwasm_region" \
        --function-name "$function_name" \
        --image-uri "$image_uri" \
        >/dev/null
      local update_log="${work_dir}/lambda_update.log"
      until aws lambda update-function-configuration \
        --region "$cwasm_region" \
        --function-name "$function_name" \
        --environment "$env_vars" \
        >/dev/null 2>"$update_log"; do
        if ! grep -q "ResourceConflictException" "$update_log"; then
          cat "$update_log" >&2
          exit 1
        fi
        echo "   update-function-configuration conflict; retry..."
        sleep 5
      done
    else
      echo ">> creating cwasm-compiler lambda"
      aws lambda create-function \
        --region "$cwasm_region" \
        --function-name "$function_name" \
        --package-type Image \
        --code "ImageUri=${image_uri}" \
        --role "$cwasm_role" \
        --architectures arm64 \
        --timeout 60 \
        --memory-size 10240 \
        --environment "$env_vars" \
        >/dev/null
    fi

    aws lambda wait function-active --region "$cwasm_region" --function-name "$function_name"
    aws lambda wait function-updated --region "$cwasm_region" --function-name "$function_name"
  )
}

ensure_cwasm_pending() {
  local new_fn0_wasmtime_version="$1"
  if [[ -z "$new_fn0_wasmtime_version" ]]; then
    echo "ensure_cwasm_pending: missing version" >&2
    return 2
  fi

  ensure_cwasm_lambda "$new_fn0_wasmtime_version"

  local control_ak control_sk r2_endpoint r2_bucket r2_ak r2_sk
  control_ak="$(pulumi_pick controlAwsAccessKeyId)"
  control_sk="$(pulumi_pick controlAwsSecretAccessKey)"
  r2_endpoint="$(pulumi_pick bundleStoreR2Endpoint)"
  r2_bucket="$(pulumi_pick bundleStoreR2BucketName)"
  r2_ak="$(pulumi_pick bundleStoreR2AccessKeyId)"
  r2_sk="$(pulumi_pick bundleStoreR2SecretAccessKey)"

  echo ">> register pending fn0-wasmtime ${new_fn0_wasmtime_version} in control"
  control_set_pending_fn0_wasmtime "$new_fn0_wasmtime_version"

  local work_dir
  work_dir="$(mktemp -d)"
  trap 'rm -rf "$work_dir"' RETURN

  __cwasm_sync_compile_all "$new_fn0_wasmtime_version" "$CWASM_LAMBDA_FUNCTION_NAME" \
    "$CWASM_LAMBDA_REGION" "$control_ak" "$control_sk" \
    "$r2_endpoint" "$r2_bucket" "$r2_ak" "$r2_sk" \
    "$work_dir"
}


__cwasm_invoke_one() {
  local entry pid code_version input_key output_key payload_file out_file meta_file rc fn_err log_b64
  entry="$1"
  pid="$(jq -r '.project_id' <<<"$entry")"
  code_version="$(jq -r '.code_version' <<<"$entry")"
  input_key="original/${pid}/${code_version}.tar"
  output_key="compiled/${CWASM_INVOKE_NEW_FN0_WASMTIME_VERSION}/${pid}/${code_version}.tar.zst"

  payload_file="${CWASM_INVOKE_WORK_DIR}/payload.${pid}.${code_version}.json"
  out_file="${CWASM_INVOKE_WORK_DIR}/out.${pid}.${code_version}.json"
  meta_file="${CWASM_INVOKE_WORK_DIR}/meta.${pid}.${code_version}.json"

  jq -nc --arg ib "$CWASM_INVOKE_R2_BUCKET" --arg ik "$input_key" \
         --arg ob "$CWASM_INVOKE_R2_BUCKET" --arg ok "$output_key" \
         '{input_bucket: $ib, input_key: $ik, output_bucket: $ob, output_key: $ok, env_bucket: null, env_key: null}' \
    > "$payload_file"

  rc=0
  AWS_ACCESS_KEY_ID="$CWASM_INVOKE_CONTROL_AK" \
  AWS_SECRET_ACCESS_KEY="$CWASM_INVOKE_CONTROL_SK" \
  AWS_DEFAULT_REGION="$CWASM_INVOKE_REGION" \
  aws lambda invoke \
    --function-name "$CWASM_INVOKE_FUNCTION_NAME" \
    --invocation-type RequestResponse \
    --log-type Tail \
    --cli-binary-format raw-in-base64-out \
    --payload "file://${payload_file}" \
    "$out_file" \
    > "$meta_file" 2>&1 || rc=$?

  if [[ $rc -ne 0 ]]; then
    {
      echo "[FAIL] ${pid} ${code_version}: aws cli rc=${rc}"
      sed 's/^/  /' < "$meta_file"
      echo "---"
    } >> "$CWASM_INVOKE_FAIL_FILE"
    return
  fi

  fn_err="$(jq -r '.FunctionError // empty' < "$meta_file" 2>/dev/null || true)"
  log_b64="$(jq -r '.LogResult // empty' < "$meta_file" 2>/dev/null || true)"

  if [[ -n "$fn_err" ]]; then
    {
      echo "[FAIL] ${pid} ${code_version}: FunctionError=${fn_err}"
      echo "  payload:"
      sed 's/^/    /' < "$out_file"
      if [[ -n "$log_b64" ]]; then
        echo "  log tail:"
        echo "$log_b64" | base64 -d | sed 's/^/    /'
      fi
      echo "---"
    } >> "$CWASM_INVOKE_FAIL_FILE"
  else
    echo "${pid} ${code_version}" >> "$CWASM_INVOKE_SUCCESS_FILE"
  fi
}
export -f __cwasm_invoke_one

__cwasm_sync_compile_all() {
  local new_fn0_wasmtime_version="$1" function_name="$2" region="$3" control_ak="$4" control_sk="$5"
  local r2_endpoint="$6" r2_bucket="$7" r2_ak="$8" r2_sk="$9"
  local work_dir="${10}"

  local originals_file="${work_dir}/originals.jsonl"
  local compiled_file="${work_dir}/compiled.txt"
  local todo_file="${work_dir}/todo.jsonl"
  local success_file="${work_dir}/success.txt"
  local fail_file="${work_dir}/fail.txt"
  : > "$originals_file"
  : > "$compiled_file"
  : > "$todo_file"
  : > "$success_file"
  : > "$fail_file"

  r2_list_objects "$r2_endpoint" "$r2_bucket" "$r2_ak" "$r2_sk" "original/" \
    | jq -c 'select(.Key | test("^original/[^/]+/[0-9]+\\.tar$")) | (.Key | capture("^original/(?<p>[^/]+)/(?<cv>[0-9]+)\\.tar$")) | {project_id: .p, code_version: (.cv | tonumber)}' \
    > "$originals_file" || true

  r2_list_objects "$r2_endpoint" "$r2_bucket" "$r2_ak" "$r2_sk" "compiled/${new_fn0_wasmtime_version}/" \
    | jq -r 'select(.Key | test("^compiled/[^/]+/[^/]+/[0-9]+\\.tar\\.zst$")) | (.Key | capture("^compiled/[^/]+/(?<p>[^/]+)/(?<cv>[0-9]+)\\.tar\\.zst$") | "\(.p)|\(.cv)")' \
    > "$compiled_file" || true

  local line pid code_version key
  while IFS= read -r line; do
    pid="$(jq -r '.project_id' <<<"$line")"
    code_version="$(jq -r '.code_version' <<<"$line")"
    key="${pid}|${code_version}"
    if grep -Fxq "$key" "$compiled_file"; then
      continue
    fi
    jq -nc --arg p "$pid" --argjson cv "$code_version" '{project_id: $p, code_version: $cv}' >> "$todo_file"
  done < "$originals_file"

  local total todo_count already
  total="$(wc -l < "$originals_file" | tr -d ' ')"
  todo_count="$(wc -l < "$todo_file" | tr -d ' ')"
  already=$((total - todo_count))

  echo ">> originals=${total} already_compiled=${already} todo=${todo_count}"

  if [[ "$todo_count" -gt 0 ]]; then
    echo ">> invoking lambda for ${todo_count} bundle(s) (parallel=${CWASM_COMPILER_PARALLEL})"
    CWASM_INVOKE_NEW_FN0_WASMTIME_VERSION="$new_fn0_wasmtime_version" \
    CWASM_INVOKE_WORK_DIR="$work_dir" \
    CWASM_INVOKE_R2_BUCKET="$r2_bucket" \
    CWASM_INVOKE_FUNCTION_NAME="$function_name" \
    CWASM_INVOKE_CONTROL_AK="$control_ak" \
    CWASM_INVOKE_CONTROL_SK="$control_sk" \
    CWASM_INVOKE_REGION="$region" \
    CWASM_INVOKE_SUCCESS_FILE="$success_file" \
    CWASM_INVOKE_FAIL_FILE="$fail_file" \
    xargs -I {} -P "$CWASM_COMPILER_PARALLEL" \
      bash -c '__cwasm_invoke_one "$@"' _ {} < "$todo_file"
  fi

  local success_count fail_count total_success
  success_count="$(wc -l < "$success_file" | tr -d ' ')"
  fail_count="$(grep -c '^\[FAIL\]' "$fail_file" || true)"
  total_success=$((already + success_count))

  echo ">> success(this run)=${success_count} fail=${fail_count} total_success=${total_success}/${total}"

  if [[ "$fail_count" -gt 0 ]]; then
    echo "" >&2
    echo "=== cwasm compile failures ===" >&2
    cat "$fail_file" >&2
    echo ">> aborting; pending stays as-is. rerun after addressing failures." >&2
    return 1
  fi
  if [[ "$total_success" -lt "$total" ]]; then
    echo ">> incomplete coverage (${total_success}/${total}); rerun." >&2
    return 1
  fi
  echo ">> cwasm full coverage gated; pending ${new_fn0_wasmtime_version} ready."
}
