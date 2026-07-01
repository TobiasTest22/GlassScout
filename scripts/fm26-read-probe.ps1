$ErrorActionPreference = "Stop"

Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;

public static class GlassScoutReadProbe {
  [DllImport("kernel32.dll", SetLastError=true)]
  public static extern IntPtr OpenProcess(uint access, bool inherit, uint processId);

  [DllImport("kernel32.dll", SetLastError=true)]
  [return: MarshalAs(UnmanagedType.Bool)]
  public static extern bool ReadProcessMemory(
    IntPtr process,
    IntPtr address,
    byte[] buffer,
    UIntPtr size,
    out UIntPtr read
  );

  [DllImport("kernel32.dll")]
  [return: MarshalAs(UnmanagedType.Bool)]
  public static extern bool CloseHandle(IntPtr handle);
}
'@

$fm = Get-Process -Name "fm" -ErrorAction Stop | Select-Object -First 1
$processVmRead = 0x0010
$processQueryLimitedInformation = 0x1000
$access = $processVmRead -bor $processQueryLimitedInformation
$handle = [GlassScoutReadProbe]::OpenProcess($access, $false, [uint32]$fm.Id)

if ($handle -eq [IntPtr]::Zero) {
  throw "OpenProcess failed with Windows error $([Runtime.InteropServices.Marshal]::GetLastWin32Error())"
}

try {
  $buffer = New-Object byte[] 64
  $bytesRead = [UIntPtr]::Zero
  $size = [UIntPtr][uint64]64
  $succeeded = [GlassScoutReadProbe]::ReadProcessMemory(
    $handle,
    $fm.MainModule.BaseAddress,
    $buffer,
    $size,
    [ref]$bytesRead
  )

  [pscustomobject]@{
    ProcessDetected = $true
    Process = "fm.exe"
    ProcessId = $fm.Id
    AccessMask = "QUERY_LIMITED_INFORMATION | VM_READ"
    BaseAddress = "0x{0:X}" -f $fm.MainModule.BaseAddress.ToInt64()
    ReadSucceeded = $succeeded
    BytesRead = $bytesRead.ToUInt64()
    Header = [Text.Encoding]::ASCII.GetString($buffer[0..1])
    WriteRightsRequested = $false
  } | Format-List
}
finally {
  [void][GlassScoutReadProbe]::CloseHandle($handle)
}
