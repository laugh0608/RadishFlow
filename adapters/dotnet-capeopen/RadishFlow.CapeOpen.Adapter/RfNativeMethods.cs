using System.Runtime.InteropServices;

namespace RadishFlow.CapeOpen.Adapter;

internal static partial class RfNativeMethods
{
    private const string LibraryName = "rf_ffi";

    [LibraryImport(LibraryName, EntryPoint = "engine_create")]
    internal static partial RfFfiStatus EngineCreate(out nint engine);

    [LibraryImport(LibraryName, EntryPoint = "engine_destroy")]
    internal static partial void EngineDestroy(nint engine);

    [LibraryImport(LibraryName, EntryPoint = "engine_last_error_message")]
    internal static partial RfFfiStatus EngineLastErrorMessage(
        nint engine,
        out nint message);

    [LibraryImport(LibraryName, EntryPoint = "engine_last_error_json")]
    internal static partial RfFfiStatus EngineLastErrorJson(
        nint engine,
        out nint message);

    [LibraryImport(LibraryName, EntryPoint = "rf_string_free")]
    internal static partial void RfStringFree(nint value);

    [LibraryImport(LibraryName, EntryPoint = "flowsheet_load_json")]
    internal static partial RfFfiStatus FlowsheetLoadJson(
        nint engine,
        byte[] jsonUtf8,
        nuint jsonLength);

    [LibraryImport(LibraryName, EntryPoint = "property_package_load_from_files")]
    internal static partial RfFfiStatus PropertyPackageLoadFromFiles(
        nint engine,
        byte[] manifestPathUtf8,
        nuint manifestPathLength,
        byte[] payloadPathUtf8,
        nuint payloadPathLength);

    [LibraryImport(LibraryName, EntryPoint = "property_package_list_json")]
    internal static partial RfFfiStatus PropertyPackageListJson(
        nint engine,
        out nint json);

    [LibraryImport(LibraryName, EntryPoint = "flowsheet_solve")]
    internal static partial RfFfiStatus FlowsheetSolve(
        nint engine,
        byte[] packageIdUtf8,
        nuint packageIdLength);

    [LibraryImport(LibraryName, EntryPoint = "flowsheet_get_snapshot_json")]
    internal static partial RfFfiStatus FlowsheetGetSnapshotJson(
        nint engine,
        out nint json);

    [LibraryImport(LibraryName, EntryPoint = "stream_get_snapshot_json")]
    internal static partial RfFfiStatus StreamGetSnapshotJson(
        nint engine,
        byte[] streamIdUtf8,
        nuint streamIdLength,
        out nint json);
}
