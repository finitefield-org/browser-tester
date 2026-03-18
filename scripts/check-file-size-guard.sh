#!/usr/bin/env bash

set -euo pipefail

warn_lines="${BROWSER_TESTER_FILE_WARN_LINES:-800}"
fail_lines="${BROWSER_TESTER_FILE_FAIL_LINES:-1500}"
allowlist_file="${BROWSER_TESTER_FILE_SIZE_ALLOWLIST:-doc/file-size-guard-allowlist.txt}"

if [[ ! -f "$allowlist_file" ]]; then
  echo "[file-size-guard] missing allowlist file: $allowlist_file" >&2
  exit 1
fi

declare -a stale_allowlist=()
declare -a prod_warn=()
declare -a prod_allowlisted_fail=()
declare -a prod_violations=()
declare -a test_watch=()

line_count() {
  local path="$1"
  wc -l < "$path" | tr -d '[:space:]'
}

while IFS= read -r path; do
  count="$(line_count "$path")"
  if (( count > warn_lines )); then
    prod_warn+=("$(printf '%6d %s' "$count" "$path")")
  fi
  if (( count > fail_lines )); then
    if grep -Fqx "$path" "$allowlist_file"; then
      prod_allowlisted_fail+=("$(printf '%6d %s' "$count" "$path")")
    else
      prod_violations+=("$(printf '%6d %s' "$count" "$path")")
    fi
  fi
done < <(find src -type f -name '*.rs' ! -path 'src/tests/*' | sort)

while IFS= read -r path; do
  count="$(line_count "$path")"
  if (( count > warn_lines )); then
    test_watch+=("$(printf '%6d %s' "$count" "$path")")
  fi
done < <(find src/tests tests -type f -name '*.rs' | sort)

while IFS= read -r path; do
  [[ -z "$path" || "$path" == \#* ]] && continue
  if [[ ! -f "$path" ]]; then
    stale_allowlist+=("$path (missing)")
    continue
  fi
  count="$(line_count "$path")"
  if (( count <= fail_lines )); then
    stale_allowlist+=("$path ($count lines)")
  fi
done < "$allowlist_file"

print_numeric_section() {
  local title="$1"
  shift
  local -a entries=("$@")
  [[ ${#entries[@]} -eq 0 ]] && return 0
  echo
  echo "$title"
  printf '%s\n' "${entries[@]}" | sort -nr
}

print_text_section() {
  local title="$1"
  shift
  local -a entries=("$@")
  [[ ${#entries[@]} -eq 0 ]] && return 0
  echo
  echo "$title"
  printf '%s\n' "${entries[@]}" | sort
}

echo "[file-size-guard] production warn threshold: $warn_lines"
echo "[file-size-guard] production fail threshold: $fail_lines"
echo "[file-size-guard] test-suite files are watch-only for now"

print_numeric_section "[file-size-guard] production files over warn threshold:" "${prod_warn[@]}"
print_numeric_section "[file-size-guard] allowlisted production files over fail threshold:" "${prod_allowlisted_fail[@]}"
print_numeric_section "[file-size-guard] test-suite watch list over warn threshold:" "${test_watch[@]}"

if (( ${#stale_allowlist[@]} > 0 )); then
  print_text_section "[file-size-guard] stale allowlist entries to remove or refresh:" "${stale_allowlist[@]}"
fi

if (( ${#prod_violations[@]} > 0 )); then
  print_numeric_section "[file-size-guard] production files over fail threshold without allowlist entry:" "${prod_violations[@]}"
  exit 1
fi

if (( ${#stale_allowlist[@]} > 0 )); then
  exit 1
fi

echo
echo "[file-size-guard] ok"
