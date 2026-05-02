internal sealed class RegistrationOptions
{
    private RegistrationOptions(
        bool showHelp,
        bool json,
        CapeOpenRegistrationAction action,
        CapeOpenRegistrationScope scope,
        CapeOpenRegistrationExecutionMode executionMode,
        string? confirmToken,
        string? comHostPath,
        string? typeLibraryPath,
        string? backupDirectory)
    {
        ShowHelp = showHelp;
        Json = json;
        Action = action;
        Scope = scope;
        ExecutionMode = executionMode;
        ConfirmToken = confirmToken;
        ComHostPath = comHostPath;
        TypeLibraryPath = typeLibraryPath;
        BackupDirectory = backupDirectory;
    }

    public bool ShowHelp { get; }

    public bool Json { get; }

    public CapeOpenRegistrationAction Action { get; }

    public CapeOpenRegistrationScope Scope { get; }

    public CapeOpenRegistrationExecutionMode ExecutionMode { get; }

    public string? ConfirmToken { get; }

    public string? ComHostPath { get; }

    public string? TypeLibraryPath { get; }

    public string? BackupDirectory { get; }

    public static string HelpText =>
        """
        RadishFlow.CapeOpen.Registration

        Prints the dry-run registration plan for the MVP CAPE-OPEN Unit Operation PMC.
        By default this tool does not write the registry, register COM classes, start a PME, or load third-party CAPE-OPEN models.

        Options:
          --action <register|unregister>           Registration action. Default: register
          --scope <current-user|local-machine>     Registry scope to plan. Default: current-user
          --comhost <path>                         Optional explicit RadishFlow.CapeOpen.UnitOp.Mvp.comhost.dll path
          --typelib <path>                         Optional explicit RadishFlow.CapeOpen.UnitOp.Mvp.tlb path
          --execute                                Execute the planned registry changes after passing all gates
          --confirm <token>                        Required confirmation token for --execute
          --backup-dir <path>                      Optional backup/log output directory for --execute
          --json                                   Print descriptor or execution result as JSON
          --help                                   Show this help text
        """;

    public static RegistrationOptions Parse(string[] args)
    {
        var showHelp = false;
        var json = false;
        var action = CapeOpenRegistrationAction.Register;
        var scope = CapeOpenRegistrationScope.CurrentUser;
        var executionMode = CapeOpenRegistrationExecutionMode.DryRun;
        string? confirmToken = null;
        string? comHostPath = null;
        string? typeLibraryPath = null;
        string? backupDirectory = null;

        for (var index = 0; index < args.Length; index++)
        {
            var arg = args[index];
            if (string.Equals(arg, "--help", StringComparison.OrdinalIgnoreCase))
            {
                showHelp = true;
                continue;
            }

            if (string.Equals(arg, "--json", StringComparison.OrdinalIgnoreCase))
            {
                json = true;
                continue;
            }

            if (string.Equals(arg, "--execute", StringComparison.OrdinalIgnoreCase))
            {
                executionMode = CapeOpenRegistrationExecutionMode.Execute;
                continue;
            }

            if (string.Equals(arg, "--action", StringComparison.OrdinalIgnoreCase))
            {
                action = ParseAction(ReadOptionValue(args, ref index, arg));
                continue;
            }

            if (string.Equals(arg, "--scope", StringComparison.OrdinalIgnoreCase))
            {
                scope = ParseScope(ReadOptionValue(args, ref index, arg));
                continue;
            }

            if (string.Equals(arg, "--confirm", StringComparison.OrdinalIgnoreCase))
            {
                confirmToken = ReadOptionValue(args, ref index, arg);
                continue;
            }

            if (string.Equals(arg, "--comhost", StringComparison.OrdinalIgnoreCase))
            {
                comHostPath = Path.GetFullPath(ReadOptionValue(args, ref index, arg));
                continue;
            }

            if (string.Equals(arg, "--typelib", StringComparison.OrdinalIgnoreCase))
            {
                typeLibraryPath = Path.GetFullPath(ReadOptionValue(args, ref index, arg));
                continue;
            }

            if (string.Equals(arg, "--backup-dir", StringComparison.OrdinalIgnoreCase))
            {
                backupDirectory = Path.GetFullPath(ReadOptionValue(args, ref index, arg));
                continue;
            }

            throw new ArgumentException($"Unknown option `{arg}`.");
        }

        return new RegistrationOptions(
            showHelp,
            json,
            action,
            scope,
            executionMode,
            confirmToken,
            comHostPath,
            typeLibraryPath,
            backupDirectory);
    }

    private static string ReadOptionValue(
        string[] args,
        ref int index,
        string option)
    {
        if (index == args.Length - 1)
        {
            throw new ArgumentException($"Missing value for option `{option}`.");
        }

        return args[++index];
    }

    private static CapeOpenRegistrationAction ParseAction(string value)
    {
        return value.ToLowerInvariant() switch
        {
            "register" => CapeOpenRegistrationAction.Register,
            "unregister" => CapeOpenRegistrationAction.Unregister,
            _ => throw new ArgumentException($"Unknown registration action `{value}`."),
        };
    }

    private static CapeOpenRegistrationScope ParseScope(string value)
    {
        return value.ToLowerInvariant() switch
        {
            "current-user" => CapeOpenRegistrationScope.CurrentUser,
            "local-machine" => CapeOpenRegistrationScope.LocalMachine,
            _ => throw new ArgumentException($"Unknown registration scope `{value}`."),
        };
    }
}
