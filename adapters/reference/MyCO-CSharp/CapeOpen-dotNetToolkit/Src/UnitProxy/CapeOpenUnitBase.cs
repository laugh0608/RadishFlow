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
using CAPEOPEN093;

namespace Mose.CapeOpenToolkit.UnitProxy
{
    public abstract class CapeOpenUnitBase : CapeOpenIdBase, ICapeUnit, ICapeUnitReport, ECapeUnknown, ECapeOutOfBounds, ECapeInvalidArgument, ECapeData, ECapeBadArgument, ECapeBadCOParameter, ECapeBadInvOrder, ECapeBoundaries, ECapeFailedInitialisation
    {
        private CapeOpenValidationStatus mStatus;
        public object mSimContext;

        public CapeOpenUnitBase()
        {
        }

        protected CapeOpenValidationStatus Status
        {
            get
            {
                return mStatus;
            }

            set
            {
                mStatus = value;
            }
        }


        #region ICapeUnit

        public CapeValidationStatus ValStatus
        {
            get
            {
                return CapeOpenValidationStatusHelper.Map(mStatus);
            }
        }

        public virtual bool Validate(ref string message)
        {
            return true;
        }

        public void Terminate()
        {
            //Vuota
        }

        public virtual object ports
        {
            get
            {
                return null;
            }
        }

        public object simulationContext
        {
            set
            {
                if (value == null)
                {
                    ATCOError = new CapeOpenError();
                    ATCOError.UserCode = (int)eCapeErrorInterfaceHR_tag.ECapeInvalidArgumentHR;
                    ATCOError.UserInterfaceName = "ICapeUnit";
                    ATCOError.UserOperation = "set_simulationContext";
                    ATCOError.UserDescription = "The input value is not a valid object";
                    throw (new COMException("Errore", (int)eCapeErrorInterfaceHR_tag.ECapeNoImplHR));
                }
                else
                {
                    mSimContext = value;
                }
            }
        }

        public void Initialize()
        {
        }

        protected abstract void OnInitialize();

        public void Calculate()
        {
            OnCalculate();
        }

        protected abstract void OnCalculate();

        //Non Implementata in VB
        public void Restore(ref object storage)
        {
            ATCOError = new CapeOpenError();
            ATCOError.UserCode = (int)eCapeErrorInterfaceHR_tag.ECapeNoImplHR;
            ATCOError.UserInterfaceName = "ICapeUnit";
            ATCOError.UserOperation = "Restore";
            ATCOError.UserDescription = "Restore method is not implemented - use COM persistence instead.";
            throw (new COMException("Errore", (int)eCapeErrorInterfaceHR_tag.ECapeNoImplHR));
        }

        public virtual object parameters
        {
            get
            {
                return null;
            }
        }


        public void Save(ref object storage)
        {
            ATCOError = new CapeOpenError();
            ATCOError.UserCode = (int)eCapeErrorInterfaceHR_tag.ECapeNoImplHR;
            ATCOError.UserInterfaceName = "ICapeUnit";
            ATCOError.UserOperation = "Save";
            ATCOError.UserDescription = "Save method is not implemented - use COM persistence instead.";
            throw (new COMException("Errore", (int)eCapeErrorInterfaceHR_tag.ECapeNoImplHR));
        }

        #endregion
        //Non implementata in VB, solleva sempre eccezione
        #region ICapeUnitReport

        public object reports
        {
            get
            {
                ATCOError = new CapeOpenError();
                ATCOError.UserCode = (int)eCapeErrorInterfaceHR_tag.ECapeNoImplHR;
                ATCOError.UserInterfaceName = "ICapeUnitReport";
                ATCOError.UserOperation = "get_Reports";
                ATCOError.UserDescription = "CAPE-OPEN reporting is not implemented in this Unit Operation.";
                throw (new COMException("Errore", (int)eCapeErrorInterfaceHR_tag.ECapeNoImplHR));
            }
        }

        public string selectedReport
        {
            get
            {
                ATCOError = new CapeOpenError();
                ATCOError.UserCode = (int)eCapeErrorInterfaceHR_tag.ECapeNoImplHR;
                ATCOError.UserInterfaceName = "ICapeUnitReport";
                ATCOError.UserOperation = "selectedReport";
                ATCOError.UserDescription = "CAPE-OPEN reporting is not implemented in this Unit Operation.";
                throw (new COMException("Errore", (int)eCapeErrorInterfaceHR_tag.ECapeNoImplHR));
            }
            set
            {
                ATCOError = new CapeOpenError();
                ATCOError.UserCode = (int)eCapeErrorInterfaceHR_tag.ECapeNoImplHR;
                ATCOError.UserInterfaceName = "ICapeUnitReport";
                ATCOError.UserOperation = "selectedReport";
                ATCOError.UserDescription = "CAPE-OPEN reporting is not implemented in this Unit Operation.";
                throw (new COMException("Errore", (int)eCapeErrorInterfaceHR_tag.ECapeNoImplHR));
            }
        }

        public void ProduceReport(ref string message)
        {
            ATCOError = new CapeOpenError();
            ATCOError.UserCode = (int)eCapeErrorInterfaceHR_tag.ECapeNoImplHR;
            ATCOError.UserInterfaceName = "ICapeUnitReport";
            ATCOError.UserOperation = "ProduceReport";
            ATCOError.UserDescription = "CAPE-OPEN reporting is not implemented in this Unit Operation.";
            throw (new COMException("Errore", (int)eCapeErrorInterfaceHR_tag.ECapeNoImplHR));
        }

        #endregion
        //OK
        #region ECapeBadArgument

        public int position
        {
            get
            {
                if (ATCOError != null) return ATCOError.ParaBadArgumentPos;
                return 0;
            }
        }

        #endregion
        //OK
        #region ECapeBadCOParameter

        public string parametername
        {
            get
            {
                if (ATCOError != null) return ATCOError.ParaBadParameterName;
                return null;
            }
        }

        public object parameter
        {
            get
            {
                if (ATCOError != null) return ATCOError.ParaBadParameterObj;
                return null;
            }
        }

        #endregion
        //OK
        #region ECapeBadInvOrder

        public string requestedOperation
        {
            get
            {
                if (ATCOError != null) return ATCOError.ParaBadInvOperation;
                return null;
            }
        }

        #endregion
        //OK
        #region ECapeBoundaries

        public double UpperBound
        {
            get
            {
                if (ATCOError != null) return ATCOError.UpperBoundary;
                return 0;
            }
        }

        public double LowerBound
        {
            get
            {
                if (ATCOError != null) return ATCOError.ParaLowBoundary;
                return 0;
            }
        }

        public double value
        {
            get
            {
                if (ATCOError != null) return ATCOError.ParaBoundaryVal;
                return 0;
            }
        }

        public string Type
        {
            get
            {
                if (ATCOError != null) return ATCOError.BoundaryType;
                return null;
            }
        }

        #endregion
    }
}
