using RadishFlow.CapeOpen.Adapter;
using RadishFlow.CapeOpen.Interop.Common;
using RadishFlow.CapeOpen.Interop.Errors;
using RadishFlow.CapeOpen.Interop.Ole;
using RadishFlow.CapeOpen.Interop.Parameters;
using RadishFlow.CapeOpen.Interop.Persistence;
using RadishFlow.CapeOpen.Interop.Unit;
using RadishFlow.CapeOpen.UnitOp.Mvp.Placeholders;
using RadishFlow.CapeOpen.UnitOp.Mvp.Results;
using System.Runtime.InteropServices;
using System.Text.Json;

namespace RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation;

public sealed partial class RadishFlowCapeOpenUnitOperation
{
    public int GetClassID(out Guid classId)
    {
        UnitOperationComTrace.Write(nameof(GetClassID), "enter");
        try
        {
            classId = Guid.Parse(UnitOperationComIdentity.ClassId);
            UnitOperationComTrace.Write(nameof(GetClassID), "result", classId.ToString("D"));
            return ComHResults.SOk;
        }
        catch (Exception error)
        {
            classId = Guid.Empty;
            UnitOperationComTrace.Exception(nameof(GetClassID), error);
            return error.HResult;
        }
        finally
        {
            UnitOperationComTrace.Write(nameof(GetClassID), "exit");
        }
    }

    public int IsDirty()
    {
        UnitOperationComTrace.Write(nameof(IsDirty), "enter");
        try
        {
            UnitOperationComTrace.Write(nameof(IsDirty), "result", "S_FALSE");
            return ComHResults.SFalse;
        }
        catch (Exception error)
        {
            UnitOperationComTrace.Exception(nameof(IsDirty), error);
            return error.HResult;
        }
        finally
        {
            UnitOperationComTrace.Write(nameof(IsDirty), "exit");
        }
    }

    int IPersistStreamInit.Load(IntPtr stream)
    {
        UnitOperationComTrace.Write("IPersistStreamInit.Load", "enter", stream == IntPtr.Zero ? "stream=null" : "stream=provided");
        try
        {
            return ComHResults.SOk;
        }
        catch (Exception error)
        {
            UnitOperationComTrace.Exception("IPersistStreamInit.Load", error);
            return error.HResult;
        }
        finally
        {
            UnitOperationComTrace.Write("IPersistStreamInit.Load", "exit");
        }
    }

    int IPersistStreamInit.Save(IntPtr stream, bool clearDirty)
    {
        UnitOperationComTrace.Write(
            "IPersistStreamInit.Save",
            "enter",
            $"stream={(stream == IntPtr.Zero ? "null" : "provided")}; clearDirty={clearDirty}");
        try
        {
            return ComHResults.SOk;
        }
        catch (Exception error)
        {
            UnitOperationComTrace.Exception("IPersistStreamInit.Save", error);
            return error.HResult;
        }
        finally
        {
            UnitOperationComTrace.Write("IPersistStreamInit.Save", "exit");
        }
    }

    public int GetSizeMax(out long size)
    {
        UnitOperationComTrace.Write(nameof(GetSizeMax), "enter");
        try
        {
            size = 0;
            UnitOperationComTrace.Write(nameof(GetSizeMax), "result", "size=0");
            return ComHResults.SOk;
        }
        catch (Exception error)
        {
            size = 0;
            UnitOperationComTrace.Exception(nameof(GetSizeMax), error);
            return error.HResult;
        }
        finally
        {
            UnitOperationComTrace.Write(nameof(GetSizeMax), "exit");
        }
    }

    public int InitNew()
    {
        UnitOperationComTrace.Write(nameof(InitNew), "enter");
        try
        {
            return ComHResults.SOk;
        }
        catch (Exception error)
        {
            UnitOperationComTrace.Exception(nameof(InitNew), error);
            return error.HResult;
        }
        finally
        {
            UnitOperationComTrace.Write(nameof(InitNew), "exit");
        }
    }

    public int InitNew(IntPtr storage)
    {
        UnitOperationComTrace.Write(
            "IPersistStorage.InitNew",
            "enter",
            storage == IntPtr.Zero ? "storage=null" : "storage=provided");
        try
        {
            return ComHResults.SOk;
        }
        catch (Exception error)
        {
            UnitOperationComTrace.Exception("IPersistStorage.InitNew", error);
            return error.HResult;
        }
        finally
        {
            UnitOperationComTrace.Write("IPersistStorage.InitNew", "exit");
        }
    }

    public int Load(IntPtr storage)
    {
        UnitOperationComTrace.Write(
            "IPersistStorage.Load",
            "enter",
            storage == IntPtr.Zero ? "storage=null" : "storage=provided");
        try
        {
            return ComHResults.SOk;
        }
        catch (Exception error)
        {
            UnitOperationComTrace.Exception("IPersistStorage.Load", error);
            return error.HResult;
        }
        finally
        {
            UnitOperationComTrace.Write("IPersistStorage.Load", "exit");
        }
    }

    public int Save(IntPtr storage, bool sameAsLoad)
    {
        UnitOperationComTrace.Write(
            "IPersistStorage.Save",
            "enter",
            $"storage={(storage == IntPtr.Zero ? "null" : "provided")}; sameAsLoad={sameAsLoad}");
        try
        {
            return ComHResults.SOk;
        }
        catch (Exception error)
        {
            UnitOperationComTrace.Exception("IPersistStorage.Save", error);
            return error.HResult;
        }
        finally
        {
            UnitOperationComTrace.Write("IPersistStorage.Save", "exit");
        }
    }

    public int SaveCompleted(IntPtr storage)
    {
        UnitOperationComTrace.Write(
            nameof(SaveCompleted),
            "enter",
            storage == IntPtr.Zero ? "storage=null" : "storage=provided");
        try
        {
            return ComHResults.SOk;
        }
        catch (Exception error)
        {
            UnitOperationComTrace.Exception(nameof(SaveCompleted), error);
            return error.HResult;
        }
        finally
        {
            UnitOperationComTrace.Write(nameof(SaveCompleted), "exit");
        }
    }

    public int HandsOffStorage()
    {
        UnitOperationComTrace.Write(nameof(HandsOffStorage), "enter");
        try
        {
            return ComHResults.SOk;
        }
        catch (Exception error)
        {
            UnitOperationComTrace.Exception(nameof(HandsOffStorage), error);
            return error.HResult;
        }
        finally
        {
            UnitOperationComTrace.Write(nameof(HandsOffStorage), "exit");
        }
    }

    public int SetClientSite(IntPtr clientSite)
    {
        UnitOperationComTrace.Write(
            nameof(SetClientSite),
            "enter",
            clientSite == IntPtr.Zero ? "clientSite=null" : "clientSite=provided");
        try
        {
            ReplaceOleClientSite(clientSite);
            return ComHResults.SOk;
        }
        catch (Exception error)
        {
            UnitOperationComTrace.Exception(nameof(SetClientSite), error);
            return error.HResult;
        }
        finally
        {
            UnitOperationComTrace.Write(nameof(SetClientSite), "exit");
        }
    }

    public int GetClientSite(out IntPtr clientSite)
    {
        UnitOperationComTrace.Write(nameof(GetClientSite), "enter");
        try
        {
            clientSite = _oleClientSite;
            if (clientSite != IntPtr.Zero)
            {
                Marshal.AddRef(clientSite);
            }

            UnitOperationComTrace.Write(nameof(GetClientSite), "result", clientSite == IntPtr.Zero ? "clientSite=null" : "clientSite=provided");
            return ComHResults.SOk;
        }
        catch (Exception error)
        {
            clientSite = IntPtr.Zero;
            UnitOperationComTrace.Exception(nameof(GetClientSite), error);
            return error.HResult;
        }
        finally
        {
            UnitOperationComTrace.Write(nameof(GetClientSite), "exit");
        }
    }

    public int SetHostNames(string? containerApplication, string? containerObject)
    {
        UnitOperationComTrace.Write(
            nameof(SetHostNames),
            "enter",
            $"containerApplication={containerApplication ?? "<null>"}; containerObject={containerObject ?? "<null>"}");
        try
        {
            return ComHResults.SOk;
        }
        catch (Exception error)
        {
            UnitOperationComTrace.Exception(nameof(SetHostNames), error);
            return error.HResult;
        }
        finally
        {
            UnitOperationComTrace.Write(nameof(SetHostNames), "exit");
        }
    }

    public int Close(uint saveOption)
    {
        UnitOperationComTrace.Write(nameof(Close), "enter", $"saveOption={saveOption}");
        try
        {
            ReleaseOleClientSite();
            return ComHResults.SOk;
        }
        catch (Exception error)
        {
            UnitOperationComTrace.Exception(nameof(Close), error);
            return error.HResult;
        }
        finally
        {
            UnitOperationComTrace.Write(nameof(Close), "exit");
        }
    }

    public int SetMoniker(uint whichMoniker, IntPtr moniker)
    {
        UnitOperationComTrace.Write(
            nameof(SetMoniker),
            "enter",
            $"whichMoniker={whichMoniker}; moniker={(moniker == IntPtr.Zero ? "null" : "provided")}");
        return ComHResults.SOk;
    }

    public int GetMoniker(uint assign, uint whichMoniker, out IntPtr moniker)
    {
        UnitOperationComTrace.Write(nameof(GetMoniker), "enter", $"assign={assign}; whichMoniker={whichMoniker}");
        moniker = IntPtr.Zero;
        UnitOperationComTrace.Write(nameof(GetMoniker), "result", "moniker=null; E_NOTIMPL");
        return ComHResults.ENotImpl;
    }

    public int InitFromData(IntPtr dataObject, bool creation, uint reserved)
    {
        UnitOperationComTrace.Write(
            nameof(InitFromData),
            "enter",
            $"dataObject={(dataObject == IntPtr.Zero ? "null" : "provided")}; creation={creation}; reserved={reserved}");
        return ComHResults.SOk;
    }

    public int GetClipboardData(uint reserved, out IntPtr dataObject)
    {
        UnitOperationComTrace.Write(nameof(GetClipboardData), "enter", $"reserved={reserved}");
        dataObject = IntPtr.Zero;
        UnitOperationComTrace.Write(nameof(GetClipboardData), "result", "dataObject=null; E_NOTIMPL");
        return ComHResults.ENotImpl;
    }

    public int DoVerb(int verb, IntPtr message, IntPtr activeSite, int index, IntPtr parentWindow, IntPtr positionRectangle)
    {
        UnitOperationComTrace.Write(
            nameof(DoVerb),
            "enter",
            $"verb={verb}; activeSite={(activeSite == IntPtr.Zero ? "null" : "provided")}; parentWindow={(parentWindow == IntPtr.Zero ? "null" : "provided")}");
        return ComHResults.SOk;
    }

    public int EnumVerbs(out IntPtr enumOleVerb)
    {
        UnitOperationComTrace.Write(nameof(EnumVerbs), "enter");
        enumOleVerb = IntPtr.Zero;
        UnitOperationComTrace.Write(nameof(EnumVerbs), "result", "enumOleVerb=null; OLEOBJ_E_NOVERBS");
        return OleConstants.OleObjectNoVerbs;
    }

    public int Update()
    {
        UnitOperationComTrace.Write(nameof(Update), "enter");
        UnitOperationComTrace.Write(nameof(Update), "exit");
        return ComHResults.SOk;
    }

    public int IsUpToDate()
    {
        UnitOperationComTrace.Write(nameof(IsUpToDate), "enter");
        UnitOperationComTrace.Write(nameof(IsUpToDate), "exit");
        return ComHResults.SOk;
    }

    public int GetUserClassID(out Guid classId)
    {
        UnitOperationComTrace.Write(nameof(GetUserClassID), "enter");
        try
        {
            classId = Guid.Parse(UnitOperationComIdentity.ClassId);
            UnitOperationComTrace.Write(nameof(GetUserClassID), "result", classId.ToString("D"));
            return ComHResults.SOk;
        }
        catch (Exception error)
        {
            classId = Guid.Empty;
            UnitOperationComTrace.Exception(nameof(GetUserClassID), error);
            return error.HResult;
        }
        finally
        {
            UnitOperationComTrace.Write(nameof(GetUserClassID), "exit");
        }
    }

    public int GetUserType(uint formOfType, out IntPtr userType)
    {
        UnitOperationComTrace.Write(nameof(GetUserType), "enter", $"formOfType={formOfType}");
        try
        {
            userType = Marshal.StringToCoTaskMemUni(UnitOperationComIdentity.DisplayName);
            UnitOperationComTrace.Write(nameof(GetUserType), "result", UnitOperationComIdentity.DisplayName);
            return ComHResults.SOk;
        }
        catch (Exception error)
        {
            userType = IntPtr.Zero;
            UnitOperationComTrace.Exception(nameof(GetUserType), error);
            return error.HResult;
        }
        finally
        {
            UnitOperationComTrace.Write(nameof(GetUserType), "exit");
        }
    }

    public int SetExtent(uint drawAspect, ref OleSize size)
    {
        UnitOperationComTrace.Write(nameof(SetExtent), "enter", $"drawAspect={drawAspect}; width={size.Width}; height={size.Height}");
        _oleExtent = size;
        UnitOperationComTrace.Write(nameof(SetExtent), "exit");
        return ComHResults.SOk;
    }

    public int GetExtent(uint drawAspect, out OleSize size)
    {
        UnitOperationComTrace.Write(nameof(GetExtent), "enter", $"drawAspect={drawAspect}");
        size = _oleExtent;
        UnitOperationComTrace.Write(nameof(GetExtent), "result", $"width={size.Width}; height={size.Height}");
        UnitOperationComTrace.Write(nameof(GetExtent), "exit");
        return ComHResults.SOk;
    }

    public int Advise(IntPtr adviseSink, out uint connection)
    {
        UnitOperationComTrace.Write(nameof(Advise), "enter", adviseSink == IntPtr.Zero ? "adviseSink=null" : "adviseSink=provided");
        connection = 0;
        UnitOperationComTrace.Write(nameof(Advise), "result", "OLE_E_ADVISENOTSUPPORTED");
        return OleConstants.OleAdviseNotSupported;
    }

    public int Unadvise(uint connection)
    {
        UnitOperationComTrace.Write(nameof(Unadvise), "enter", $"connection={connection}");
        UnitOperationComTrace.Write(nameof(Unadvise), "result", "OLE_E_ADVISENOTSUPPORTED");
        return OleConstants.OleAdviseNotSupported;
    }

    public int EnumAdvise(out IntPtr enumAdvise)
    {
        UnitOperationComTrace.Write(nameof(EnumAdvise), "enter");
        enumAdvise = IntPtr.Zero;
        UnitOperationComTrace.Write(nameof(EnumAdvise), "result", "enumAdvise=null; OLE_E_ADVISENOTSUPPORTED");
        return OleConstants.OleAdviseNotSupported;
    }

    public int GetMiscStatus(uint aspect, out uint status)
    {
        UnitOperationComTrace.Write(nameof(GetMiscStatus), "enter", $"aspect={aspect}");
        status = OleConstants.OleMiscNone;
        UnitOperationComTrace.Write(nameof(GetMiscStatus), "result", $"status={status}");
        return ComHResults.SOk;
    }

    public int SetColorScheme(IntPtr logPalette)
    {
        UnitOperationComTrace.Write(nameof(SetColorScheme), "enter", logPalette == IntPtr.Zero ? "logPalette=null" : "logPalette=provided");
        UnitOperationComTrace.Write(nameof(SetColorScheme), "exit");
        return ComHResults.SOk;
    }
}
