SHELL := /bin/bash

PROJECT := swiftyapp/kubo-macos.xcodeproj
SCHEME := kubo-macos
BUILD_SCRIPT := ./build.sh
RUST_CRATE_DIR := rustylib
DERIVED_DATA ?= .build/xcode
CONFIGURATION ?= Debug
APP_NAME := kubo-macos.app
INSTALL_DIR ?= $(HOME)/Applications
SYMROOT := $(abspath $(DERIVED_DATA))
OBJROOT := $(SYMROOT)/Intermediates.noindex
HOST_ARCH := $(shell uname -m)
SIMULATOR_ARCH := $(if $(filter arm64,$(HOST_ARCH)),arm64,x86_64)
IOS_DESTINATION ?= generic/platform=iOS Simulator
MAC_DESTINATION ?= platform=macOS,variant=Designed for iPad,name=My Mac
MAC_APP_PATH := $(SYMROOT)/$(CONFIGURATION)-iphoneos/$(APP_NAME)

.DEFAULT_GOAL := help

.PHONY: help rust resolve app mac install clean

help:
	@printf '%s\n' \
		'make rust      - build the Rust library, bindings, and XCFramework' \
		'make resolve   - resolve local Swift package dependencies' \
		'make app       - build the iOS app for a generic simulator destination' \
		'make mac       - build the app for macOS (Designed for iPad)' \
		'make install   - install the macOS (Designed for iPad) app into ~/Applications' \
		'make clean     - clean Rust and Xcode build artifacts'

rust:
	$(BUILD_SCRIPT)

resolve:
	xcodebuild -resolvePackageDependencies -project "$(PROJECT)" -scheme "$(SCHEME)"

app: rust resolve
	xcodebuild \
		-project "$(PROJECT)" \
		-scheme "$(SCHEME)" \
		-configuration "$(CONFIGURATION)" \
		-derivedDataPath "$(DERIVED_DATA)" \
		SYMROOT="$(SYMROOT)" \
		OBJROOT="$(OBJROOT)" \
		-destination "$(IOS_DESTINATION)" \
		ARCHS="$(SIMULATOR_ARCH)" \
		ONLY_ACTIVE_ARCH=YES \
		build

mac: rust resolve
	xcodebuild \
		-project "$(PROJECT)" \
		-scheme "$(SCHEME)" \
		-configuration "$(CONFIGURATION)" \
		-derivedDataPath "$(DERIVED_DATA)" \
		SYMROOT="$(SYMROOT)" \
		OBJROOT="$(OBJROOT)" \
		-destination "$(MAC_DESTINATION)" \
		ARCHS="$(HOST_ARCH)" \
		ONLY_ACTIVE_ARCH=YES \
		build

install: mac
	mkdir -p "$(INSTALL_DIR)"
	rm -rf "$(INSTALL_DIR)/$(APP_NAME)"
	cp -R "$(MAC_APP_PATH)" "$(INSTALL_DIR)/$(APP_NAME)"
	/System/Library/Frameworks/CoreServices.framework/Versions/A/Frameworks/LaunchServices.framework/Versions/A/Support/lsregister -f "$(INSTALL_DIR)/$(APP_NAME)"
	@echo ""
	@echo "Installed $(APP_NAME) to $(INSTALL_DIR)"
	@echo ""
	@echo "NOTE: This is a 'Designed for iPad' app (iOS binary). macOS Finder cannot"
	@echo "launch iOS apps directly. To run it:"
	@echo "  1. Find it in Launchpad and click it there."
	@echo "  2. Or open the project in Xcode and press Run."
	@echo "  3. Or run: make mac"
	@echo ""

clean:
	cd "$(RUST_CRATE_DIR)" && cargo clean
	rm -rf "$(DERIVED_DATA)"