# Screenshot capture script using Greenshot
param(
    [string]$type = "window", # window, region, or full
    [string]$name = "screenshot"
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
    New-Item -ItemType Directory -Path $screenshotsDir
}

# Capture screenshot based on type
switch ($type) {
    "window" {
        & $greenshotPath --window --output "$screenshotsDir\$name.png"
    }
    "region" {
        & $greenshotPath --region --output "$screenshotsDir\$name.png"
    }
    "full" {
        & $greenshotPath --fullscreen --output "$screenshotsDir\$name.png"
    }
    default {
        Write-Error "Invalid type. Use 'window', 'region', or 'full'"
        exit 1
    }
}

Write-Host "Screenshot saved to $screenshotsDir\$name.png" 