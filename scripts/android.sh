#!/bin/sh
# Builds the debug APK and installs it on the running emulator or a USB phone.
# NDK r30 ships only llvm-ar and llvm-ranlib; vendored OpenSSL looks for the prefixed names.
set -e
export ANDROID_HOME="${ANDROID_HOME:-$HOME/Library/Android/sdk}"
export NDK_HOME="${NDK_HOME:-$(ls -d "$ANDROID_HOME"/ndk/* | sort -V | tail -1)}"
export JAVA_HOME="${JAVA_HOME:-/opt/homebrew/opt/openjdk}"
bin="$NDK_HOME/toolchains/llvm/prebuilt/darwin-x86_64/bin"
export AR_aarch64_linux_android="$bin/llvm-ar"
export RANLIB_aarch64_linux_android="$bin/llvm-ranlib"
bun tauri android build --debug --apk --target aarch64
"$ANDROID_HOME/platform-tools/adb" install -r src-tauri/gen/android/app/build/outputs/apk/universal/debug/app-universal-debug.apk
