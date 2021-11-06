$packageName = 'chandler'
$version = '3.0.0'
$url = "https://github.com/forbjok/chandler3/releases/download/v$version/chandler-$version-windows-i686.zip"
$url64 = "https://github.com/forbjok/chandler3/releases/download/v$version/chandler-$version-windows-x86_64.zip"
$unzipLocation = "$(Split-Path -Parent $MyInvocation.MyCommand.Definition)"

$packageArgs = @{
  packageName = $packageName
  url = $url
  url64bit = $url64
  unzipLocation = $unzipLocation
}

Install-ChocolateyZipPackage @packageArgs
