#!/usr/bin/env sh

# Determine the path from the first or last argument
if [ -d "$1" ]; then
  # First argument is a directory
  path="$1"
  shift  # Remove the first argument
elif [ -d "${@: -1}" ]; then
  # Last argument is a directory
  path="${@: -1}"
  set -- "${@:1:$#-1}"  # Remove the last argument
else
  # Default to current directory if no directory is provided
  path="."
fi

# Remaining arguments are assumed to be crate names
crates="$@"

# Find and execute cargo update for the Cargo.lock files
find "$path" -name Cargo.lock -execdir cargo update --verbose $crates \;
