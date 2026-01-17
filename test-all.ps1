# Test script for all Titan templates

Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "Testing Titan CLI and Templates" -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan

# Test CLI version
Write-Host "`n1. Testing CLI version..." -ForegroundColor Yellow
node index.js --version

# Test help
Write-Host "`n2. Testing help command..." -ForegroundColor Yellow
node index.js help | Select-Object -First 5

# Test all templates
Write-Host "`n3. Testing builds for all templates..." -ForegroundColor Yellow

$templates = @("js", "ts", "rust-js", "rust-ts")

foreach ($template in $templates) {
    Write-Host "`n--- Testing $template ---" -ForegroundColor Magenta
    $appDir = "test-apps\test-$template"
    
    if (Test-Path $appDir) {
        Push-Location $appDir
        
        Write-Host "Building $template..." -ForegroundColor Gray
        node ..\..\index.js build 2>&1 | Out-Null
        
        # Check if build artifacts exist
        if (Test-Path "server\routes.json") {
            Write-Host "✅ routes.json created" -ForegroundColor Green
        } else {
            Write-Host "❌ routes.json missing" -ForegroundColor Red
        }
        
        if (Test-Path "server\actions") {
            Write-Host "✅ actions directory exists" -ForegroundColor Green
        } else {
            Write-Host "❌ actions directory missing" -ForegroundColor Red
        }
        
        # Check Dockerfile
        if (Test-Path "Dockerfile") {
            Write-Host "✅ Dockerfile exists" -ForegroundColor Green
            $dockerContent = Get-Content "Dockerfile" -Raw
            if ($dockerContent -match "FROM" -and $dockerContent -match "WORKDIR") {
                Write-Host "✅ Dockerfile looks valid" -ForegroundColor Green
            } else {
                Write-Host "⚠️  Dockerfile may be invalid" -ForegroundColor Yellow
            }
        } else {
            Write-Host "❌ Dockerfile missing" -ForegroundColor Red
        }
        
        Pop-Location
    } else {
        Write-Host "❌ Directory not found: $appDir" -ForegroundColor Red
    }
}

Write-Host "`n==========================================" -ForegroundColor Cyan
Write-Host "✅ All tests completed!" -ForegroundColor Green
Write-Host "==========================================" -ForegroundColor Cyan

