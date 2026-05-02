using System;
using System.Diagnostics;
using System.Reflection;
using System.Runtime.InteropServices;
using System.Threading.Tasks;

namespace ComUnRegister
{
    internal class Program
    {
        static void Main(string[] args)
        {
            // 执行主程序
            EnsureAdminAndExecuteAsync().Wait();
        }

        // 确保管理员权限并执行相关逻辑
        public static async Task EnsureAdminAndExecuteAsync()
        {
            if (!IsAdministrator())
            {
                // 如果不是管理员，重新启动应用
                var exePath = Assembly.GetEntryAssembly().Location;
                var startInfo = new ProcessStartInfo(exePath)
                {
                    Verb = "runas",
                    UseShellExecute = true
                };

                Process.Start(startInfo);
                return;
            }
            // 已经是管理员，让用户输入回车确认执行
            await WaitForEnterAsync();
            // 提示用户输入
            Console.Write("请输入您的选择 (1或2): ");
            // 读取用户的输入
            string userInput = Console.ReadLine();
            // 根据用户的输入执行逻辑
            if (userInput == "1")
            {
                Console.WriteLine("您选择了操作 [1]。");
                // 执行注册逻辑
                await RegisterComponentsWithDelay();
                Console.WriteLine("组件注册和单元模块加载完成。");
            }
            else if (userInput == "2")
            {
                Console.WriteLine("您选择了操作 [2]。");
                // 执行反注册逻辑
                await UnregisterComponentsAsync();
                Console.WriteLine("组件反注册和单元模块卸载完成。");
            }
            else
            {
                // 处理无效输入
                Console.WriteLine("无效的选择。请输入 1 或 2。");
            }
            // 等待用户按任意键退出，以便在控制台窗口中查看输出
            Console.WriteLine("按任意键退出...");
            Console.ReadKey();

            // 执行注册逻辑
            //await RegisterComponentsWithDelay();
            // 执行反注册逻辑
            //await UnregisterComponentsAsync();
        }

        // 组件注册逻辑
        private static async Task RegisterComponentsWithDelay()
        {
            Console.WriteLine("管理员权限获取成功，开始加载模块...");
            await Task.Delay(1000);

            RegistrationServices regSvcs = new RegistrationServices();

            Assembly asm1 = Assembly.LoadFrom("CapeOpen.dll");
            regSvcs.RegisterAssembly(asm1, AssemblyRegistrationFlags.SetCodeBase);

            Console.WriteLine("环境依赖注册成功...");
            await Task.Delay(1000);

            Assembly asm2 = Assembly.LoadFrom("MyMixerTest.dll");
            regSvcs.RegisterAssembly(asm2, AssemblyRegistrationFlags.SetCodeBase);

            Console.WriteLine("单元模块注册成功，3 秒后退出程序...");
            await Task.Delay(3000);
        }

        // 反注册逻辑
        private static async Task UnregisterComponentsAsync()
        {
            RegistrationServices regSvcs = new RegistrationServices();

            try
            {
                Console.WriteLine("开始卸载单元模块...");
                await Task.Delay(1000);

                // 取消注册第一个组件
                Assembly asm2 = Assembly.LoadFrom("MyMixerTest.dll");
                regSvcs.UnregisterAssembly(asm2);
                Console.WriteLine("单元模块取消注册成功...");

                // 异步等待
                Console.WriteLine("正在卸载环境依赖...");
                await Task.Delay(1000);

                // 取消注册第二个组件
                Assembly asm1 = Assembly.LoadFrom("CapeOpen.dll");
                regSvcs.UnregisterAssembly(asm1);
                Console.WriteLine("环境依赖取消注册成功");

                await Task.Delay(1000);
                Console.WriteLine("依赖环境卸载成功...");
                
                Console.WriteLine("等待 3 秒后自动退出...");
                // 异步等待
                await Task.Delay(3000);
            }
            catch (Exception ex)
            {
                Console.WriteLine($"取消注册失败: {ex.Message}");
            }
        }

        // 提权逻辑
        public static bool IsAdministrator()
        {
            var identity = System.Security.Principal.WindowsIdentity.GetCurrent();
            var principal = new System.Security.Principal.WindowsPrincipal(identity);
            return principal.IsInRole(System.Security.Principal.WindowsBuiltInRole.Administrator);
        }
        // 等待用户输入逻辑
        private static async Task WaitForEnterAsync()
        {
            Console.WriteLine("请确认已经关闭了所有的 PME 环境后按回车继续...");
            await Task.Run(() =>
            {
                while (Console.ReadKey(true).Key != ConsoleKey.Enter) { }
            });
            Console.WriteLine("请输入数字选择要执行的操作：");
            Console.WriteLine("[1]: 执行组件注册和单元模块加载。");
            Console.WriteLine("[2]: 执行组件反注册和单元模块卸载。");
        }
    }
}
