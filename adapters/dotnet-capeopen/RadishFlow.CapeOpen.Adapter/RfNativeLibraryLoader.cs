using System.Reflection;
using System.Runtime.InteropServices;

namespace RadishFlow.CapeOpen.Adapter;

public static class RfNativeLibraryLoader
{
    private static readonly object Gate = new();
    private static bool _resolverInstalled;
    private static string? _nativeLibraryDirectory;

    public static void ConfigureSearchDirectory(string directoryPath)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(directoryPath);

        lock (Gate)
        {
            InstallResolverIfNeeded();
            _nativeLibraryDirectory = Path.GetFullPath(directoryPath);
        }
    }

    internal static void EnsureResolverInstalled()
    {
        lock (Gate)
        {
            InstallResolverIfNeeded();
        }
    }

    private static void InstallResolverIfNeeded()
    {
        if (_resolverInstalled)
        {
            return;
        }

        NativeLibrary.SetDllImportResolver(
            typeof(RfNativeMethods).Assembly,
            ResolveLibrary);
        _resolverInstalled = true;
    }

    private static nint ResolveLibrary(
        string libraryName,
        Assembly assembly,
        DllImportSearchPath? searchPath)
    {
        var expectedName = GetPlatformLibraryFileName();
        if (!string.Equals(libraryName, "rf_ffi", StringComparison.OrdinalIgnoreCase) &&
            !string.Equals(libraryName, expectedName, StringComparison.OrdinalIgnoreCase))
        {
            return IntPtr.Zero;
        }

        var directory = _nativeLibraryDirectory;
        if (string.IsNullOrWhiteSpace(directory))
        {
            return IntPtr.Zero;
        }

        var candidate = Path.Combine(directory, expectedName);
        if (!File.Exists(candidate))
        {
            return IntPtr.Zero;
        }

        return NativeLibrary.Load(candidate);
    }

    private static string GetPlatformLibraryFileName()
    {
        if (OperatingSystem.IsWindows())
        {
            return "rf_ffi.dll";
        }

        if (OperatingSystem.IsMacOS())
        {
            return "librf_ffi.dylib";
        }

        return "librf_ffi.so";
    }
}
