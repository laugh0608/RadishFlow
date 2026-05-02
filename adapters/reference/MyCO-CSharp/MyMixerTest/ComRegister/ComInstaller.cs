using System;
using System.Collections;
using System.ComponentModel;
using System.Configuration.Install;
using System.Diagnostics;
using System.IO;
using System.Reflection;
using System.Runtime.InteropServices;
using System.Threading;
using System.Threading.Tasks;
using System.Windows.Forms;

namespace ComRegister
{
    [RunInstaller(true)]
    public partial class ComInstaller : Installer
    {
        public ComInstaller()
        {
            InitializeComponent();
        }

        // 可选：覆盖 Install 方法：在安装过程中调用
        public override void Install(IDictionary stateSaver)
        {
            base.Install(stateSaver); // 务必调用基类的 Install 方法

            MessageBox.Show("正在执行自定义安装操作...");
            // 这里可以添加其他安装逻辑
        }

        // 覆盖 Commit 方法：安装成功完成时调用
        public override void Commit(IDictionary savedState)
        {
            base.Commit(savedState); // 务必调用基类的 Commit 方法

            // 在这里显示你的消息框
            MessageBox.Show("安装成功！");
        }

        // 可选：覆盖 Rollback 方法：安装失败回滚时调用
        public override void Rollback(IDictionary savedState)
        {
            base.Rollback(savedState); // 务必调用基类的 Rollback 方法

            MessageBox.Show("安装已回滚！", "安装失败回滚");
        }

        // 可选：覆盖 Uninstall 方法：卸载时调用
        public override void Uninstall(IDictionary savedState)
        {
            base.Uninstall(savedState); // 务必调用基类的 Uninstall 方法

            MessageBox.Show("卸载完成！");
        }


        // 自定义 dll 注册操作，OnCommitted 阶段是安装完毕之后提交阶段
        // 权限为完全管理员
        protected override void OnCommitted(IDictionary savedState)
        {
            base.OnCommitted(savedState);

            MessageBox.Show("开始执行类库注册...");

            // 拿到 dll 文件路径
            var paths = GetDllPath();
            var coPath = paths.GetCoPath;
            var opPath = paths.GetOpPath;

            // 执行 CapeOpen.dll 和 MyMixerTest.dll 注册
            RegistrationServices regSvcs = new RegistrationServices();
            Assembly coAsm = Assembly.LoadFrom(coPath);  // CapeOpen.dll
            regSvcs.RegisterAssembly(coAsm, AssemblyRegistrationFlags.SetCodeBase);
            Thread.Sleep(1000);
            Assembly opAsm = Assembly.LoadFrom(opPath);  // MyMixerTest.dll
            regSvcs.RegisterAssembly(opAsm, AssemblyRegistrationFlags.SetCodeBase);
            MessageBox.Show("类库注册完毕！");
        }
        // 自定义 dll 反注册操作，OnBeforeUninstall 阶段是在执行卸载程序之前
        // 权限为完全管理员
        protected override void OnBeforeUninstall(IDictionary savedState)
        {
            base.OnBeforeUninstall(savedState);

            MessageBox.Show("开始执行类库卸载...");

            // 拿到 dll 文件路径
            var paths = GetDllPath();
            var coPath = paths.GetCoPath;
            var opPath = paths.GetOpPath;

            // 执行 CapeOpen.dll 和 MyMixerTest.dll 反注册
            RegistrationServices regSvcs = new RegistrationServices();
            Assembly opAsm = Assembly.LoadFrom(opPath);  // MyMixerTest.dll
            regSvcs.UnregisterAssembly(opAsm);
            Thread.Sleep(1000);
            Assembly coAsm = Assembly.LoadFrom(coPath);  // CapeOpen.dll
            regSvcs.UnregisterAssembly(coAsm);
            MessageBox.Show("类库卸载完毕！");
        }

        // 获取安装路径和 DLL 文件路径
        public (string GetCoPath, string GetOpPath) GetDllPath()
        {
            // 获取原始安装路径，它可能包含末尾反斜杠，也可能因 CustomActionData 解析问题而有双反斜杠
            var setupDirRaw = this.Context.Parameters["targetdir"];
            // 使用 GetFullPath 规范化路径。它可以处理双反斜杠、相对路径等问题。
            string setupDir = Path.GetFullPath(setupDirRaw);
            // 检查路径是否为空（虽然 Installer Project 通常会提供，但防御性检查是好的）
            if (string.IsNullOrEmpty(setupDir))
            {
                // 在自定义操作中抛出异常会导致安装回滚
                // 如果是在 Commit 或 OnAfterInstall，可能不会导致回滚，但仍然不推荐
                // 更好的做法是记录错误或显示一个警告
                MessageBox.Show("错误：未能获取安装路径！", "安装错误", MessageBoxButtons.OK, MessageBoxIcon.Error);
                //return; // 退出方法
            }
            // 经过 Path.GetFullPath 处理后，setupDir 通常会是一个规范化的路径，通常不包含多余的末尾反斜杠
            // 但为了安全起见，确保它以反斜杠结尾（对于目录路径通常是需要的）
            if (!setupDir.EndsWith(Path.DirectorySeparatorChar.ToString()))
            {
                setupDir += Path.DirectorySeparatorChar;
            }
            MessageBox.Show($"读取到安装路径: {setupDir}");
            // 使用 Path.Combine() 拼接目录和文件名。
            // Path.Combine 会智能处理分隔符，如果 setupDir 已经是规范的，这里就不会出现双反斜杠。
            string coFileName = "CapeOpen.dll";
            string opFileName = "MyMixerTest.dll";
            // 使用 Path.Combine() 拼接目录和文件名
            string coPath = Path.Combine(setupDir, coFileName);
            string opPath = Path.Combine(setupDir, opFileName);
            var pathInfo = new string[]{
                $"安装路径: {setupDir}",
                $"CapeOpen.dll的完整路径: {coPath}",
                $"MyMixerTest.dll的完整路径: {opPath}"
            };
            MessageBox.Show(string.Join(Environment.NewLine, pathInfo),
                "路径信息", MessageBoxButtons.OK, MessageBoxIcon.Information);
            // 测试文件是否存在
            if (File.Exists(coPath))
            {
                MessageBox.Show("文件存在。");
            }
            else
            {
                MessageBox.Show("文件不存在。");
            }

            return (coPath, opPath);
        }
    }
}
