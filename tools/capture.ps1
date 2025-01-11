# Screenshot capture utility for Tinker development
param(
    [Parameter(Mandatory=$false)]
    [ValidateSet('window', 'region', 'full')]
    [string]$type = 'window',
    
    [Parameter(Mandatory=$false)]
    [string]$name = (Get-Date -Format "yyyy-MM-dd-HH-mm-ss")
)

$greenshotPath = "C:\Program Files\Greenshot\Greenshot.exe"
$screenshotsDir = "..\screenshots"

# Check if Greenshot exists
if (-not (Test-Path $greenshotPath)) {
    Write-Error "Greenshot not found at $greenshotPath"
    exit 1
}

# Create screenshots directory if it doesn't exist
if (-not (Test-Path $screenshotsDir)) {
    New-Item -ItemType Directory -Path $screenshotsDir | Out-Null
}

# Build the screenshot path
$screenshotPath = Join-Path $screenshotsDir "$name.png"

# Capture screenshot based on type
switch ($type) {
    'window' {
        Start-Process -FilePath $greenshotPath -ArgumentList "/capture=window /savepath=$screenshotPath" -Wait
    }
    'region' {
        Start-Process -FilePath $greenshotPath -ArgumentList "/capture=region /savepath=$screenshotPath" -Wait
    }
    'full' {
        Start-Process -FilePath $greenshotPath -ArgumentList "/capture=screen /savepath=$screenshotPath" -Wait
    }
}

if (Test-Path $screenshotPath) {
    Write-Host "Screenshot saved to: $screenshotPath"
} else {
    Write-Error "Failed to save screenshot"
    exit 1
} 