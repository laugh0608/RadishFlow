using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Runtime.ExceptionServices;
using System.Text;
using System.Threading.Tasks;
using System.Windows.Forms;
using log4net;
using log4net.Appender;
using log4net.Config;
using log4net.Core;

namespace CasterCore
{
    public static class CasterLogger
    {
        private static ILog log;

        static CasterLogger()
        {
            var assembly = typeof(CasterLogger).Assembly;
            var directory = Path.GetDirectoryName(assembly.Location);
            var configPath = Path.Combine(directory, "log4net.config");
            if (File.Exists(configPath))
            {
                log4net.GlobalContext.Properties["CasterLogFolder"] = directory;
                XmlConfigurator.ConfigureAndWatch(new FileInfo(configPath));
            }
            else
            {
                XmlConfigurator.Configure();
            }
            log = LogManager.GetLogger(typeof(CasterLogger));
            AppDomain.CurrentDomain.FirstChanceException += FirstChanceException;
            AppDomain.CurrentDomain.UnhandledException += UnhandledException;  // this method probably won't be called because the simulator will catch all errors
        }

        private static void FirstChanceException(object sender, FirstChanceExceptionEventArgs e)
        {
            Error($"FirstChanceException: {e.Exception.Message}");
            Error(e.Exception.StackTrace);
        }

        private static void UnhandledException(object sender, UnhandledExceptionEventArgs e)
        {
            Error(e.ExceptionObject.ToString());
        }

        public static void Shutdown()
        {
            LogManager.Shutdown();
        }

        public static void Debug(string msg)
        {
            //var file = LogManager.GetRepository().GetAppenders().FirstOrDefault() as FileAppender;
            //MessageBox.Show(file.File, "test");
            log.Debug(msg);
        }

        public static void DebugFormatted(string format, params object[] args)
        {
            log.DebugFormat(format, args);
        }

        public static void Info(object message)
        {
            log.Info(message);
        }

        public static void InfoFormatted(string format, params object[] args)
        {
            log.InfoFormat(format, args);
        }

        public static void Warn(object message)
        {
            log.Warn(message);
        }

        public static void Warn(object message, Exception exception)
        {
            log.Warn(message, exception);
        }

        public static void WarnFormatted(string format, params object[] args)
        {
            log.WarnFormat(format, args);
        }

        public static void Error(object message)
        {
            log.Error(message);
        }

        public static void Error(object message, Exception exception)
        {
            log.Error(message, exception);
        }

        public static void ErrorFormatted(string format, params object[] args)
        {
            log.ErrorFormat(format, args);
        }

        public static void Fatal(object message)
        {
            log.Fatal(message);
        }

        public static void Fatal(object message, Exception exception)
        {
            log.Fatal(message, exception);
        }

        public static void FatalFormatted(string format, params object[] args)
        {
            log.FatalFormat(format, args);
        }

    }
}
