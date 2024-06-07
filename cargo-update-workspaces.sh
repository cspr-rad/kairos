#!/usr/bin/env sh

# Set the path to the first argument, defaulting to the current directory if no argument is provided
path="${1:-.}"

# Find and update Cargo.lock files starting from the specified path or the default path
find "$path" -name Cargo.lock -execdir cargo update --verbose \;
