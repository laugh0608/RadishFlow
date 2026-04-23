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
using System.Runtime.InteropServices;

namespace Mose.CapeOpenToolkit.UnitProxy
{
    public class SimplePortCollection : CapeOpenSimpleUnitCollection
    {
        public SimplePortCollection()
            : base()
        {
        }

        public void Add(CapeOpenUnitPort port)
        {
            List.Add(port);
        }

        public CapeOpenUnitPort this[int index]
        {
            get
            {
                return (CapeOpenUnitPort)List[index];
            }
        }

        public CapeOpenUnitPort GetPortByName(string name)
        {
            for (int i = 0; i < List.Count; i++)
            {
                CapeOpenUnitPort zPort = (CapeOpenUnitPort)List[i];
                if (zPort.Name == name)
                {
                    return (CapeOpenUnitPort)List[i];
                }
            }
            ATCOError = new CapeOpenError();
            ATCOError.UserCode = (int)CAPEOPEN093.eCapeErrorInterfaceHR_tag.ECapeInvalidArgumentHR;
            ATCOError.UserInterfaceName = "ICapeUnitCollection_Item";
            ATCOError.UserOperation = "Item";
            ATCOError.UserDescription = "The input value is a string but not rapresents a valid portname";
            throw new COMException("Errore", (int)CAPEOPEN093.eCapeErrorInterfaceHR_tag.ECapeInvalidArgumentHR);
        }

        protected override object OnItem(object id)
        {
            Int16 indice16;
            int indice32;
            if (!(id is int) && !(id is string) && !(id is Int16))
            {
                ATCOError = new CapeOpenError();
                ATCOError.UserCode = (int)CAPEOPEN093.eCapeErrorInterfaceHR_tag.ECapeInvalidArgumentHR;
                ATCOError.UserInterfaceName = "ICapeUnitCollection_Item";
                ATCOError.UserOperation = "Item";
                ATCOError.UserDescription = "The input value is not a valid number or string";
                throw (new COMException("Errore", (int)CAPEOPEN093.eCapeErrorInterfaceHR_tag.ECapeInvalidArgumentHR));
            }
            else
            {
                if (id is string)
                {
                    string str;
                    str = (string)id;
                    for (int i = 0; i < List.Count; i++)
                    {
                        CapeOpenUnitPort zPort = (CapeOpenUnitPort)List[i];
                        if (zPort.Name == str)
                        {
                            return List[i];
                        }
                    }
                    ATCOError = new CapeOpenError();
                    ATCOError.UserCode = (int)CAPEOPEN093.eCapeErrorInterfaceHR_tag.ECapeInvalidArgumentHR;
                    ATCOError.UserInterfaceName = "ICapeUnitCollection_Item";
                    ATCOError.UserOperation = "Item";
                    ATCOError.UserDescription = "The input value is a string but not rapresents a valid portname";
                    throw new COMException("Errore", (int)CAPEOPEN093.eCapeErrorInterfaceHR_tag.ECapeInvalidArgumentHR);
                }
                else
                {//int or int16
                    try
                    {
                        if (id is int)
                        {
                            indice32 = (int)id;
                            return List[indice32 - 1];
                        }
                        if (id is Int16)
                        {
                            indice16 = (Int16)id;
                            return List[indice16 - 1];
                        }
                        return null;
                    }
                    catch
                    {
                        ATCOError = new CapeOpenError();
                        ATCOError.UserCode = (int)CAPEOPEN093.eCapeErrorInterfaceHR_tag.ECapeInvalidArgumentHR;
                        ATCOError.UserInterfaceName = "ICapeUnitCollection_Item";
                        ATCOError.UserOperation = "Item";
                        ATCOError.UserDescription = "The input value is a integer but is out of index";
                        throw (new COMException("Errore", (int)CAPEOPEN093.eCapeErrorInterfaceHR_tag.ECapeInvalidArgumentHR));
                    }
                }

            }

        }
    }
}
