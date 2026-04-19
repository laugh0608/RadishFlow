using System.Reflection;
using System.Runtime.InteropServices;

namespace RadishFlow.CapeOpen.Adapter;

public static class RfNativeLibraryLoader
{
    private const string NativeLibraryDirectoryEnvironmentVariable = "RADISHFLOW_NATIVE_LIB_DIR";
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

        foreach (var directory in EnumerateCandidateDirectories(assembly))
        {
            var candidate = Path.Combine(directory, expectedName);
            if (File.Exists(candidate))
            {
                return NativeLibrary.Load(candidate);
            }
        }

        return IntPtr.Zero;
    }

    private static IEnumerable<string> EnumerateCandidateDirectories(Assembly assembly)
    {
        var seen = new HashSet<string>(StringComparer.OrdinalIgnoreCase);

        foreach (var candidate in new[]
        {
            _nativeLibraryDirectory,
            Environment.GetEnvironmentVariable(NativeLibraryDirectoryEnvironmentVariable),
            AppContext.BaseDirectory,
            Path.GetDirectoryName(assembly.Location),
            Environment.CurrentDirectory,
        })
        {
            if (string.IsNullOrWhiteSpace(candidate))
            {
                continue;
            }

            var fullPath = Path.GetFullPath(candidate);
            if (!seen.Add(fullPath))
            {
                continue;
            }

            yield return fullPath;
        }
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
