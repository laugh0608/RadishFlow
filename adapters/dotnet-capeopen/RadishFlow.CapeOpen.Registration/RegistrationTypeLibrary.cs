using System.Runtime.InteropServices;
using System.Runtime.InteropServices.ComTypes;

internal enum CapeOpenTypeLibraryLoadKind
{
    Default = 0,
    Register = 1,
    None = 2,
}

internal static class CapeOpenTypeLibraryNativeMethods
{
    [DllImport("oleaut32.dll", CharSet = CharSet.Unicode, PreserveSig = false)]
    internal static extern void LoadTypeLibEx(
        string typeLibPath,
        CapeOpenTypeLibraryLoadKind regKind,
        [MarshalAs(UnmanagedType.Interface)] out ITypeLib typeLibrary);

    [DllImport("oleaut32.dll", CharSet = CharSet.Unicode, PreserveSig = false)]
    internal static extern void RegisterTypeLib(
        [MarshalAs(UnmanagedType.Interface)] ITypeLib typeLibrary,
        string fullPath,
        string helpDirectory);

    [DllImport("oleaut32.dll", CharSet = CharSet.Unicode, PreserveSig = false, EntryPoint = "RegisterTypeLibForUser")]
    internal static extern void RegisterTypeLibForUser(
        [MarshalAs(UnmanagedType.Interface)] ITypeLib typeLibrary,
        string fullPath,
        string helpDirectory);

    [DllImport("oleaut32.dll", PreserveSig = false)]
    internal static extern void UnRegisterTypeLib(
        ref Guid libraryId,
        short majorVersion,
        short minorVersion,
        int localeId,
        SYSKIND sysKind);

    [DllImport("oleaut32.dll", PreserveSig = false, EntryPoint = "UnRegisterTypeLibForUser")]
    internal static extern void UnRegisterTypeLibForUser(
        ref Guid libraryId,
        short majorVersion,
        short minorVersion,
        int localeId,
        SYSKIND sysKind);
}

internal sealed record CapeOpenTypeLibraryIdentity(
    Guid Guid,
    Version Version,
    int LocaleId,
    SYSKIND SysKind);

internal static class CapeOpenTypeLibraryVersionParser
{
    public static Version Parse(string versionText)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(versionText);
        if (!Version.TryParse(versionText, out var version))
        {
            throw new InvalidOperationException($"Invalid type library version `{versionText}`.");
        }

        return version;
    }
}

internal static class CapeOpenTypeLibraryPathResolver
{
    public static string Resolve(
        Type componentType,
        string? explicitPath,
        string comHostPath)
    {
        ArgumentNullException.ThrowIfNull(componentType);
        ArgumentException.ThrowIfNullOrWhiteSpace(comHostPath);

        if (!string.IsNullOrWhiteSpace(explicitPath))
        {
            return Path.GetFullPath(explicitPath);
        }

        var comHostDirectory = Path.GetDirectoryName(comHostPath) ?? Environment.CurrentDirectory;
        var assemblyDirectory = Path.GetDirectoryName(componentType.Assembly.Location) ?? Environment.CurrentDirectory;
        var fileName = RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation.UnitOperationComIdentity.TypeLibraryFileName;
        var candidates = new[]
        {
            Path.Combine(comHostDirectory, fileName),
            Path.Combine(comHostDirectory, "typelib", fileName),
            Path.Combine(assemblyDirectory, fileName),
            Path.Combine(assemblyDirectory, "typelib", fileName),
            Path.Combine(AppContext.BaseDirectory, fileName),
            Path.Combine(AppContext.BaseDirectory, "typelib", fileName),
        };

        var resolved = candidates.FirstOrDefault(File.Exists);
        return resolved is not null
            ? Path.GetFullPath(resolved)
            : Path.GetFullPath(candidates[0]);
    }
}

internal static class CapeOpenTypeLibraryInspector
{
    public static CapeOpenTypeLibraryIdentity Inspect(string typeLibraryPath)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(typeLibraryPath);

        CapeOpenTypeLibraryNativeMethods.LoadTypeLibEx(typeLibraryPath, CapeOpenTypeLibraryLoadKind.None, out var typeLibrary);
        typeLibrary.GetLibAttr(out var libAttrPointer);
        try
        {
            var libAttr = Marshal.PtrToStructure<TYPELIBATTR>(libAttrPointer);
            return new CapeOpenTypeLibraryIdentity(
                Guid: libAttr.guid,
                Version: new Version(libAttr.wMajorVerNum, libAttr.wMinorVerNum),
                LocaleId: libAttr.lcid,
                SysKind: libAttr.syskind);
        }
        finally
        {
            typeLibrary.ReleaseTLibAttr(libAttrPointer);
        }
    }
}

internal static class CapeOpenTypeLibraryRegistrar
{
    public static CapeOpenTypeLibraryIdentity Register(
        CapeOpenRegistrationScope scope,
        string typeLibraryPath)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(typeLibraryPath);

        CapeOpenTypeLibraryNativeMethods.LoadTypeLibEx(typeLibraryPath, CapeOpenTypeLibraryLoadKind.None, out var typeLibrary);
        var helpDirectory = Path.GetDirectoryName(typeLibraryPath) ?? Environment.CurrentDirectory;

        if (scope == CapeOpenRegistrationScope.CurrentUser)
        {
            CapeOpenTypeLibraryNativeMethods.RegisterTypeLibForUser(typeLibrary, typeLibraryPath, helpDirectory);
        }
        else
        {
            CapeOpenTypeLibraryNativeMethods.RegisterTypeLib(typeLibrary, typeLibraryPath, helpDirectory);
        }

        return CapeOpenTypeLibraryInspector.Inspect(typeLibraryPath);
    }

    public static CapeOpenTypeLibraryIdentity Unregister(
        CapeOpenRegistrationScope scope,
        string typeLibraryPath)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(typeLibraryPath);

        var identity = CapeOpenTypeLibraryInspector.Inspect(typeLibraryPath);
        var libraryId = identity.Guid;
        if (scope == CapeOpenRegistrationScope.CurrentUser)
        {
            CapeOpenTypeLibraryNativeMethods.UnRegisterTypeLibForUser(
                ref libraryId,
                (short)identity.Version.Major,
                (short)identity.Version.Minor,
                identity.LocaleId,
                identity.SysKind);
        }
        else
        {
            CapeOpenTypeLibraryNativeMethods.UnRegisterTypeLib(
                ref libraryId,
                (short)identity.Version.Major,
                (short)identity.Version.Minor,
                identity.LocaleId,
                identity.SysKind);
        }

        return identity;
    }
}
