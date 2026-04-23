#region Copyright (c) 2005-2013, Mose, All rights reserved.
/*
// Copyright (c) 2005-2013, Mose Srl (http://www.mose.units.it/)
// Original capeopentoolkit.net Source Code: Copyright (c) 2005, Marco Carone, Marco Parenzan (e-mail: marco.carone@yahoo.it; marco.parenzan@libero.it)
// All rights reserved.
//  
// Redistribution and use in source and binary forms, with or without modification, are permitted 
// provided that the following conditions are met: 
//  
// (1) Redistributions of source code must retain the above copyright notice, this list of 
// conditions and the following disclaimer. 
// (2) Redistributions in binary form must reproduce the above copyright notice, this list of 
// conditions and the following disclaimer in the documentation and/or other materials 
// provided with the distribution. 
// (3) Neither the name of the Mose nor the names of its contributors may be used 
// to endorse or promote products derived from this software without specific prior 
// written permission.
//      
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND ANY EXPRESS 
// OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY 
// AND FITNESS FOR A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT OWNER OR 
// CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL 
// DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, 
// DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER 
// IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT 
// OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
// -------------------------------------------------------------------------
//
// Original capeopentoolkit.net Source Code: Copyright (c) 2005, Marco Carone, Marco Parenzan (e-mail: marco.carone@yahoo.it; marco.parenzan@libero.it)
// 
// Mose is a registered trademark of Mose Srl.
// 
// For portions of this software, the some additional copyright notices may apply 
// which can either be found in the license.txt file included in the source distribution
// or following this notice. 
//
*/
#endregion

using System;
using System.Collections.Generic;
using System.Text;
using System.IO;
using System.Reflection;
using System.Runtime.InteropServices;

using Mose.CapeOpenToolkit.UnitProxy;

namespace Mose.CapeOpenToolkit.UnitProxy
{
    public class COMRegistryInfoWriter
    {
        private Assembly mTargetAssembly;
        private Type mTargetUnitType;
        private Guid mUnitGuid;

        public COMRegistryInfoWriter(string targetAssemblyFileName, string targetUnitTypeName)
        {
            mTargetAssembly = Assembly.LoadFile(targetAssemblyFileName);
            mTargetUnitType = mTargetAssembly.GetType(targetUnitTypeName);
            GuidAttribute zUnitGuidAttribute = ((GuidAttribute[])mTargetUnitType.GetCustomAttributes(typeof(GuidAttribute), false))[0];
            mUnitGuid = new Guid(zUnitGuidAttribute.Value);
            mUnitGuid = new Guid(zUnitGuidAttribute.Value);
        }

        public COMRegistryInfoWriter(Assembly targetAssembly, string targetUnitTypeName)
        {
            mTargetAssembly = targetAssembly;
            mTargetUnitType = mTargetAssembly.GetType(targetUnitTypeName);
            GuidAttribute zUnitGuidAttribute = ((GuidAttribute[])mTargetUnitType.GetCustomAttributes(typeof(GuidAttribute), false))[0];
            mUnitGuid = new Guid(zUnitGuidAttribute.Value);
        }

        public COMRegistryInfoWriter(Assembly targetAssembly, Type targetUnitType)
        {
            mTargetAssembly = targetAssembly;
            mTargetUnitType = targetUnitType;
            GuidAttribute zUnitGuidAttribute = ((GuidAttribute[])mTargetUnitType.GetCustomAttributes(typeof(GuidAttribute), false))[0];
            mUnitGuid = new Guid(zUnitGuidAttribute.Value);
        }

        public void Write(Stream stream)
        {
            StreamWriter zWriter = new StreamWriter(stream, Encoding.ASCII);

            //
            //  backslash to slash
            //
            string zNormalizedTargetAssemblyFileName = mTargetAssembly.Location.Replace("\\", "/");

            zWriter.WriteLine("REGEDIT4");
            zWriter.WriteLine();

            zWriter.WriteLine("[HKEY_CLASSES_ROOT\\" + mTargetUnitType.FullName + "]");
            zWriter.WriteLine("@=\"" + mTargetUnitType.FullName + "\"");
            zWriter.WriteLine();

            zWriter.WriteLine("[HKEY_CLASSES_ROOT\\" + mTargetUnitType.FullName + "\\CLSID]");
            zWriter.WriteLine("@=\"{" + mUnitGuid.ToString() + "}\"");
            zWriter.WriteLine();

            zWriter.WriteLine("[HKEY_CLASSES_ROOT\\CLSID\\{" + mUnitGuid.ToString() + "}]");
            zWriter.WriteLine("@=\"" + mTargetUnitType.FullName + "\"");
            zWriter.WriteLine();

            zWriter.WriteLine("[HKEY_CLASSES_ROOT\\CLSID\\{" + mUnitGuid.ToString() + "}\\InprocServer32]");
            zWriter.WriteLine("@=\"mscoree.dll\"");
            zWriter.WriteLine("\"ThreadingModel\"=\"Both\"");
            zWriter.WriteLine("\"Class\"=\"" + mTargetUnitType.FullName + "\"");
            zWriter.WriteLine("\"Assembly\"=\"" + mTargetAssembly.FullName + "\"");
            zWriter.WriteLine("\"RuntimeVersion\"=\"v2.0.50727\"");
            zWriter.WriteLine("\"CodeBase\" = \"file:///" + zNormalizedTargetAssemblyFileName + "\"");
            zWriter.WriteLine();

            zWriter.WriteLine("[HKEY_CLASSES_ROOT\\CLSID\\{" + mUnitGuid.ToString() + "}\\InprocServer32\\1.0.0.0]");
            zWriter.WriteLine("\"Class\"=\"" + mTargetUnitType.FullName + "\"");
            zWriter.WriteLine("\"Assembly\"=\"" + mTargetAssembly.FullName + "\"");
            zWriter.WriteLine("\"RuntimeVersion\"=\"v2.0.50727\"");
            zWriter.WriteLine("\"CodeBase\" = \"file:///" + zNormalizedTargetAssemblyFileName + "\"");
            zWriter.WriteLine();

            zWriter.WriteLine("[HKEY_CLASSES_ROOT\\CLSID\\{" + mUnitGuid.ToString() + "}\\ProgId]");
            zWriter.WriteLine("@=\"" + mTargetUnitType.FullName + "\"");
            zWriter.WriteLine();

            zWriter.WriteLine("[HKEY_CLASSES_ROOT\\CLSID\\{" + mUnitGuid.ToString() + "}\\Implemented Categories\\{62C8FE65-4EBB-45E7-B440-6E39B2CDBF29}]");
            zWriter.WriteLine("[HKEY_CLASSES_ROOT\\CLSID\\{" + mUnitGuid.ToString() + "}\\Implemented Categories\\{678C09A5-7D66-11D2-A67D-00105A42887F}]");
            zWriter.WriteLine("[HKEY_CLASSES_ROOT\\CLSID\\{" + mUnitGuid.ToString() + "}\\Implemented Categories\\{0DE86A53-2BAA-11CF-A229-00AA003D7352}]");
            zWriter.WriteLine("[HKEY_CLASSES_ROOT\\CLSID\\{" + mUnitGuid.ToString() + "}\\Implemented Categories\\{0DE86A57-2BAA-11CF-A229-00AA003D7352}]");
            zWriter.WriteLine("[HKEY_CLASSES_ROOT\\CLSID\\{" + mUnitGuid.ToString() + "}\\Implemented Categories\\{40FC6ED5-2438-11CF-A3DB-080036F12502}]");
            zWriter.WriteLine();

            zWriter.WriteLine("[HKEY_CLASSES_ROOT\\CLSID\\{" + mUnitGuid.ToString() + "}\\CapeDescription]");

            CapeOpenUnitInfoAttribute[] zCapeOpenAttributes = (CapeOpenUnitInfoAttribute[])mTargetAssembly.GetCustomAttributes(typeof(CapeOpenUnitInfoAttribute), false);
            foreach (CapeOpenUnitInfoAttribute zAttribute in zCapeOpenAttributes)
            {
                zWriter.WriteLine(string.Format("\"{0}\"=\"{1}\"", zAttribute.CapeOpenUnitInfo.ToString(), zAttribute.CapeOpenInfoValue));
            }

            zWriter.WriteLine();
            zWriter.Flush();
        }
    }
}
