EXT := if os() == "windows" { ".exe" } else { "" }
CWD := invocation_directory()
BIN_NAME := "rokit"

# Builds the Rokit CLI binary
[no-exit-message]
build *ARGS:
	#!/usr/bin/env bash
	set -euo pipefail
	cargo build --bin {{BIN_NAME}} {{ARGS}}

# Used in GitHub workflow to zip up the built binary
[no-exit-message]
zip-release TARGET_TRIPLE:
	#!/usr/bin/env bash
	set -euo pipefail
	rm -rf staging
	rm -rf release.zip
	mkdir -p staging
	cp "target/{{TARGET_TRIPLE}}/release/{{BIN_NAME}}{{EXT}}" staging/
	cd staging
	if [ "{{os_family()}}" = "windows" ]; then
		7z a ../release.zip *
	else
		chmod +x {{BIN_NAME}}
		zip ../release.zip *
	fi
	cd "{{CWD}}"
	rm -rf staging

# Used in GitHub workflow to move per-matrix release zips
[no-exit-message]
unpack-releases RELEASES_DIR:
	#!/usr/bin/env bash
	set -euo pipefail
	#
	if [ ! -d "{{RELEASES_DIR}}" ]; then
		echo "Releases directory is missing"
		exit 1
	fi
	#
	cd "{{RELEASES_DIR}}"
	echo ""
	echo "Releases dir:"
	ls -lhrt
	echo ""
	echo "Searching for zipped releases..."
	#
	for DIR in * ; do
		if [ -d "$DIR" ]; then
			cd "$DIR"
			for FILE in * ; do
				if [ ! -d "$FILE" ]; then
					if [ "$FILE" = "release.zip" ]; then
						echo "Found zipped release '$DIR'"
						mv "$FILE" "../$DIR.zip"
						rm -rf "../$DIR/"
					fi
				fi
			done
			cd ..
		fi
	done
	#
	echo ""
	echo "Releases dir:"
	ls -lhrt
