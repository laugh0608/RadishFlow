using System.Runtime.InteropServices;
using CAPEOPEN;

namespace WPFUnit
{
    [ComVisible(true)]
    [Guid("A4E2182E-1458-4222-A895-332EE07594FC")]
    [InterfaceType(ComInterfaceType.InterfaceIsDual)]
    public interface ICapeUnitOperation : ICapeUnit, ICapeUtilities
    {
        dynamic ports { get; }
        CapeValidationStatus ValStatus { get; }
        dynamic parameters { get; }
        dynamic simulationContext { set; }
        string name { get; }
        int code { get; }
        string description { get; }
        string interfaceName { get; }
        string moreInfo { get; }
        string operation { get; }
        string scope { get; }
        string ComponentDescription { get; set; }
        string ComponentName { get; set; }
        void Calculate();
        bool Validate(ref string message);
        void Initialize();
        void Terminate();
        int Edit();
    }
}