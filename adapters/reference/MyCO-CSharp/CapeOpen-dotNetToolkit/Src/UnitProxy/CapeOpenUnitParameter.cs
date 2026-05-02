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
using CAPEOPEN093;
using System.Runtime.InteropServices;

namespace Mose.CapeOpenToolkit.UnitProxy
{
    public abstract class CapeOpenUnitParameter : CapeOpenIdBase, ICapeParameter
    {
        protected CapeOpenParameterMode mParameterMode;
        protected object mValue;
        protected CapeOpenValidationStatus mStatus;
        protected ICapeParameterSpec mParameterSpec;

        public CapeOpenParameterMode ParameterMode
        {
            get
            {
                //LogClass.WriteLog("ParameterMode get");
                return mParameterMode;
            }
            set
            {
                //LogClass.WriteLog("Parameter Mode set");
                mParameterMode = value;
            }
        }

        public ICapeParameterSpec ParameterSpec
        {
            get
            {
                //LogClass.WriteLog("Parameter Spec get");
                return mParameterSpec;
            }
            set
            {
                //LogClass.WriteLog("Parameter Spec set");
                mParameterSpec = value;
            }
        }

        public CapeOpenValidationStatus Status
        {
            get
            {
                //LogClass.WriteLog("Status get");
                return mStatus;
            }
            set
            {
                //LogClass.WriteLog("status set");
                mStatus = value;
            }

        }

        protected CapeOpenUnitParameter()
        {
        }



        #region ICapeParameter

        public CapeValidationStatus ValStatus
        {
            get
            {
                //LogClass.WriteLog("Val status get");
                return CapeOpenValidationStatusHelper.Map(Status);
            }
        }

        public bool Validate(ref string message)
        {
            //LogClass.WriteLog("Validate Parameter");
            //'validate using the spec object
            bool zValidate;

            if (mParameterSpec.Type == CapeParamType.CAPE_REAL)
            {
                ICapeRealParameterSpec zRealSpec;
                zRealSpec = (ICapeRealParameterSpec)mParameterSpec;
                //Forzatura sul tipo
                zValidate = zRealSpec.Validate((double)mValue, ref message);
            }
            else
            {
                if (mParameterSpec.Type == CapeParamType.CAPE_INT)
                {
                    ICapeIntegerParameterSpec zIntSpec;
                    zIntSpec = (ICapeIntegerParameterSpec)mParameterSpec;
                    //Forzatura sul tipo
                    zValidate = zIntSpec.Validate((int)mValue, ref message);
                }
                else
                {
                    if (mParameterSpec.Type == CapeParamType.CAPE_OPTION)
                    {
                        ICapeOptionParameterSpec zOptionSpec;
                        zOptionSpec = (ICapeOptionParameterSpec)mParameterSpec;
                        zValidate = zOptionSpec.Validate((string)mValue, ref message);
                    }
                    else
                    {
                        zValidate = false;
                    }
                }
            }

            if (zValidate == true)
            {
                Status = CapeOpenValidationStatus.Valid;
                return true;
            }
            else
            {
                Status = CapeOpenValidationStatus.Invalid;
                return false;
            }
        }

        public void Reset()
        {
            //LogClass.WriteLog("Reset");
            if (mParameterSpec != null)
            {
                if (mParameterSpec.Type == CapeParamType.CAPE_REAL)
                {
                    ICapeRealParameterSpec zRealSpec;
                    zRealSpec = (ICapeRealParameterSpec)mParameterSpec;
                    mValue = zRealSpec.DefaultValue;
                }
                else
                {
                    if (mParameterSpec.Type == CapeParamType.CAPE_INT)
                    {
                        ICapeIntegerParameterSpec zIntSpec;
                        zIntSpec = (ICapeIntegerParameterSpec)mParameterSpec;
                        mValue = zIntSpec.DefaultValue;
                    }
                    else
                    {
                        if (mParameterSpec.Type == CapeParamType.CAPE_OPTION)
                        {
                            ICapeOptionParameterSpec zOptionSpec;
                            zOptionSpec = (ICapeOptionParameterSpec)mParameterSpec;
                            mValue = zOptionSpec.DefaultValue;
                        }
                    }
                }
            }
        }

        public object Specification
        {
            get
            {
              //  LogClass.WriteLog("Specification get");
                return mParameterSpec;
            }
        }

        public CapeParamMode Mode
        {
            get
            {
                //LogClass.WriteLog("Mode get " + mParameterMode.ToString());
                return CapeOpenParameterModeHelper.Map(mParameterMode);
            }
            set
            {
                //LogClass.WriteLog("Mode set");
                if ((value != CapeOpenParameterModeHelper.Map(CapeOpenParameterMode.Read)) && (value != CapeOpenParameterModeHelper.Map(CapeOpenParameterMode.ReadWrite)) && (value != CapeOpenParameterModeHelper.Map(CapeOpenParameterMode.Write)))
                {
                    ATCOError = new CapeOpenError();
                    ATCOError.UserCode = (int)eCapeErrorInterfaceHR_tag.ECapeInvalidArgumentHR;
                    ATCOError.UserInterfaceName = "ICapeParameter";
                    ATCOError.UserOperation = "set_mode";
                    ATCOError.UserDescription = "The input value is not a valid mode";
                    throw (new COMException("Errore", (int)eCapeErrorInterfaceHR_tag.ECapeInvalidArgumentHR));
                }
                else
                {
                    mParameterMode = CapeOpenParameterModeHelper.Map(value);
                }
            }
        }

        public object value
        {
            get
            {
                return mValue;
            }
            set
            {
                if (mParameterSpec.Type == CapeParamType.CAPE_INT)
                {
                    if (!(value is int) && !(value is long))
                    {
                        ATCOError = new CapeOpenError();
                        ATCOError.UserCode = (int)eCapeErrorInterfaceHR_tag.ECapeInvalidArgumentHR;
                        ATCOError.UserInterfaceName = "ICapeParameter";
                        ATCOError.UserOperation = "set_Value";
                        ATCOError.UserDescription = "The input value is not a integer number";
                        throw (new COMException("Errore", (int)eCapeErrorInterfaceHR_tag.ECapeInvalidArgumentHR));
                    }
                }
                else
                {
                    if (mParameterSpec.Type == CapeParamType.CAPE_REAL)
                    {
                        //LogClass.WriteLog("Parameter spec è CAPE_REAL");
                        //LogClass.WriteLog("Il valore è di tipo " + value.GetType().ToString());
                        if (!(value is double) && !(value is Single))
                        {
                            ATCOError = new CapeOpenError();
                            ATCOError.UserCode = (int)eCapeErrorInterfaceHR_tag.ECapeInvalidArgumentHR;
                            ATCOError.UserInterfaceName = "ICapeParameter";
                            ATCOError.UserOperation = "set_Value";
                            ATCOError.UserDescription = "The input value is not a double number";
                            throw (new COMException("Errore", (int)eCapeErrorInterfaceHR_tag.ECapeInvalidArgumentHR));
                        }
                    }
                    else
                    {
                        if (mParameterSpec.Type == CapeParamType.CAPE_OPTION)
                        {
                            if (!(value is string))
                            {
                                ATCOError = new CapeOpenError();
                                ATCOError.UserCode = (int)eCapeErrorInterfaceHR_tag.ECapeInvalidArgumentHR;
                                ATCOError.UserInterfaceName = "ICapeParameter";
                                ATCOError.UserOperation = "set_Value";
                                ATCOError.UserDescription = "The input value is not a string";
                                throw (new COMException("Errore", (int)eCapeErrorInterfaceHR_tag.ECapeInvalidArgumentHR));
                            }
                        }
                    }
                }
                mValue = value;
                mStatus = CapeOpenValidationStatus.NotYetValidated;
            }
        }

        #endregion

    }
}
