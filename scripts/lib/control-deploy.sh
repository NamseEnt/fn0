# shellcheck shell=bash
# fn0-control deploy helpers (R2 upload + cwasm-compiler invoke). Source-only.
#
# upload_r2_original <project_id> <code_version> <local_tar_path>
# compile_via_cwasm <project_id> <code_version> <wasmtime_version> <lambda_function_name>

if [[ -n "${__FN0_CONTROL_DEPLOY_LOADED:-}" ]]; then
  return 0
fi
__FN0_CONTROL_DEPLOY_LOADED=1

upload_r2_original() {
  local project_id="$1" code_version="$2" local_tar="$3"
  if [[ -z "$project_id" || -z "$code_version" || -z "$local_tar" ]]; then
    echo "upload_r2_original: missing args" >&2
    return 2
  fi
  if [[ ! -f "$local_tar" ]]; then
    echo "upload_r2_original: tar not found: ${local_tar}" >&2
    return 1
  fi
  local r2_endpoint r2_bucket r2_ak r2_sk
  r2_endpoint="$(pulumi_pick bundleStoreR2Endpoint)"
  r2_bucket="$(pulumi_pick bundleStoreR2BucketName)"
  r2_ak="$(pulumi_pick bundleStoreR2AccessKeyId)"
  r2_sk="$(pulumi_pick bundleStoreR2SecretAccessKey)"
  for v in r2_endpoint r2_bucket r2_ak r2_sk; do
    if [[ -z "${!v}" ]]; then
      echo "upload_r2_original: missing pulumi output (${v})" >&2
      return 1
    fi
  done
  echo ">> R2 upload original/${project_id}/${code_version}.tar"
  AWS_ACCESS_KEY_ID="$r2_ak" \
  AWS_SECRET_ACCESS_KEY="$r2_sk" \
  AWS_DEFAULT_REGION=auto \
  aws s3api put-object \
    --endpoint-url "$r2_endpoint" \
    --bucket "$r2_bucket" \
    --key "original/${project_id}/${code_version}.tar" \
    --body "$local_tar" \
    >/dev/null
}

compile_via_cwasm() {
  local project_id="$1" code_version="$2" wasmtime="$3" function_name="$4"
  if [[ -z "$project_id" || -z "$code_version" || -z "$wasmtime" || -z "$function_name" ]]; then
    echo "compile_via_cwasm: missing args" >&2
    return 2
  fi
  local cwasm_region control_ak control_sk r2_bucket
  cwasm_region="$(pulumi_pick cwasmCompilerBucketRegion)"
  control_ak="$(pulumi_pick controlAwsAccessKeyId)"
  control_sk="$(pulumi_pick controlAwsSecretAccessKey)"
  r2_bucket="$(pulumi_pick bundleStoreR2BucketName)"
  for v in cwasm_region control_ak control_sk r2_bucket; do
    if [[ -z "${!v}" ]]; then
      echo "compile_via_cwasm: missing pulumi output (${v})" >&2
      return 1
    fi
  done

  local input_key="original/${project_id}/${code_version}.tar"
  local output_key="compiled/${wasmtime}/${project_id}/${code_version}.tar.zst"
  local payload_file out_file meta_file
  payload_file="$(mktemp)"
  out_file="$(mktemp)"
  meta_file="$(mktemp)"
  # shellcheck disable=SC2064
  trap "rm -f '$payload_file' '$out_file' '$meta_file'" RETURN

  jq -nc \
    --arg ib "$r2_bucket" --arg ik "$input_key" \
    --arg ob "$r2_bucket" --arg ok "$output_key" \
    '{input_bucket:$ib, input_key:$ik, output_bucket:$ob, output_key:$ok, env_bucket:null, env_key:null}' \
    > "$payload_file"

  echo ">> cwasm-compiler invoke (${project_id}, code_version=${code_version}, wasmtime=${wasmtime})"
  local rc=0
  AWS_ACCESS_KEY_ID="$control_ak" \
  AWS_SECRET_ACCESS_KEY="$control_sk" \
  AWS_DEFAULT_REGION="$cwasm_region" \
  aws lambda invoke \
    --function-name "$function_name" \
    --invocation-type RequestResponse \
    --log-type Tail \
    --cli-binary-format raw-in-base64-out \
    --payload "file://${payload_file}" \
    "$out_file" \
    > "$meta_file" 2>&1 || rc=$?

  if [[ $rc -ne 0 ]]; then
    echo "lambda invoke failed (rc=${rc}):" >&2
    cat "$meta_file" >&2
    return $rc
  fi

  local fn_err log_b64
  fn_err="$(jq -r '.FunctionError // empty' <"$meta_file" 2>/dev/null || true)"
  log_b64="$(jq -r '.LogResult // empty' <"$meta_file" 2>/dev/null || true)"
  if [[ -n "$fn_err" ]]; then
    echo "cwasm-compiler FunctionError=${fn_err}:" >&2
    echo "  payload:" >&2
    sed 's/^/    /' <"$out_file" >&2
    if [[ -n "$log_b64" ]]; then
      echo "  log tail:" >&2
      echo "$log_b64" | base64 -d | sed 's/^/    /' >&2
    fi
    return 1
  fi
  echo "   compiled -> ${output_key}"
}
