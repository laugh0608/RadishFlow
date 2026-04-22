using System.Text.Json;
using System.Text.Json.Serialization;
using Microsoft.Win32;

internal static class CapeOpenRegistrationExecutor
{
    public static CapeOpenRegistrationRunResult Execute(
        CapeOpenRegistrationDescriptor descriptor,
        RegistrationOptions options)
    {
        ArgumentNullException.ThrowIfNull(descriptor);
        ArgumentNullException.ThrowIfNull(options);

        ValidateExecutionRequest(descriptor, options);

        var backupDirectory = ResolveBackupDirectory(options, descriptor);
        Directory.CreateDirectory(backupDirectory);

        var backupBundle = CapeOpenRegistryBackupBundleBuilder.Capture(descriptor);
        var backupFilePath = Path.Combine(backupDirectory, "registry-backup.json");
        File.WriteAllText(backupFilePath, SerializeJson(backupBundle));

        var operationResults = new List<CapeOpenRegistryOperationResult>();

        try
        {
            foreach (var entry in descriptor.RegistryPlan)
            {
                operationResults.Add(CapeOpenRegistryPlanExecutor.Apply(descriptor.Scope, entry));
            }

            var executionLogPath = Path.Combine(backupDirectory, "execution-log.json");
            var summary = new CapeOpenRegistrationExecutionSummary(
                Executed: true,
                Succeeded: true,
                BackupDirectory: backupDirectory,
                BackupFilePath: backupFilePath,
                ExecutionLogPath: executionLogPath,
                RollbackAttempted: false,
                RollbackSucceeded: false,
                Message: "Registry execution completed successfully.",
                OperationResults: operationResults);
            var runResult = new CapeOpenRegistrationRunResult(descriptor, summary);
            File.WriteAllText(executionLogPath, SerializeJson(runResult));
            return runResult;
        }
        catch (Exception error)
        {
            var rollbackSucceeded = CapeOpenRegistryRollbackExecutor.TryRestore(descriptor.Scope, backupBundle, out var rollbackError);
            var message = rollbackSucceeded
                ? $"Registry execution failed and the captured backup was restored: {error.Message}"
                : rollbackError is null
                    ? $"Registry execution failed and rollback status is unknown: {error.Message}"
                    : $"Registry execution failed and rollback also failed: {rollbackError.Message}";

            operationResults.Add(new CapeOpenRegistryOperationResult(
                Operation: CapeOpenRegistryPlanOperation.Verify,
                Hive: CapeOpenRegistryHiveAccessor.GetHiveName(descriptor.Scope),
                KeyPath: string.Empty,
                ValueName: null,
                ValueData: null,
                Status: CapeOpenRegistryOperationResultStatus.Failed,
                Detail: message));

            var executionLogPath = Path.Combine(backupDirectory, "execution-log.json");
            var summary = new CapeOpenRegistrationExecutionSummary(
                Executed: true,
                Succeeded: false,
                BackupDirectory: backupDirectory,
                BackupFilePath: backupFilePath,
                ExecutionLogPath: executionLogPath,
                RollbackAttempted: true,
                RollbackSucceeded: rollbackSucceeded,
                Message: message,
                OperationResults: operationResults);
            var runResult = new CapeOpenRegistrationRunResult(descriptor, summary);
            File.WriteAllText(executionLogPath, SerializeJson(runResult));
            throw new CapeOpenRegistrationExecutionException(message, error, runResult);
        }
    }

    private static void ValidateExecutionRequest(
        CapeOpenRegistrationDescriptor descriptor,
        RegistrationOptions options)
    {
        if (options.ExecutionMode != CapeOpenRegistrationExecutionMode.Execute)
        {
            throw new InvalidOperationException("Registration execution requires --execute.");
        }

        if (!string.Equals(options.ConfirmToken, descriptor.RequiredConfirmToken, StringComparison.Ordinal))
        {
            throw new InvalidOperationException(
                $"Registration execution requires --confirm {descriptor.RequiredConfirmToken}.");
        }

        var failedChecks = descriptor.PreflightChecks
            .Where(check => check.Status == CapeOpenPreflightCheckStatus.Fail)
            .ToArray();
        if (failedChecks.Length > 0)
        {
            throw new InvalidOperationException(
                $"Registration execution is blocked by preflight failures: {string.Join("; ", failedChecks.Select(check => $"{check.Name}: {check.Detail}"))}");
        }

        if (descriptor.Scope == CapeOpenRegistrationScope.LocalMachine &&
            !CapeOpenRegistrationElevationChecker.IsProcessElevated())
        {
            throw new InvalidOperationException("Local-machine registration requires elevation.");
        }
    }

    private static string ResolveBackupDirectory(
        RegistrationOptions options,
        CapeOpenRegistrationDescriptor descriptor)
    {
        if (!string.IsNullOrWhiteSpace(options.BackupDirectory))
        {
            return options.BackupDirectory;
        }

        var localAppData = Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData);
        var timestamp = DateTimeOffset.UtcNow.ToString("yyyyMMdd-HHmmss");
        var action = descriptor.Action == CapeOpenRegistrationAction.Register ? "register" : "unregister";
        var scope = descriptor.Scope == CapeOpenRegistrationScope.CurrentUser ? "current-user" : "local-machine";
        return Path.Combine(localAppData, "RadishFlow", "CapeOpen", "RegistrationBackups", $"{timestamp}-{action}-{scope}");
    }

    private static string SerializeJson<T>(T value)
    {
        return JsonSerializer.Serialize(
            value,
            new JsonSerializerOptions
            {
                WriteIndented = true,
                Converters = { new JsonStringEnumConverter() },
            });
    }
}

internal static class CapeOpenRegistryPlanExecutor
{
    public static CapeOpenRegistryOperationResult Apply(
        CapeOpenRegistrationScope scope,
        CapeOpenRegistryPlanEntry entry)
    {
        var hive = CapeOpenRegistryHiveAccessor.OpenRoot(scope);
        return entry.Operation switch
        {
            CapeOpenRegistryPlanOperation.Verify => new CapeOpenRegistryOperationResult(
                entry.Operation,
                entry.Hive,
                entry.KeyPath,
                entry.ValueName,
                entry.ValueData,
                CapeOpenRegistryOperationResultStatus.Skipped,
                "Verify steps are enforced through preflight and are not written during execution."),
            CapeOpenRegistryPlanOperation.SetValue => ApplySetValue(hive, entry),
            CapeOpenRegistryPlanOperation.DeleteTree => ApplyDeleteTree(hive, entry),
            _ => throw new InvalidOperationException($"Unsupported registry plan operation `{entry.Operation}`."),
        };
    }

    private static CapeOpenRegistryOperationResult ApplySetValue(
        RegistryKey hive,
        CapeOpenRegistryPlanEntry entry)
    {
        using var key = hive.CreateSubKey(entry.KeyPath, writable: true)
            ?? throw new InvalidOperationException($"Failed to create or open registry key `{entry.Hive}\\{entry.KeyPath}`.");
        key.SetValue(entry.ValueName ?? string.Empty, entry.ValueData ?? string.Empty, RegistryValueKind.String);
        return new CapeOpenRegistryOperationResult(
            entry.Operation,
            entry.Hive,
            entry.KeyPath,
            entry.ValueName,
            entry.ValueData,
            CapeOpenRegistryOperationResultStatus.Applied,
            "Registry value written.");
    }

    private static CapeOpenRegistryOperationResult ApplyDeleteTree(
        RegistryKey hive,
        CapeOpenRegistryPlanEntry entry)
    {
        var existed = CapeOpenRegistryHiveAccessor.RegistryKeyExists(hive, entry.KeyPath);
        hive.DeleteSubKeyTree(entry.KeyPath, throwOnMissingSubKey: false);
        return new CapeOpenRegistryOperationResult(
            entry.Operation,
            entry.Hive,
            entry.KeyPath,
            entry.ValueName,
            entry.ValueData,
            existed ? CapeOpenRegistryOperationResultStatus.Applied : CapeOpenRegistryOperationResultStatus.Skipped,
            existed ? "Registry tree deleted." : "Registry tree was already absent.");
    }
}

internal static class CapeOpenRegistryBackupBundleBuilder
{
    public static CapeOpenRegistryBackupBundle Capture(CapeOpenRegistrationDescriptor descriptor)
    {
        var hive = CapeOpenRegistryHiveAccessor.OpenRoot(descriptor.Scope);
        var hiveName = CapeOpenRegistryHiveAccessor.GetHiveName(descriptor.Scope);
        var trees = CapeOpenRegistryKeySet.CreateUnitOperationMvp()
            .AllTopLevelKeys
            .Select(keyPath => CaptureTree(hive, hiveName, keyPath))
            .ToArray();

        return new CapeOpenRegistryBackupBundle(
            FormatVersion: "1",
            CapturedAt: DateTimeOffset.UtcNow,
            Action: descriptor.Action,
            Scope: descriptor.Scope,
            ClassId: descriptor.ClassId,
            Trees: trees);
    }

    private static CapeOpenRegistryTreeSnapshot CaptureTree(
        RegistryKey hive,
        string hiveName,
        string keyPath)
    {
        using var key = hive.OpenSubKey(keyPath, writable: false);
        return key is null
            ? new CapeOpenRegistryTreeSnapshot(hiveName, keyPath, false, null)
            : new CapeOpenRegistryTreeSnapshot(hiveName, keyPath, true, CaptureKey(key));
    }

    private static CapeOpenRegistryKeySnapshot CaptureKey(RegistryKey key)
    {
        var values = key.GetValueNames()
            .Select(name => CaptureValue(key, name))
            .ToArray();
        var subKeys = key.GetSubKeyNames()
            .Select(name =>
            {
                using var subKey = key.OpenSubKey(name, writable: false)
                    ?? throw new InvalidOperationException($"Failed to reopen registry subkey `{name}` during backup.");
                return new CapeOpenRegistryNamedSubKeySnapshot(name, CaptureKey(subKey));
            })
            .ToArray();
        return new CapeOpenRegistryKeySnapshot(values, subKeys);
    }

    private static CapeOpenRegistryValueSnapshot CaptureValue(
        RegistryKey key,
        string name)
    {
        var kind = key.GetValueKind(name);
        var value = key.GetValue(name, null, RegistryValueOptions.DoNotExpandEnvironmentNames);
        return kind switch
        {
            RegistryValueKind.String or RegistryValueKind.ExpandString => new CapeOpenRegistryValueSnapshot(
                Name: NormalizeValueName(name),
                Kind: kind,
                StringValue: value as string,
                DWordValue: null,
                QWordValue: null,
                MultiStringValue: null,
                BinaryBase64Value: null),
            RegistryValueKind.DWord => new CapeOpenRegistryValueSnapshot(
                Name: NormalizeValueName(name),
                Kind: kind,
                StringValue: null,
                DWordValue: value is int dword ? dword : null,
                QWordValue: null,
                MultiStringValue: null,
                BinaryBase64Value: null),
            RegistryValueKind.QWord => new CapeOpenRegistryValueSnapshot(
                Name: NormalizeValueName(name),
                Kind: kind,
                StringValue: null,
                DWordValue: null,
                QWordValue: value is long qword ? qword : null,
                MultiStringValue: null,
                BinaryBase64Value: null),
            RegistryValueKind.MultiString => new CapeOpenRegistryValueSnapshot(
                Name: NormalizeValueName(name),
                Kind: kind,
                StringValue: null,
                DWordValue: null,
                QWordValue: null,
                MultiStringValue: value as string[],
                BinaryBase64Value: null),
            RegistryValueKind.Binary or RegistryValueKind.None => new CapeOpenRegistryValueSnapshot(
                Name: NormalizeValueName(name),
                Kind: kind,
                StringValue: null,
                DWordValue: null,
                QWordValue: null,
                MultiStringValue: null,
                BinaryBase64Value: value is byte[] bytes ? Convert.ToBase64String(bytes) : null),
            _ => throw new InvalidOperationException($"Unsupported registry value kind `{kind}` during backup."),
        };
    }

    private static string? NormalizeValueName(string name)
    {
        return string.IsNullOrEmpty(name) ? null : name;
    }
}

internal static class CapeOpenRegistryRollbackExecutor
{
    public static bool TryRestore(
        CapeOpenRegistrationScope scope,
        CapeOpenRegistryBackupBundle backupBundle,
        out Exception? error)
    {
        try
        {
            var hive = CapeOpenRegistryHiveAccessor.OpenRoot(scope);
            foreach (var tree in backupBundle.Trees)
            {
                RestoreTree(hive, tree);
            }

            error = null;
            return true;
        }
        catch (Exception restoreError)
        {
            error = restoreError;
            return false;
        }
    }

    private static void RestoreTree(
        RegistryKey hive,
        CapeOpenRegistryTreeSnapshot tree)
    {
        hive.DeleteSubKeyTree(tree.KeyPath, throwOnMissingSubKey: false);
        if (!tree.Exists || tree.Key is null)
        {
            return;
        }

        using var root = hive.CreateSubKey(tree.KeyPath, writable: true)
            ?? throw new InvalidOperationException($"Failed to recreate registry key `{tree.Hive}\\{tree.KeyPath}` during rollback.");
        RestoreKey(root, tree.Key);
    }

    private static void RestoreKey(
        RegistryKey target,
        CapeOpenRegistryKeySnapshot snapshot)
    {
        foreach (var value in snapshot.Values)
        {
            target.SetValue(value.Name ?? string.Empty, ResolveValueData(value), value.Kind);
        }

        foreach (var subKey in snapshot.SubKeys)
        {
            using var child = target.CreateSubKey(subKey.Name, writable: true)
                ?? throw new InvalidOperationException($"Failed to recreate registry subkey `{subKey.Name}` during rollback.");
            RestoreKey(child, subKey.Snapshot);
        }
    }

    private static object ResolveValueData(CapeOpenRegistryValueSnapshot snapshot)
    {
        return snapshot.Kind switch
        {
            RegistryValueKind.String or RegistryValueKind.ExpandString => snapshot.StringValue ?? string.Empty,
            RegistryValueKind.DWord => snapshot.DWordValue ?? 0,
            RegistryValueKind.QWord => snapshot.QWordValue ?? 0L,
            RegistryValueKind.MultiString => snapshot.MultiStringValue ?? [],
            RegistryValueKind.Binary or RegistryValueKind.None => snapshot.BinaryBase64Value is null
                ? Array.Empty<byte>()
                : Convert.FromBase64String(snapshot.BinaryBase64Value),
            _ => throw new InvalidOperationException($"Unsupported registry value kind `{snapshot.Kind}` during rollback."),
        };
    }
}

internal sealed class CapeOpenRegistrationExecutionException : Exception
{
    public CapeOpenRegistrationExecutionException(
        string message,
        Exception innerException,
        CapeOpenRegistrationRunResult runResult)
        : base(message, innerException)
    {
        RunResult = runResult;
    }

    public CapeOpenRegistrationRunResult RunResult { get; }
}
