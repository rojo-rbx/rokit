Set-Location "$env:temp"

$LatestReleaseUri = "https://github.com/rojo-rbx/rokit/releases/latest/"

# Function to get the (redirected) versioned URL from the latest release endpoint
# Courtesy of https://www.reddit.com/r/PowerShell/comments/5d516e/comment/da20wf0/
Function Get-RedirectedUrl {
	Param (
		[Parameter(Mandatory = $true)]
		[String]$Uri
	)

	try {
		$request = [System.Net.WebRequest]::Create($Uri)
		$request.AllowAutoRedirect = $false
		$response = $request.GetResponse()

		If (($response.StatusCode -eq "Found") -or ($response.StatusCode -eq "MovedPermanently")) {
			$RedirectUrl = $response.GetResponseHeader("Location")
		}

		$response.Close()
		if ($RedirectUrl) { return(Write-Output $RedirectUrl) }
		Else { 
			Write-Error "No redirect URL found at $Uri"
			return $null
		}
	}
	catch {
		Write-Error "Failed to get redirected URL: $_"
		return $null
	}
}

try {
	Write-Host "Detecting latest version..." -ForegroundColor Cyan
	$LatestTaggedReleaseUri = Get-RedirectedUrl -Uri $LatestReleaseUri
    
	if (-not $LatestTaggedReleaseUri) {
		throw "Failed to detect latest version. Could not get redirect from $LatestReleaseUri"
	}

	# Extract version tag from the redirect URL (preserving the v prefix for download URL)
	$VersionTagMatch = $LatestTaggedReleaseUri | Select-String -Pattern '\/tag\/(v?.+)$'
	if (-not $VersionTagMatch.Matches) {
		throw "Could not parse version tag from URL: $LatestTaggedReleaseUri"
	}
    
	$VersionTag = $VersionTagMatch.Matches.Groups[1].Value
	$NumericVersion = $VersionTag -replace '^v', ''

	Write-Host "Latest version detected: $NumericVersion (tag: $VersionTag)" -ForegroundColor Green

	# Construct the download URL - keep the v prefix if it exists in the tag
	$DownloadUrl = "https://github.com/rojo-rbx/rokit/releases/download/$VersionTag/rokit-$NumericVersion-windows-x86_64.zip"
	Write-Host "Downloading from: $DownloadUrl" -ForegroundColor Cyan

	# Grab the latest release asset
	try {
		Invoke-WebRequest $DownloadUrl -OutFile rokit.zip -ErrorAction Stop
		Write-Host "Download successful" -ForegroundColor Green
	}
	catch {
		throw "Failed to download from $DownloadUrl`: $_"
	}

	# Extract the archive
	try {
		Expand-Archive -Path rokit.zip -Force -ErrorAction Stop
		Write-Host "Extraction successful" -ForegroundColor Green
	}
	catch {
		throw "Failed to extract rokit.zip: $_"
	}

	# Run self-install
	try {
		if (Test-Path ".\rokit\rokit.exe") {
			Write-Host "Installing rokit..." -ForegroundColor Cyan
			Start-Process -FilePath ".\rokit\rokit.exe" -ArgumentList "self-install" -Wait -NoNewWindow
			Write-Host "Installation successful" -ForegroundColor Green
		}
		else {
			throw "rokit.exe not found in the extracted directory"
		}
	}
	catch {
		throw "Failed to run self-install: $_"
	}

	# Prevent "rokit.exe" is in use error
	Start-Sleep -Seconds 1

	# Cleanup
	try {
		Remove-Item rokit.zip -ErrorAction SilentlyContinue
		Remove-Item -Recurse -Path .\rokit -ErrorAction SilentlyContinue
		Write-Host "Cleanup completed" -ForegroundColor Green
	}
	catch {
		Write-Warning "Cleanup failed: $_"
	}

	Write-Host "Rokit installation completed successfully!" -ForegroundColor Green
}
catch {
	Write-Error "Installation failed: $_"
	exit 1
}
