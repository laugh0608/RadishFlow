using System.Text.Json;
using System.Text.Json.Serialization;

internal static class RegistrationExecutable
{
    public static int Run(string[] args)
    {
        RegistrationOptions? options = null;

        try
        {
            options = RegistrationOptions.Parse(args);
            if (options.ShowHelp)
            {
                Console.WriteLine(RegistrationOptions.HelpText);
                return 0;
            }

            var descriptor = CapeOpenRegistrationDescriptor.CreateUnitOperationMvp(
                options.Action,
                options.Scope,
                options.ExecutionMode,
                options.ComHostPath,
                options.TypeLibraryPath);

            var runResult = options.ExecutionMode == CapeOpenRegistrationExecutionMode.Execute
                ? CapeOpenRegistrationExecutor.Execute(descriptor, options)
                : new CapeOpenRegistrationRunResult(descriptor, null);

            WriteOutput(options, runResult);
            return 0;
        }
        catch (CapeOpenRegistrationExecutionException error)
        {
            if (options?.Json == true)
            {
                WriteOutput(options, error.RunResult);
            }

            Console.Error.WriteLine("Registration execution failed.");
            Console.Error.WriteLine(error.Message);
            return 1;
        }
        catch (Exception error)
        {
            Console.Error.WriteLine("Registration preflight failed.");
            Console.Error.WriteLine(error.Message);
            return 1;
        }
    }

    private static void WriteOutput(
        RegistrationOptions options,
        CapeOpenRegistrationRunResult runResult)
    {
        if (options.Json)
        {
            object payload = runResult.ExecutionSummary is null
                ? runResult.Descriptor
                : runResult;
            Console.WriteLine(JsonSerializer.Serialize(
                payload,
                new JsonSerializerOptions
                {
                    WriteIndented = true,
                    Converters = { new JsonStringEnumConverter() },
                }));
            return;
        }

        Console.WriteLine(CapeOpenRegistrationPlanFormatter.Format(runResult));
    }
}
