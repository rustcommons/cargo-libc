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
    local public_key=$1 fingerprint=$2 dist=$3 home status primary
    home=$(mktemp -d)
    trap 'cleanup_home "$home"' EXIT
    chmod 700 "$home"
    gpg --homedir "$home" --batch --import "$public_key" >/dev/null 2>&1
    primary=$(gpg --homedir "$home" --batch --with-colons --list-keys |
        awk -F: '$1 == "pub" { primary=1; next } primary && $1 == "fpr" { print $10; primary=0 }')
    test "$primary" = "$fingerprint"
    status=$(gpg --homedir "$home" --batch --no-auto-key-retrieve --no-auto-key-import --status-fd 1 \
        --verify "$dist/SHA256SUMS.asc" "$dist/SHA256SUMS")
    # VALIDSIG's final field is the primary fingerprint, including subkey signatures.
    awk -v pin="$fingerprint" '$1 == "[GNUPG:]" && $2 == "VALIDSIG" && $NF == pin { valid=1 } END { exit !valid }' <<<"$status"
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
