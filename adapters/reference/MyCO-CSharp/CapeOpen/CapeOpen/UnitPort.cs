using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;

namespace CapeOpen
{

    [System.Runtime.InteropServices.ComVisible(false)]
    class PortConverter : System.ComponentModel.ExpandableObjectConverter
    {
        public override bool CanConvertTo(System.ComponentModel.ITypeDescriptorContext context, System.Type destinationType)
        {
            if ((typeof(ICapeUnitPort)).IsAssignableFrom(destinationType))
                return true;
            return base.CanConvertTo(context, destinationType);
        }

        public override Object ConvertTo(System.ComponentModel.ITypeDescriptorContext context, System.Globalization.CultureInfo culture, Object value, System.Type destinationType)
        {
            if ((typeof(System.String)).IsAssignableFrom(destinationType) && (typeof(ICapeUnitPort).IsAssignableFrom(value.GetType())))
            {
                object connectedObject = ((ICapeUnitPort)value).connectedObject;
                if (connectedObject != null)
                {
                    return ((ICapeIdentification)connectedObject).ComponentName;
                }
                return "No Connected Object.";
            }
            return base.ConvertTo(context, culture, value, destinationType);
        }
    };

    /// <summary>
    /// Represents the method that will handle connecting an object to a unit port.
    /// </summary>
    [System.Runtime.InteropServices.ComVisibleAttribute(false)]
    public delegate void PortConnectedHandler(Object sender, PortConnectedEventArgs args);

    /// <summary>
    /// </summary>
    /// <remarks>
    /// </remarks>
    [System.Runtime.InteropServices.InterfaceType(System.Runtime.InteropServices.ComInterfaceType.InterfaceIsIDispatch)]
    [System.Runtime.InteropServices.ComVisibleAttribute(true)]
    [System.Runtime.InteropServices.GuidAttribute("3530B780-5E59-42B1-801B-3C18F2AD08EE")]
    [System.ComponentModel.DescriptionAttribute("CapeRealParameterEvents Interface")]
    interface IUnitPortEvents
    {
        /// <summary>
        /// Occurs when the user connects a new object to a unit port.
        /// </summary>
        /// <remarks><para>Raising an event invokes the event handler through a delegate.</para>
        /// <para>The <c>OnPortConnected</c> method also allows derived classes to handle the event without attaching a delegate. This is the preferred 
        /// technique for handling the event in a derived class.</para>
        /// <para>Notes to Inheritors: </para>
        /// <para>When overriding <c>OnPortConnected</c> in a derived class, be sure to call the base class's <c>OnPortConnected</c> method so that registered 
        /// delegates receive the event.</para>
        /// </remarks>
        /// <param name = "sender">The <see cref = "UnitPort">CapeUnitPort</see> that raised the event.</param>
        /// <param name = "args">A <see cref = "PortConnectedEventArgs">ParameterValueChangedEventArgs</see> that contains information about the event.</param>
        void PortConnected(UnitPort sender, PortConnectedEventArgs args);

        /// <summary>
        /// Occurs when the user disconnets a object from a unit port.
        /// </summary>
        /// <remarks><para>Raising an event invokes the event handler through a delegate.</para>
        /// <para>The <c>OnPortDisconnected</c> method also allows derived classes to handle the event without attaching a delegate. This is the preferred 
        /// technique for handling the event in a derived class.</para>
        /// <para>Notes to Inheritors: </para>
        /// <para>When overriding <c>OnPortDisconnected</c> in a derived class, be sure to call the base class's <c>OnPortDisconnected</c> method so that registered 
        /// delegates receive the event.</para>
        /// </remarks>
        /// <param name = "sender">The <see cref = "UnitPort">CapeUnitPort</see> that raised the event.</param>
        /// <param name = "args">A <see cref = "PortDisconnectedEventArgs">ParameterValueChangedEventArgs</see> that contains information about the event.</param>
        void PortDisconnected(UnitPort sender, PortDisconnectedEventArgs args);
    }

    /// <summary>
    /// This class represents the behaviour of a Unit 
    ///	Operation connection point (Unit Operation Port). It provides different 
    ///	attributes for configuring the port as well as to connect 
    ///	it to a material, energy or information object.
    /// </summary>
    /// <remarks>
    /// <para>
    /// The unit port provides the the means by which a Flowsheet Unit is connected to its streams. 
    /// Streams are implemented by means of material objects.
    /// </para>
    /// <para>
    /// The three types of port: material, energy and 
    ///	information, have a lot of functionality in common. By combining the three into one we can simplify 
    ///	the interface to a useful degree. Each port type is to be distinguished by the value of an attribute.
    /// </para>
    /// </remarks>
    [Serializable]
    [System.Runtime.InteropServices.ComVisible(true)]
    [System.Runtime.InteropServices.ComSourceInterfaces(typeof(IUnitPortEvents))]
    [System.Runtime.InteropServices.Guid("51066F52-C0F9-48d7-939E-3A229010E77C")]//ICapeThermoMaterialObject_IID)
    [System.ComponentModel.Description("")]
    [System.Runtime.InteropServices.ClassInterface(System.Runtime.InteropServices.ClassInterfaceType.None)]
    [System.ComponentModel.TypeConverter(typeof(PortConverter))]
    public class UnitPort : CapeIdentification,
        ICapeUnitPort,
        ICapeUnitPortCOM
    {
        private CapePortDirection m_Direction;
        private CapePortType m_Type;
        private Object m_ConnectedObject;
        private bool isConnectObjectSerializable;
        [NonSerialized()]
        private Object m_ConnectedNonSerializableObject;

        /// <summary>
        /// Initializes a new instance of the <see cref="UnitPort"/> class.
        /// </summary>
        /// <param name="Name"><see cref="ICapeIdentification.ComponentName"/> of the <see cref="UnitPort"/>.</param>
        /// <param name="Description"><see cref="ICapeIdentification.ComponentDescription"/> of the <see cref="UnitPort"/>.</param>
        /// <param name="Direction"><see cref="CapePortDirection"/> of the <see cref="UnitPort"/></param>
        /// <param name="Type"><see cref="CapePortType"/> of the <see cref="UnitPort"/></param>
        public UnitPort(String Name, String Description, CapePortDirection Direction, CapePortType Type)
            : base(Name, Description)
        {
            m_Direction = Direction;
            m_Type = Type;
            m_ConnectedObject = null;
            m_ConnectedNonSerializableObject = null;
            isConnectObjectSerializable = false;
        }

        /// <summary>
        /// Finalizer for the <see cref = "UnitPort"/> class.
        /// </summary>
        /// <remarks>
        /// This will finalize the current instance of the class.
        /// </remarks>
        ~UnitPort()
        {
            this.Disconnect(false);
        }

        /// <summary>Creates a new object that is a copy of the current instance.</summary>
        /// <remarks>
        /// <para>
        /// Clone can be implemented either as a deep copy or a shallow copy. In a deep copy, all objects are duplicated; 
        /// in a shallow copy, only the top-level objects are duplicated and the lower levels contain references.
        /// </para>
        /// <para>
        /// The resulting clone must be of the same type as, or compatible with, the original instance.
        /// </para>
        /// <para>
        /// See <see cref="Object.MemberwiseClone"/> for more information on cloning, deep versus shallow copies, and examples.
        /// </para>
        /// </remarks>
        /// <returns>A new object that is a copy of this instance.</returns>
        override public object Clone()
        {
            return new UnitPort(this.ComponentName, this.ComponentDescription, this.direction, this.portType);
        }

        /// <summary>
        /// Occurs when the user connects a new object to the port.
        /// </summary>
        public event PortConnectedHandler PortConnected;
        /// <summary>
        /// Occurs when the user connects a new object to the port.
        /// </summary>
        /// <remarks><para>Raising an event invokes the event handler through a delegate.</para>
        /// <para>The <c>OnPortConencted</c> method also allows derived classes to handle the event without attaching a delegate. This is the preferred 
        /// technique for handling the event in a derived class.</para>
        /// <para>Notes to Inheritors: </para>
        /// <para>When overriding <c>OnPortConencted</c> in a derived class, be sure to call the base class's <c>OnPortConnected</c> method so that registered 
        /// delegates receive the event.</para>
        /// </remarks>
        /// <param name = "args">A <see cref = "PortConnectedEventArgs">PortConnectedEventArgs</see> that contains information about the event.</param>
        protected void OnPortConnected(PortConnectedEventArgs args)
        {
            if (PortConnected != null)
            {
                PortConnected(this, args);
            }
        }

        /// <summary>
        /// Occurs when the user disconnects an object from the port.
        /// </summary>
        public event PortDisconnectedHandler PortDisconnected;
        /// <summary>
        /// Occurs when the user disconnects an object from the port.
        /// </summary>
        /// <remarks><para>Raising an event invokes the event handler through a delegate.</para>
        /// <para>The <c>OnPortDisconencted</c> method also allows derived classes to handle the event without attaching a delegate. This is the preferred 
        /// technique for handling the event in a derived class.</para>
        /// <para>Notes to Inheritors: </para>
        /// <para>When overriding <c>OnPortDisconencted</c> in a derived class, be sure to call the base class's <c>OnPortConnected</c> method so that registered 
        /// delegates receive the event.</para>
        /// </remarks>
        /// <param name = "args">A <see cref = "PortDisconnectedEventArgs">PortDisconnectedEventArgs</see> that contains information about the event.</param>
        protected void OnPortDisconnected(PortDisconnectedEventArgs args)
        {
            if (PortDisconnected != null)
            {
                PortDisconnected(this, args);
            }
        }


        //ICapeUnitPort
        /// <summary>
        ///	<para> Returns the type of this port and allows the developer to change 
        /// the port type (allowed types are among the ones included in the CapePortType type.</para>
        /// </summary>
        /// <see cref="CapePortType">CapePortType</see> 
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeFailedInitialisation">ECapeFailedInitialisation</exception>
        [System.ComponentModel.CategoryAttribute("ICapeUnitPort")]
        public CapePortType portType
        {
            get
            {
                return m_Type;
            }
            set
            {
                m_Type = value;
                NotifyPropertyChanged("portType");
            }
        }

        /// <summary>
        ///	Returns the direction of this port and allows the developer to change 
        /// the port direction. Allowed values are among those included 
        /// in the CapePortDirection type).
        /// </summary>
        /// <see cref="CapePortDirection">CapePortDirection</see>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeFailedInitialisation">ECapeFailedInitialisation</exception>
        [System.ComponentModel.CategoryAttribute("ICapeUnitPort")]
        public CapePortDirection direction
        {
            get
            {
                return m_Direction;
            }
            set
            {
                m_Direction = value;
                NotifyPropertyChanged("direction");
            }
        }

        /// <summary>
        ///	Returns to the client the object that is connected to this port.
        /// </summary>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeFailedInitialisation">ECapeFailedInitialisation</exception>
        Object ICapeUnitPortCOM.connectedObject
        {
            get
            {
                if (!isConnectObjectSerializable)
                {
                    if (m_ConnectedNonSerializableObject != null)
                    {
                        if (typeof(MaterialObjectWrapper).IsAssignableFrom(m_ConnectedNonSerializableObject.GetType()))
                        {
                            return ((MaterialObjectWrapper)m_ConnectedNonSerializableObject).MaterialObject10;
                        }
                    }
                    return m_ConnectedNonSerializableObject;
                }
                return m_ConnectedObject;
            }
        }

        /// <summary>
        ///	Returns to the client the object that is connected to this port.
        /// </summary>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeFailedInitialisation">ECapeFailedInitialisation</exception>
        virtual public Object connectedObject
        {
            get
            {
                if (isConnectObjectSerializable) return m_ConnectedObject;
                return m_ConnectedNonSerializableObject;
            }
        }

        /// <summary>
        ///	Connects an object to the port. For a material port it must 
        /// be an object implementing the ICapeThermoMaterialObject interface, 
        /// for Energy and Information ports it must be an object implementing 
        /// the ICapeParameter interface. 
        /// </summary>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        virtual public void Connect(Object objectToConnect)
        {
            this.Disconnect(true);
            PortConnectedEventArgs args = new PortConnectedEventArgs(this.ComponentName);
            if (objectToConnect.GetType().IsCOMObject)
            {
                //If the port is a material port
                if (m_Type == CapePortType.CAPE_MATERIAL || m_Type == CapePortType.CAPE_ANY)
                {
                    // does the material object support both thermo 1.0 and 1.1?
                    if (objectToConnect is ICapeThermoMaterialObjectCOM && objectToConnect is ICapeThermoMaterialCOM)
                    {
                        //use the material wrapper that exposes both interfaces.
                        m_ConnectedNonSerializableObject = new MaterialObjectWrapper(objectToConnect);
                        m_ConnectedObject = null;
                        isConnectObjectSerializable = false;
                        OnPortConnected(args);
                        NotifyPropertyChanged("connectedObject");
                        return;
                    }
                    // Does the material only support thermo 1.1?
                    else if (objectToConnect is ICapeThermoMaterialCOM)
                    {
                        //then only use the material wrapper that exposes thermo 1.1
                        m_ConnectedNonSerializableObject = new MaterialObjectWrapper11(objectToConnect);
                        m_ConnectedObject = null;
                        isConnectObjectSerializable = false;
                        OnPortConnected(args);
                        NotifyPropertyChanged("connectedObject");
                        return;
                    }
                    //Does the thermo only support thermo 1.0?
                    else if (objectToConnect is ICapeThermoMaterialObjectCOM)
                    {
                        // then use the wrapper that supports thermo 1.0.
                        m_ConnectedNonSerializableObject = new MaterialObjectWrapper1(objectToConnect);
                        m_ConnectedObject = null;
                        isConnectObjectSerializable = false;
                        OnPortConnected(args);
                        NotifyPropertyChanged("connectedObject");
                        return;
                    }
                    //If we get here, the object to connect is not a material.
                    //The object will be connected as-is. 
                    m_ConnectedNonSerializableObject = objectToConnect;
                    m_ConnectedObject = null;
                    isConnectObjectSerializable = false;
                    OnPortConnected(args);
                    NotifyPropertyChanged("connectedObject");
                    return;
                }
            }
            if (objectToConnect.GetType().IsSerializable)
            {
                m_ConnectedObject = objectToConnect;
                isConnectObjectSerializable = true;
                m_ConnectedNonSerializableObject = null;
                OnPortConnected(args);
                NotifyPropertyChanged("connectedObject");
                return;
            }
            m_ConnectedNonSerializableObject = objectToConnect;
            m_ConnectedObject = null;
            isConnectObjectSerializable = false;
            OnPortConnected(args);
            NotifyPropertyChanged("connectedObject");
        }

        /// <summary>
        ///	Disconnects whatever object is connected to this port.
        /// </summary>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        virtual public void Disconnect()
        {
            this.Disconnect(false);
        }

        /// <summary>
        ///	Disconnects whatever object is connected to this port.
        /// </summary>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        private void Disconnect(bool connecting)
        {
            if (m_ConnectedNonSerializableObject != null)
            {
                if (m_ConnectedNonSerializableObject.GetType().IsCOMObject)
                {
                    System.Runtime.InteropServices.Marshal.FinalReleaseComObject(m_ConnectedNonSerializableObject);
                }
                if (m_ConnectedNonSerializableObject is MaterialObjectWrapper)
                    ((MaterialObjectWrapper)m_ConnectedNonSerializableObject).Dispose();
                m_ConnectedNonSerializableObject = null;
                return;
            }
            m_ConnectedObject = null;
            PortDisconnectedEventArgs args = new PortDisconnectedEventArgs(this.ComponentName);
            OnPortDisconnected(args);
            if (!connecting) NotifyPropertyChanged("connectedObject");
        }
    };
}