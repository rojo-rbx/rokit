$PROGRAM_NAME = "rokit"
$REPOSITORY = "rojo-rbx/rokit"

$originalPath = Get-Location

Set-Location "$env:temp"

# Function to get release info from GitHub API
Function Get-ReleaseInfo {
	param (
		[string]$ApiUrl
	)
    
	$headers = @{
		'X-GitHub-Api-Version' = '2022-11-28'
	}
    
	if ($env:GITHUB_PAT) {
		$headers['Authorization'] = "token $env:GITHUB_PAT"
	}
    
	try {
		$response = Invoke-RestMethod -Uri $ApiUrl -Headers $headers -ErrorAction Stop
		return $response
	}
	catch {
		throw "Failed to fetch release info: $_"
	}
}

try {
	if ($env:GITHUB_PAT) {
		Write-Host "NOTE: Using provided GITHUB_PAT for authentication"
	}
 
	Write-Host "`n[1 / 3] Looking for latest $PROGRAM_NAME release"
    
	$apiUrl = "https://api.github.com/repos/$REPOSITORY/releases/latest"
	$releaseInfo = Get-ReleaseInfo -ApiUrl $apiUrl
    
	$versionTag = $releaseInfo.tag_name
	$numericVersion = $versionTag -replace '^v', ''
    
	# Construct the download URL
	$downloadUrl = "https://github.com/$REPOSITORY/releases/download/$versionTag/$PROGRAM_NAME-$numericVersion-windows-x86_64.zip"
	Write-Host "[2 / 3] Downloading '$PROGRAM_NAME-$numericVersion-windows-x86_64.zip'"

	# Grab the latest release asset
	try {
		Invoke-WebRequest $downloadUrl -OutFile rokit.zip -ErrorAction Stop
	}
	catch {
		throw "Failed to download from $downloadUrl`: $_"
	}

	# Extract the archive
	try {
		Expand-Archive -Path rokit.zip -Force -ErrorAction Stop
	}
	catch {
		throw "Failed to extract rokit.zip: $_"
	}

	# Run self-install
	try {
		if (Test-Path ".\rokit\rokit.exe") {
			Write-Host "[3 / 3] Running $PROGRAM_NAME installation`n"
			Start-Process -FilePath ".\rokit\rokit.exe" -ArgumentList "self-install" -Wait -NoNewWindow
		}
		else {
			throw "rokit.exe not found in the extracted directory"
		}
	}
	catch {
		throw "Failed to run self-install: $_"
	}

	# Cleanup
	try {
		Remove-Item rokit.zip -ErrorAction SilentlyContinue
		Remove-Item -Recurse -Path .\rokit -ErrorAction SilentlyContinue
	}
	catch {
		Write-Warning "Cleanup failed: $_"
	}
}
catch {
	Write-Error "Installation failed: $_"
	exit 1
}
finally {
	# Return to original directory
	Set-Location -Path $originalPath
}
