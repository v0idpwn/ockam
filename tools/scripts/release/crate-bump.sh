#!/usr/bin/env bash

# This script bumps all crates that have been updated compared to
# last git tag. RELEASE_VERSION value is to be set to indicate the
# release version of all crates (usually minor). If there are crates
# that are not to follow the RELEASE_VERSION value, we can further
# set MODIFIED_RELEASE value to indicates individual crates and how
# they are to be bumped "signature_core:minor ockam:major" signature_core
# crate will be bumped as a minor and ockam crate will be bumped as
# major.

if [[ -z $RELEASE_VERSION ]]; then
    echo "please set RELEASE_VERSION variable"
    exit 1
fi

source tools/scripts/release/crates-to-publish.sh

declare -A specified_crate_version

crate_array=($MODIFIED_RELEASE)

for word in ${crate_array[@]}; do
    key="${word%%:*}"
    value="${word##*:}"
    specified_crate_version[$key]=$value
done

for to_update in ${updated_crates[@]}; do
    version=$RELEASE_VERSION

    if [[ ! -z "${specified_crate_version[$to_update]}" ]]; then
        echo "bumping $to_update as ${specified_crate_version[$to_update]}"
        version="${specified_crate_version[$to_update]}"
    fi

    echo y | cargo release $version --no-push --no-publish --no-tag --no-dev-version --package $to_update --execute
done
