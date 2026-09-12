using RadishFlow.CapeOpen.Adapter;
using RadishFlow.CapeOpen.Interop.Errors;

internal static class NativeEngineLifetimeTests
{
    private static readonly TimeSpan Timeout = TimeSpan.FromSeconds(10);

    public static void Run()
    {
        DisposedEngineRejectsAllOperations();
        CompetingOperationWaits(dispose: false);
        CompetingOperationWaits(dispose: true);
        ConcurrentFailuresKeepTheirOwnDiagnostics();
        Console.WriteLine("Native engine lifetime tests passed (4 scenarios).");
    }

    private static void DisposedEngineRejectsAllOperations()
    {
        using var engine = new RadishFlowNativeEngine();
        engine.Dispose();
        engine.Dispose();
        Action[] operations =
        [
            () => engine.LoadFlowsheetJson("{}"),
            () => engine.LoadPropertyPackageFiles("manifest.json", "payload.json"),
            () => engine.SolveFlowsheet("package"),
            () => engine.GetPropertyPackageListJson(),
            () => engine.GetFlowsheetSnapshotJson(),
            () => engine.GetStreamSnapshotJson("stream"),
            () => engine.TryGetLastErrorMessage(),
            () => engine.TryGetLastErrorJson(),
        ];
        foreach (var operation in operations)
        {
            ExpectDisposed(operation);
        }
    }

    private static void CompetingOperationWaits(bool dispose)
    {
        using var engine = new RadishFlowNativeEngine();
        using var entered = new ManualResetEventSlim();
        using var release = new ManualResetEventSlim();
        Exception? operationFailure = null;
        Exception? contenderFailure = null;
        var operation = new Thread(() =>
        {
            try
            {
                engine.WithHandle(_ =>
                {
                    entered.Set();
                    Require(release.Wait(Timeout), "Timed out waiting to release native operation.");
                    // A real P/Invoke must remain usable while Dispose is waiting.
                    return engine.GetPropertyPackageListJson();
                });
            }
            catch (Exception error)
            {
                operationFailure = error;
            }
        }) { IsBackground = true };
        var contender = new Thread(() =>
        {
            try
            {
                if (dispose)
                {
                    engine.Dispose();
                }
                else
                {
                    engine.GetPropertyPackageListJson();
                }
            }
            catch (Exception error)
            {
                contenderFailure = error;
            }
        }) { IsBackground = true };

        operation.Start();
        try
        {
            Require(entered.Wait(Timeout), "Native operation did not enter its critical section.");
            contender.Start();
            Require(SpinWait.SpinUntil(() =>
                (contender.ThreadState & ThreadState.WaitSleepJoin) != 0 || !contender.IsAlive,
                Timeout), "Contending operation did not reach a blocked or completed state.");
            Require(contender.IsAlive, "A competing call or Dispose completed while the native operation was active.");
        }
        finally
        {
            release.Set();
            Require(operation.Join(Timeout), "Native operation did not finish.");
            if ((contender.ThreadState & ThreadState.Unstarted) == 0)
            {
                Require(contender.Join(Timeout), "Contending operation did not finish.");
            }
        }

        if (operationFailure is not null)
        {
            throw new InvalidOperationException("Native operation failed.", operationFailure);
        }
        if (contenderFailure is not null)
        {
            throw new InvalidOperationException("Contending operation failed.", contenderFailure);
        }
        if (dispose)
        {
            ExpectDisposed(() => engine.GetPropertyPackageListJson());
        }
        else
        {
            engine.GetPropertyPackageListJson();
        }
    }

    private static void ConcurrentFailuresKeepTheirOwnDiagnostics()
    {
        using var engine = new RadishFlowNativeEngine();
        Parallel.For(0, 64, index =>
        {
            var expectedOperation = index % 2 == 0 ? "flowsheet_load_json" : "flowsheet_get_snapshot_json";
            try
            {
                if (index % 2 == 0)
                {
                    engine.LoadFlowsheetJson("invalid-json");
                }
                else
                {
                    engine.GetFlowsheetSnapshotJson();
                }
                throw new InvalidOperationException("Invalid native request unexpectedly succeeded.");
            }
            catch (CapeOpenException error)
            {
                Require(error.Operation == expectedOperation, "Concurrent operations mixed their error operation.");
                var expectedMessage = index % 2 == 0 ? "deserialize" : "snapshot";
                Require(error.Message.Contains(expectedMessage, StringComparison.OrdinalIgnoreCase),
                    "Concurrent operations mixed their native error message.");
            }
            // Also verifies exceptions release the gate for subsequent calls.
            engine.GetPropertyPackageListJson();
        });
    }

    private static void ExpectDisposed(Action operation)
    {
        try
        {
            operation();
        }
        catch (ObjectDisposedException)
        {
            return;
        }
        throw new InvalidOperationException("Disposed engine accepted a native operation.");
    }

    private static void Require(bool condition, string message)
    {
        if (!condition)
        {
            throw new InvalidOperationException(message);
        }
    }
}
