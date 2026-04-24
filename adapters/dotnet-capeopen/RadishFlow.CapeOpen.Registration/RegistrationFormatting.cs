internal static class CapeOpenRegistrationPlanFormatter
{
    public static string Format(CapeOpenRegistrationRunResult runResult)
    {
        ArgumentNullException.ThrowIfNull(runResult);
        var descriptor = runResult.Descriptor;

        var lines = new List<string>
        {
            "RadishFlow CAPE-OPEN registration preflight",
            string.Empty,
            "Component:",
            $"  Name: {descriptor.ComponentName}",
            $"  Description: {descriptor.Description}",
            $"  CLSID: {descriptor.ClassId}",
            $"  ProgID: {descriptor.ProgId}",
            $"  Versioned ProgID: {descriptor.VersionedProgId}",
            $"  TypeLib ID: {descriptor.TypeLibraryId}",
            $"  TypeLib version: {descriptor.TypeLibraryVersion}",
            $"  Assembly: {descriptor.AssemblyName}",
            $"  Type: {descriptor.TypeName}",
            $"  Action: {descriptor.Action}",
            $"  Scope: {descriptor.Scope}",
            $"  Execution mode: {descriptor.ExecutionMode}",
            $"  Required confirm token: {descriptor.RequiredConfirmToken}",
            $"  Comhost path: {descriptor.ResolvedComHostPath}",
            $"  TypeLib path: {descriptor.ResolvedTypeLibraryPath}",
            string.Empty,
            "CAPE-OPEN Categories:",
        };

        lines.AddRange(descriptor.Categories.Select(
            category => $"  - {category.Name}: {category.CategoryId}"));

        lines.Add(string.Empty);
        lines.Add("Implemented Interfaces:");
        lines.AddRange(descriptor.ImplementedInterfaces.Select(
            implementedInterface => $"  - {implementedInterface.Name}: {implementedInterface.InterfaceId}"));

        lines.Add(string.Empty);
        lines.Add("Preflight checks:");
        lines.AddRange(descriptor.PreflightChecks.Select(FormatPreflightCheck));

        lines.Add(string.Empty);
        lines.Add("Backup plan:");
        lines.AddRange(descriptor.BackupPlan.Select(FormatBackupPlanEntry));

        lines.Add(string.Empty);
        lines.Add("Registry plan:");
        lines.AddRange(descriptor.RegistryPlan.Select(FormatRegistryPlanEntry));

        lines.Add(string.Empty);
        lines.Add("Boundary:");
        lines.Add($"  Writes registry: {FormatBoolean(descriptor.WritesRegistry)}");
        lines.Add($"  Requires COM registration now: {FormatBoolean(descriptor.RequiresComRegistration)}");
        lines.Add($"  Requires PME automation now: {FormatBoolean(descriptor.RequiresPmeAutomation)}");
        lines.Add($"  Supports third-party CAPE-OPEN models: {FormatBoolean(descriptor.SupportsThirdPartyCapeOpenModels)}");

        if (runResult.ExecutionSummary is not null)
        {
            lines.Add(string.Empty);
            lines.Add("Execution:");
            lines.AddRange(FormatExecutionSummary(runResult.ExecutionSummary));
        }

        return string.Join(Environment.NewLine, lines);
    }

    private static IEnumerable<string> FormatExecutionSummary(CapeOpenRegistrationExecutionSummary summary)
    {
        yield return $"  Executed: {FormatBoolean(summary.Executed)}";
        yield return $"  Succeeded: {FormatBoolean(summary.Succeeded)}";
        yield return $"  Backup directory: {summary.BackupDirectory}";
        yield return $"  Backup file: {summary.BackupFilePath}";
        yield return $"  Execution log: {summary.ExecutionLogPath}";
        yield return $"  Rollback attempted: {FormatBoolean(summary.RollbackAttempted)}";
        yield return $"  Rollback succeeded: {FormatBoolean(summary.RollbackSucceeded)}";
        yield return $"  Message: {summary.Message}";
        yield return "  Operation results:";
        foreach (var operation in summary.OperationResults)
        {
            var valueName = operation.ValueName is null ? "(Default)" : operation.ValueName;
            var valueData = operation.ValueData is null ? string.Empty : $" = {operation.ValueData}";
            yield return $"    - {operation.Status}: {operation.Operation} {operation.Hive}\\{operation.KeyPath}\\{valueName}{valueData} | {operation.Detail}";
        }
    }

    private static string FormatBoolean(bool value)
    {
        return value ? "yes" : "no";
    }

    private static string FormatRegistryPlanEntry(CapeOpenRegistryPlanEntry entry)
    {
        var valueName = entry.ValueName is null ? "(Default)" : entry.ValueName;
        var valueData = entry.ValueData is null ? string.Empty : $" = {entry.ValueData}";
        return entry.Operation switch
        {
            CapeOpenRegistryPlanOperation.SetValue => $"  - Set {entry.Hive}\\{entry.KeyPath}\\{valueName}{valueData} | {entry.Reason}",
            CapeOpenRegistryPlanOperation.RegisterTypeLibrary => $"  - RegisterTypeLibrary {entry.Hive}\\{entry.KeyPath}{valueData} | {entry.Reason}",
            CapeOpenRegistryPlanOperation.UnregisterTypeLibrary => $"  - UnregisterTypeLibrary {entry.Hive}\\{entry.KeyPath}{valueData} | {entry.Reason}",
            CapeOpenRegistryPlanOperation.DeleteTree => $"  - DeleteTree {entry.Hive}\\{entry.KeyPath} | {entry.Reason}",
            _ => $"  - Verify {entry.Hive}\\{entry.KeyPath} | {entry.Reason}",
        };
    }

    private static string FormatPreflightCheck(CapeOpenPreflightCheck check)
    {
        return $"  - {check.Status}: {check.Name} | {check.Detail}";
    }

    private static string FormatBackupPlanEntry(CapeOpenRegistryBackupPlanEntry entry)
    {
        var state = entry.Exists ? "exists" : "absent";
        return $"  - {entry.Hive}\\{entry.KeyPath}: {state} | {entry.Reason}";
    }
}
