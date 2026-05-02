using System.Diagnostics;
using System.Globalization;

namespace RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation;

internal static class UnitOperationComTrace
{
    private const string TraceDirectoryEnvironmentVariable = "RADISHFLOW_CAPEOPEN_TRACE_DIR";
    private const string TraceFileNameEnvironmentVariable = "RADISHFLOW_CAPEOPEN_TRACE_FILE";
    private const string DefaultTraceFileName = "radishflow-unitop-trace.log";
    private static readonly object Gate = new();

    public static void Write(string memberName, string stage, string? detail = null)
    {
        try
        {
            var tracePath = TryGetTracePath();
            if (tracePath is null)
            {
                return;
            }

            Directory.CreateDirectory(Path.GetDirectoryName(tracePath)!);
            var line = FormatLine(memberName, stage, detail);
            lock (Gate)
            {
                File.AppendAllText(tracePath, line + Environment.NewLine);
            }
        }
        catch
        {
            // Trace must never change COM/PME behavior.
        }
    }

    public static void Exception(string memberName, Exception error)
    {
        Write(memberName, "exception", $"{error.GetType().FullName}: {error.Message}");
    }

    private static string? TryGetTracePath()
    {
        var traceDirectory = Environment.GetEnvironmentVariable(TraceDirectoryEnvironmentVariable);
        if (string.IsNullOrWhiteSpace(traceDirectory))
        {
            return null;
        }

        var traceFileName = Environment.GetEnvironmentVariable(TraceFileNameEnvironmentVariable);
        if (string.IsNullOrWhiteSpace(traceFileName))
        {
            traceFileName = DefaultTraceFileName;
        }

        return Path.Combine(traceDirectory, traceFileName);
    }

    private static string FormatLine(string memberName, string stage, string? detail)
    {
        var process = Process.GetCurrentProcess();
        return string.Join(
            " | ",
            DateTimeOffset.Now.ToString("O", CultureInfo.InvariantCulture),
            $"pid={process.Id}",
            $"process={process.ProcessName}",
            $"tid={Environment.CurrentManagedThreadId}",
            $"member={memberName}",
            $"stage={stage}",
            $"detail={detail ?? string.Empty}");
    }
}
