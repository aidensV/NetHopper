param(
    [string]$OutputPath = (Join-Path $PSScriptRoot "..\public\orbit-logo.png"),
    [int]$Size = 512
)

Add-Type -AssemblyName PresentationCore
Add-Type -AssemblyName WindowsBase

$scale = $Size / 28.0
$iconSize = 17.0 * $scale
$iconOffset = ($Size - $iconSize) / 2.0
$iconScale = $iconSize / 24.0

$visual = [System.Windows.Media.DrawingVisual]::new()
$context = $visual.RenderOpen()

$backgroundBrush = [System.Windows.Media.SolidColorBrush]::new(
    [System.Windows.Media.Color]::FromRgb(34, 211, 238)
)
$cornerRadius = 8.0 * $scale
$context.DrawRoundedRectangle(
    $backgroundBrush,
    $null,
    [System.Windows.Rect]::new(0, 0, $Size, $Size),
    $cornerRadius,
    $cornerRadius
)

$strokeBrush = [System.Windows.Media.SolidColorBrush]::new(
    [System.Windows.Media.Color]::FromRgb(2, 6, 23)
)
$pen = [System.Windows.Media.Pen]::new($strokeBrush, 2.4)
$pen.StartLineCap = [System.Windows.Media.PenLineCap]::Round
$pen.EndLineCap = [System.Windows.Media.PenLineCap]::Round
$pen.LineJoin = [System.Windows.Media.PenLineJoin]::Round

$transform = [System.Windows.Media.TransformGroup]::new()
$transform.Children.Add([System.Windows.Media.ScaleTransform]::new($iconScale, $iconScale))
$transform.Children.Add([System.Windows.Media.TranslateTransform]::new($iconOffset, $iconOffset))

$context.PushTransform($transform)

$paths = @(
    "M20.341,6.484 A10,10 0 0 1 10.266,21.85",
    "M3.659,17.516 A10,10 0 0 1 13.74,2.152"
)

foreach ($path in $paths) {
    $geometry = [System.Windows.Media.Geometry]::Parse($path)
    $context.DrawGeometry($null, $pen, $geometry)
}

foreach ($circle in @(
    @(12.0, 12.0, 3.0),
    @(19.0, 5.0, 2.0),
    @(5.0, 19.0, 2.0)
)) {
    $center = [System.Windows.Point]::new($circle[0], $circle[1])
    $context.DrawEllipse($null, $pen, $center, $circle[2], $circle[2])
}

$context.Pop()
$context.Close()

$bitmap = [System.Windows.Media.Imaging.RenderTargetBitmap]::new(
    $Size,
    $Size,
    96,
    96,
    [System.Windows.Media.PixelFormats]::Pbgra32
)
$bitmap.Render($visual)

$encoder = [System.Windows.Media.Imaging.PngBitmapEncoder]::new()
$encoder.Frames.Add([System.Windows.Media.Imaging.BitmapFrame]::Create($bitmap))

$resolvedOutput = [System.IO.Path]::GetFullPath($OutputPath)
[System.IO.Directory]::CreateDirectory([System.IO.Path]::GetDirectoryName($resolvedOutput)) | Out-Null
$stream = [System.IO.File]::Create($resolvedOutput)
try {
    $encoder.Save($stream)
}
finally {
    $stream.Dispose()
}

Write-Output $resolvedOutput
