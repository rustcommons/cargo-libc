#!/usr/bin/env bash
# Production entry points never accept a key or fingerprint override.
set -euo pipefail
set +x
readonly RELEASE_PRIMARY_FINGERPRINT=B281C3010A76A11737B4D066C7396F6EA7647CC6
readonly RELEASE_SIGNING_SUBKEY=248DFD639B75D1EFF2A87289D7C7DE7BBCCF35EC
RELEASE_PUBLIC_KEY="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)/.github/release-signing-key.asc"
readonly RELEASE_PUBLIC_KEY

cleanup_home() {
    local result=$?
    gpgconf --homedir "$1" --kill gpg-agent || result=1
    rm -rf -- "$1"
    return "$result"
}

# Internal helpers also exercise the same operations with throwaway test keys.
verify_with_key() (
    set -euo pipefail
    local public_key=$1 fingerprint=$2 dist=$3 home status primary bad sigkey sigtime keyexp good valids valid
    home=$(mktemp -d)
    trap 'cleanup_home "$home"' EXIT
    chmod 700 "$home"
    gpg --homedir "$home" --batch --import "$public_key" >/dev/null 2>&1
    primary=$(gpg --homedir "$home" --batch --with-colons --list-keys |
        awk -F: '$1 == "pub" { primary=1; next } primary && $1 == "fpr" { print $10; primary=0 }')
    test "$primary" = "$fingerprint"
    status=$(gpg --homedir "$home" --batch --no-auto-key-retrieve --no-auto-key-import --status-fd 1 \
        --verify "$dist/SHA256SUMS.asc" "$dist/SHA256SUMS" || true)
    # Match rustcommons/.github tools/release-signing/ci-verify.sh (PR #27).
    # VALIDSIG alone also occurs for revoked or expired signing keys.
    bad=$(awk '$1=="[GNUPG:]" && $2 ~ /^(REVKEYSIG|KEYREVOKED|EXPSIG|BADSIG|ERRSIG|NO_PUBKEY)$/ {print $2}' <<<"$status")
    if [[ -n $bad ]]; then
        echo "Signature refused: $bad" >&2
        exit 1
    fi
    # KEYEXPIRED may describe a sibling subkey rather than this signature.
    if awk '$1=="[GNUPG:]" && $2=="EXPKEYSIG" {expired=1} END {exit !expired}' <<<"$status"; then
        sigkey=$(awk '$1=="[GNUPG:]" && $2=="VALIDSIG" {print $3; exit}' <<<"$status")
        sigtime=$(awk '$1=="[GNUPG:]" && $2=="VALIDSIG" {print $5; exit}' <<<"$status")
        keyexp=$(gpg --homedir "$home" --batch --with-colons --list-keys |
            awk -F: -v k="$sigkey" '($1=="pub" || $1=="sub") {e=$7} $1=="fpr" && $10==k {print e; exit}')
        if [[ ! $sigtime =~ ^[0-9]+$ || ! $keyexp =~ ^[0-9]+$ ]] || (( sigtime >= keyexp )); then
            echo 'Signature refused: signing time is not before signing-key expiry' >&2
            exit 1
        fi
    fi
    good=$(awk '$1=="[GNUPG:]" && ($2=="GOODSIG" || $2=="EXPKEYSIG") {n++} END {print n+0}' <<<"$status")
    valids=$(awk '$1=="[GNUPG:]" && $2=="VALIDSIG" {n++} END {print n+0}' <<<"$status")
    [[ $good == 1 && $valids == 1 ]]
    # VALIDSIG's final field identifies the primary key, including subkey signatures.
    valid=$(awk '$1=="[GNUPG:]" && $2=="VALIDSIG" {print $NF}' <<<"$status")
    [[ $valid == "$fingerprint" ]]
    (cd -- "$dist" && sha256sum --check --strict SHA256SUMS)
)

sign_with_key() (
    set -euo pipefail
    local public_key=$1 fingerprint=$2 subkey=$3 dist=$4 home private_key passphrase
    private_key=${RELEASE_SIGNING_KEY:?missing signing key}
    passphrase=${RELEASE_SIGNING_PASSPHRASE:?missing signing passphrase}
    unset RELEASE_SIGNING_KEY RELEASE_SIGNING_PASSPHRASE
    home=$(mktemp -d)
    trap 'cleanup_home "$home"' EXIT
    chmod 700 "$home"
    printf '%s' "$private_key" | gpg --homedir "$home" --batch --import >/dev/null 2>&1
    unset private_key
    printf '%s' "$passphrase" | gpg --homedir "$home" --batch --pinentry-mode loopback \
        --passphrase-fd 0 --local-user "${subkey}!" --armor --detach-sign \
        --output "$dist/SHA256SUMS.asc" "$dist/SHA256SUMS"
    unset passphrase
    verify_with_key "$public_key" "$fingerprint" "$dist"
)

if [[ ${BASH_SOURCE[0]} == "$0" ]]; then
    case ${1:-} in
        sign) sign_with_key "$RELEASE_PUBLIC_KEY" "$RELEASE_PRIMARY_FINGERPRINT" "$RELEASE_SIGNING_SUBKEY" "${2:?asset directory required}" ;;
        verify) verify_with_key "$RELEASE_PUBLIC_KEY" "$RELEASE_PRIMARY_FINGERPRINT" "${2:?asset directory required}" ;;
        *) echo 'Usage: release-signing.sh {sign|verify} ASSET_DIRECTORY' >&2; exit 2 ;;
    esac
fi
