using System.Diagnostics;
using System.Globalization;

namespace RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation;

internal static class UnitOperationComTrace
{
    private const string TraceDirectory = @"D:\Code\RadishFlow\artifacts\pme-trace";
    private const string TraceFileName = "radishflow-unitop-trace.log";
    private static readonly object Gate = new();

    public static void Write(string memberName, string stage, string? detail = null)
    {
        try
        {
            Directory.CreateDirectory(TraceDirectory);
            var line = FormatLine(memberName, stage, detail);
            lock (Gate)
            {
                File.AppendAllText(Path.Combine(TraceDirectory, TraceFileName), line + Environment.NewLine);
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
