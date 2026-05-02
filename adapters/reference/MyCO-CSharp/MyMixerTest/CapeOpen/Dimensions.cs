using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Xml;

namespace CapeOpen
{

    /// <summary>
    /// Provides information related to a unit of measure associated with a parameter.
    /// </summary>
    /// <remarks>
    /// <para>The unit of maesure can be either an System Internation (SI) or customary unit. Each unit is assigned to a 
    /// <see cref = "unitCategory"/> that has information related to the dimensionality of the unit.</para>
    /// </remarks>
    struct unit
    {
        /// <summary>
        /// The name of the unit of measure
        /// </summary>
        /// <remarks>The common name of the unit of measure. Typically, this field represents the abbreviation for the unit.</remarks>
        public String Name;
        /// <summary>
        /// A description of the unit of measure
        /// </summary>
        /// <remarks>The description of the unit of measure.</remarks>
        public String Description;
        /// <summary>
        /// The category  of the unit of measure
        /// </summary>
        /// <remarks><para>The category for a unit of measure defines the dimensionality of the unit.</para>
        /// <para>The dimensionality of the parameter represents the physical dimensional axes of this parameter. It 
        /// is expected that the dimensionality must cover at least 6 fundamental axes (length, mass, time, angle, 
        /// temperature and charge). A possible implementation could consist in being a constant length array vector 
        /// that contains the exponents of each basic SI unit, following directives of SI-brochure (from 
        /// http://www.bipm.fr/). So if we agree on order &lt;m kg s A K,&gt; ... velocity would be &lt;1,0,-1,0,0,0&gt;: 
        /// that is m1 * s-1 =m/s. We have suggested to the  CO Scientific Committee to use the SI base units plus the 
        /// SI derived units with special symbols (for a better usability and for allowing the definition of angles).
        /// </para>
        /// </remarks>
        public String Category;
        /// <summary>
        /// A conversion factor used to multiply the value of the measurement by to convert the unit to its SI
        /// equivalent.
        /// </summary>
        /// <remarks>
        /// <para>Units are converted to and from the SI equivalent for the unit category. Unit conversions are 
        /// accomplished by first adding any offset, stored in <see cref = "ConversionPlus"/> to the value of the unit. 
        /// The sum is then multiplied by the value of the <see cref = "ConversionTimes"/> for the unit to get the 
        /// measured value in SI units.</para>
        /// </remarks>
        public double ConversionTimes;
        /// <summary>
        /// An offset factor used in converting the value of the measurement to its SI equivalent.
        /// </summary>
        /// <remarks>
        /// <para>Units are converted to and from the SI equivalent for the unit category. Unit conversions are 
        /// accomplished by first adding any offset, stored in <see cref = "ConversionPlus"/> to the value of the unit. 
        /// The sum is then multiplied by the value of the <see cref = "ConversionTimes"/> for the unit to get the 
        /// measured value in SI units.</para>
        /// </remarks>
        public double ConversionPlus;
    };

    struct unitCategory
    {
        /// <summary>
        /// Gets the name of the unit category, e.g. pressure, tempurature.
        /// </summary>
        /// <remarks>
        /// <para>The unit category repsresents the unique combination of dimensions (mass, length, 
        /// time, temperature, amount of substance (moles), electrical current, luminosity) associated with a particular
        /// unit of measure.
        /// </para>
        /// </remarks>
        public String Name;
        /// <summary>
        /// Gets the display unit for the parameter. Used by AspenPlus(TM).
        /// </summary>
        /// <remarks>
        /// <para>DisplayUnits defines the unit of measurement symbol for a parameter.</para>
        /// <para>Note: The symbol must be one of the uppercase strings recognized by Aspen
        /// Plus to ensure that it can perform unit of measurement conversions on the 
        /// parameter value. The system converts the parameter's value from SI units for
        /// display in the data browser and converts updated values back into SI.
        /// </para>
        /// </remarks>
        public String AspenUnit;
        /// <summary>
        /// Gets the name of the SI unit associated with the unit category, e.g. Pascals for pressure.
        /// </summary>
        /// <remarks>
        /// <para>The SI unit is the basis for conversions between any two units of the same category, either SI or 
        /// customary.
        /// </para>
        /// </remarks>
        public String SI_Unit;
        /// <summary>
        /// Gets the mass dimensionality associated with the unit category.
        /// </summary>
        /// <remarks>
        /// <para>The mass dimensionality of the unit category.
        /// </para>
        /// </remarks>
        public double Mass;
        /// <summary>
        /// Gets the time dimensionality associated with the unit category.
        /// </summary>
        /// <remarks>
        /// <para>The time dimensionality of the unit category.
        /// </para>
        /// </remarks>
        public double Time;
        /// <summary>
        /// Gets the length dimensionality associated with the unit category.
        /// </summary>
        /// <remarks>
        /// <para>The length dimensionality of the unit category.
        /// </para>
        /// </remarks>
        public double Length;
        /// <summary>
        /// Gets the electrical current (amperage) dimensionality associated with the unit category.
        /// </summary>
        /// <remarks>
        /// <para>The electrical current (amperage) dimensionality of the unit category.
        /// </para>
        /// </remarks>
        public double ElectricalCurrent;
        /// <summary>
        /// Gets the temperature dimensionality associated with the unit category.
        /// </summary>
        /// <remarks>
        /// <para>The temperature dimensionality of the unit category.
        /// </para>
        /// </remarks>
        public double Temperature;
        /// <summary>
        /// Gets the amount of substance (moles) dimensionality associated with the unit category.
        /// </summary>
        /// <remarks>
        /// <para>The amount of substance (moles) dimensionality of the unit category.
        /// </para>
        /// </remarks>
        public double AmountOfSubstance;
        /// <summary>
        /// Gets the luminousity dimensionality associated with the unit category.
        /// </summary>
        /// <remarks>
        /// <para>The luminousity dimensionality of the unit category.
        /// </para>
        /// </remarks>
        public double Luminous;
        /// <summary>
        /// Gets the financial currency dimensionality associated with the unit category.
        /// </summary>
        /// <remarks>
        /// <para>The financial currency dimensionality of the unit category.
        /// </para>
        /// </remarks>
        public double Currency;
    };

    /// <summary>
    /// Static class representing support for CAPE-OPEN dimensionalty and units of measures for real-valued
    /// parameters.
    /// </summary>
    /// <remarks>
    /// This class supports the use of CAPE-OPEN dimensionalities and conversion between SI and customary units of
    /// measure for real-valued parameters.
    /// </remarks>
    static class CDimensions
    {
        static System.Collections.ArrayList units;
        static System.Collections.ArrayList unitCategories;

        /// <summary>
        /// Initializes the static fields of the <see cref = "CDimensions"/> class
        /// </summary>
        /// <remarks>Loads units and unit category data from XML files.</remarks>
        static CDimensions()
        {
            units = new System.Collections.ArrayList();
            unitCategories = new System.Collections.ArrayList();
            System.AppDomain domain = System.AppDomain.CurrentDomain;
            //System.Reflection.Assembly myAssembly = System.Reflection.Assembly.GetExecutingAssembly();
            //System.IO.Stream resStream = myAssembly.GetManifestResourceStream("CapeOpen.Resources.units.xml.resources");
            //System.Resources.ResourceReader resReader = new System.Resources.ResourceReader(resStream);
            //System.Collections.IDictionaryEnumerator en = resReader.GetEnumerator();
            //String temp = String.Empty;
            //while (en.MoveNext())
            //{
            //    if (en.Key.ToString() == "units") temp = en.Value.ToString();
            //}
            System.Xml.XmlDocument reader = new System.Xml.XmlDocument();
            reader.LoadXml(Properties.Resources.units);
            System.Xml.XmlNodeList list = reader.SelectNodes("Units/Unit_Specs");
            System.Globalization.CultureInfo ci = new System.Globalization.CultureInfo(0x0409, false);
            for (int i = 0; i < list.Count; i++)
            {
                unit newUnit;
                String UnitName = list[i].SelectSingleNode("Unit").InnerText;
                newUnit.Name = UnitName.Trim();
                newUnit.Description = "";
                newUnit.Category = list[i].SelectSingleNode("Category").InnerText;
                newUnit.ConversionTimes = Convert.ToDouble(list[i].SelectSingleNode("ConversionTimes").InnerText, ci.NumberFormat);
                newUnit.ConversionPlus = Convert.ToDouble(list[i].SelectSingleNode("ConversionPlus").InnerText, ci.NumberFormat);
                units.Add(newUnit);
            }
            String userUnitPath = String.Concat(domain.BaseDirectory, "//data//user_defined_UnitsResult.XML");
            if (System.IO.File.Exists(userUnitPath))
            {
                reader.Load(userUnitPath);
                list = reader.SelectNodes("Units/Unit_Specs");
                for (int i = 0; i < list.Count; i++)
                {
                    unit newUnit = new unit();
                    String UnitName = list[i].SelectSingleNode("Unit").InnerText;
                    newUnit.Name = UnitName.Trim();
                    newUnit.Category = list[i].SelectSingleNode("Category").InnerText;
                    newUnit.ConversionTimes = Convert.ToDouble(list[i].SelectSingleNode("ConversionTimes").InnerText, ci.NumberFormat);
                    newUnit.ConversionPlus = Convert.ToDouble(list[i].SelectSingleNode("ConversionPlus").InnerText, ci.NumberFormat);
                    units.Add(newUnit);
                }
            }
            //resStream = myAssembly.GetManifestResourceStream("CapeOpen.Resources.unitCategories.xml.resources");
            //resReader = new System.Resources.ResourceReader(resStream);
            //en = resReader.GetEnumerator();
            //while (en.MoveNext())
            //{
            //    if (en.Key.ToString() == "unitCategories") temp = en.Value.ToString();
            //}
            reader.LoadXml(Properties.Resources.unitCategories);
            list = reader.SelectNodes("CategorySpecifications/Category_Spec");
            for (int i = 0; i < list.Count; i++)
            {
                String UnitName = list[i].SelectSingleNode("Category").InnerText;
                unitCategory category;
                category.Name = UnitName;
                category.AspenUnit = list[i].SelectSingleNode("Aspen").InnerText;
                category.SI_Unit = list[i].SelectSingleNode("SI_Unit").InnerText;
                category.Mass = Convert.ToDouble(list[i].SelectSingleNode("Mass").InnerText);
                category.Time = Convert.ToDouble(list[i].SelectSingleNode("Time").InnerText);
                category.Length = Convert.ToDouble(list[i].SelectSingleNode("Length").InnerText);
                category.ElectricalCurrent = Convert.ToDouble(list[i].SelectSingleNode("ElectricalCurrent").InnerText);
                category.Temperature = Convert.ToDouble(list[i].SelectSingleNode("Temperature").InnerText);
                category.AmountOfSubstance = Convert.ToDouble(list[i].SelectSingleNode("AmountOfSubstance").InnerText);
                category.Luminous = Convert.ToDouble(list[i].SelectSingleNode("Luminous").InnerText, ci.NumberFormat);
                category.Currency = Convert.ToDouble(list[i].SelectSingleNode("Currency").InnerText, ci.NumberFormat);
                unitCategories.Add(category);
            }
            String userUnitCategoryPath = String.Concat(domain.BaseDirectory, "data//user_defined_units.XML");
            if (System.IO.File.Exists(userUnitCategoryPath))
            {
                reader.Load(userUnitCategoryPath);
                list = reader.SelectNodes("CategorySpecifications/Category_Spec");
                for (int i = 0; i < list.Count; i++)
                {
                    String UnitName = list[i].SelectSingleNode("Category").InnerText;
                    unitCategory category;
                    category.Name = UnitName;
                    category.AspenUnit = list[i].SelectSingleNode("Aspen").InnerText;
                    category.SI_Unit = list[i].SelectSingleNode("SI_Unit").InnerText;
                    category.Mass = Convert.ToDouble(list[i].SelectSingleNode("Mass").InnerText, ci.NumberFormat);
                    category.Time = Convert.ToDouble(list[i].SelectSingleNode("Time").InnerText, ci.NumberFormat);
                    category.Length = Convert.ToDouble(list[i].SelectSingleNode("Length").InnerText, ci.NumberFormat);
                    category.ElectricalCurrent = Convert.ToDouble(list[i].SelectSingleNode("ElectricalCurrent").InnerText, ci.NumberFormat);
                    category.Temperature = Convert.ToDouble(list[i].SelectSingleNode("Temperature").InnerText, ci.NumberFormat);
                    category.AmountOfSubstance = Convert.ToDouble(list[i].SelectSingleNode("AmountOfSubstance").InnerText, ci.NumberFormat);
                    category.Luminous = Convert.ToDouble(list[i].SelectSingleNode("Luminous").InnerText, ci.NumberFormat);
                    category.Currency = Convert.ToDouble(list[i].SelectSingleNode("Currency").InnerText, ci.NumberFormat);
                    unitCategories.Add(category);
                }
            }
        }


        /// <summary>
        /// The SI unit asscoiated with a dimensionality.
        /// </summary>
        /// <remarks>
        /// <para>The SI unit of measure associated with a dimensionality.</para>
        /// <para>The dimensionality of the parameter represents the physical dimensional axes of this parameter. It 
        /// is expected that the dimensionality must cover at least 6 fundamental axes (length, mass, time, angle, 
        /// temperature and charge). A possible implementation could consist in being a constant length array vector 
        /// that contains the exponents of each basic SI unit, following directives of SI-brochure (from 
        /// http://www.bipm.fr/). So if we agree on order &lt;m kg s A K,&gt; ... velocity would be &lt;1,0,-1,0,0,0&gt;: 
        /// that is m1 * s-1 =m/s. We have suggested to the  CO Scientific Committee to use the SI base units plus the 
        /// SI derived units with special symbols (for a better usability and for allowing the definition of angles).
        /// </para>
        /// </remarks>
        /// <param name="dimensions">The dimensionality of the unit.</param>
        /// <returns>The SI unit having the desired dimensionality</returns>
        public static String SIUnit(int[] dimensions)
        {
            foreach (unitCategory category in unitCategories)
            {
                if (dimensions[0] == category.Length &&
                    dimensions[1] == category.Mass &&
                    dimensions[2] == category.Time &&
                    dimensions[3] == category.ElectricalCurrent &&
                    dimensions[4] == category.Temperature &&
                    dimensions[5] == category.AmountOfSubstance &&
                    dimensions[6] == category.Luminous)
                    return category.SI_Unit;
            }
            return string.Empty;
        }

        /// <summary>
        /// The SI unit asscoiated with a dimensionality.
        /// </summary>
        /// <remarks>
        /// <para>The SI unit of measure associated with a dimensionality.</para>
        /// <para>The dimensionality of the parameter represents the physical dimensional axes of this parameter. It 
        /// is expected that the dimensionality must cover at least 6 fundamental axes (length, mass, time, angle, 
        /// temperature and charge). A possible implementation could consist in being a constant length array vector 
        /// that contains the exponents of each basic SI unit, following directives of SI-brochure (from 
        /// http://www.bipm.fr/). So if we agree on order &lt;m kg s A K,&gt; ... velocity would be &lt;1,0,-1,0,0,0&gt;: 
        /// that is m1 * s-1 =m/s. We have suggested to the  CO Scientific Committee to use the SI base units plus the 
        /// SI derived units with special symbols (for a better usability and for allowing the definition of angles).
        /// </para>
        /// </remarks>
        /// <param name="dimensions">The dimensionality of the unit.</param>
        /// <returns>The SI unit having the desired dimensionality</returns>
        public static String SIUnit(double[] dimensions)
        {
            foreach (unitCategory category in unitCategories)
            {
                if (dimensions[0] == category.Length &&
                    dimensions[1] == category.Mass &&
                    dimensions[2] == category.Time &&
                    dimensions[3] == category.ElectricalCurrent &&
                    dimensions[4] == category.Temperature &&
                    dimensions[5] == category.AmountOfSubstance &&
                    dimensions[6] == category.Luminous)
                    return category.SI_Unit;
            }
            return string.Empty;
        }

        /// <summary>
        /// Provides all units supported by the dimensionality package.
        /// </summary>
        /// <remarks>Provides a list of all the units of measure available in this unit conversion package.</remarks>
        /// <value>The list of all units.</value>
        public static String[] Units
        {
            get
            {
                String[] retVal = new String[units.Count];
                for (int i = 0; i < units.Count; i++)
                {
                    retVal[i] = ((unit)units[i]).Name;
                }
                return retVal;
            }
        }

        /// <summary>
        /// A conversion factor used to multiply the value of the measurement by to convert the unit to its SI
        /// equivalent.
        /// </summary>
        /// <remarks>
        /// <para>Units are converted to and from the SI equivalent for the unit category. Unit conversions are 
        /// accomplished by first adding any offset, stored in <see cref = "ConverionsPlus"/> to the value of the unit. 
        /// The sum is then multiplied by the value of the <see cref = "ConverionsTimes"/> for the unit to get the 
        /// measured value in SI units.</para>
        /// </remarks>
        /// <param name="unit">The unit to get the conversion factor for.</param>
        /// <returns>The multiplicative part of the conversion factor.</returns>
        public static double ConverionsTimes(String unit)
        {
            double retVal = 0;
            bool found = false;
            for (int i = 0; i < units.Count; i++)
            {
                CapeOpen.unit current = (CapeOpen.unit)units[i];
                if (current.Name == unit)
                {
                    retVal = current.ConversionTimes;
                    found = true;
                }
            }
            if (!found) throw new CapeOpen.CapeBadArgumentException(String.Concat("Unit: ", unit, " was not found"), 1);
            return retVal;
        }

        /// <summary>
        /// An offset factor used in converting the value of the measurement to its SI equivalent.
        /// </summary>
        /// <remarks>
        /// <para>Units are converted to and from the SI equivalent for the unit category. Unit conversions are 
        /// accomplished by first adding any offset, stored in <see cref = "ConverionsPlus"/> to the value of the unit. 
        /// The sum is then multiplied by the value of the <see cref = "ConverionsTimes"/> for the unit to get the 
        /// measured value in SI units.</para>
        /// </remarks>
        /// <param name="unit">The unit to get the conversion factor for.</param>
        /// <returns>The additive part of the conversion factor.</returns>
        public static double ConverionsPlus(String unit)
        {
            double retVal = 0;
            bool found = false;
            for (int i = 0; i < units.Count; i++)
            {
                CapeOpen.unit current = (CapeOpen.unit)units[i];
                if (current.Name == unit)
                {
                    retVal = current.ConversionPlus;
                    found = true;
                }
            }
            if (!found) throw new CapeOpen.CapeBadArgumentException(String.Concat("Unit: ", unit, " was not found"), 1);
            return retVal;
        }

        /// <summary>
        /// The category  of the unit of measure
        /// </summary>
        /// <remarks><para>The category for a unit of measure defines the dimensionality of the unit.</para>
        /// </remarks>
        /// <param name="unit">The unit to get the category of.</param>
        /// <returns>The unit category.</returns>
        public static String UnitCategory(String unit)
        {
            String retVal = String.Empty;
            bool found = false;
            for (int i = 0; i < units.Count; i++)
            {
                CapeOpen.unit current = (CapeOpen.unit)units[i];
                if (current.Name == unit)
                {
                    retVal = current.Category;
                    found = true;
                }
            }
            //for (int i = 0; i < unitCategories.Count; i++)
            //{
            //    CapeOpen.unitCategory current = (CapeOpen.unitCategory)unitCategories[i];
            //    if (current.Name == unit)
            //    {
            //        retVal = current.AspenUnit;
            //        found = true;
            //    }
            //}
            if (!found) throw new CapeOpen.CapeBadArgumentException(String.Concat("Unit: ", unit, " was not found"), 1);
            return retVal;
        }

        /// <summary>
        /// Gets the name of the SI unit associated with the unit category, e.g. Pascals for pressure.
        /// </summary>
        /// <remarks>
        /// <para>The SI unit is the basis for conversions between any two units of the same category, either SI or 
        /// customary.
        /// </para>
        /// </remarks>
        /// <returns>The Aspen(TM) display unit that corresponds to the current unit.</returns>
        /// <param name="unit">The unit to get the Aspen (TM) unit for.</param>
        public static String AspenUnit(String unit)
        {
            String retVal = String.Empty;
            String category = String.Empty;
            bool found = false;
            for (int i = 0; i < units.Count; i++)
            {
                CapeOpen.unit current = (CapeOpen.unit)units[i];
                if (current.Name == unit)
                {
                    category = current.Category;
                    found = true;
                }
            }
            for (int i = 0; i < unitCategories.Count; i++)
            {
                CapeOpen.unitCategory current = (CapeOpen.unitCategory)unitCategories[i];
                if (current.Name == category)
                {
                    retVal = current.AspenUnit;
                    found = true;
                }
            }
            if (!found) throw new CapeOpen.CapeBadArgumentException(String.Concat("Unit: ", unit, " was not found"), 1);
            return retVal;
        }

        /// <summary>
        /// A description of the unit of measure
        /// </summary>
        /// <remarks>The description of the unit of measure.</remarks>
        /// <param name="unit">The unit to get the conversion factor for.</param>
        /// <returns>The description of the unit of measure.</returns>
        public static String UnitDescription(String unit)
        {
            String retVal = String.Empty;
            bool found = false;
            for (int i = 0; i < units.Count; i++)
            {
                CapeOpen.unit current = (CapeOpen.unit)units[i];
                if (current.Name == unit)
                {
                    retVal = current.Description;
                    found = true;
                }
            }
            if (!found) throw new CapeOpen.CapeBadArgumentException(String.Concat("Unit: ", unit, " was not found"), 1);
            return retVal;
        }

        ///// <summary>
        ///// Changes the description of the unit of measure
        ///// </summary>
        ///// <remarks>Changes the description of the unit of measure.</remarks>
        //public static void ChangeUnitDescription(String unit, String newDescription)
        //{
        //    bool found = false;
        //    for (int i = 0; i < units.Count; i++)
        //    {
        //        CapeOpen.unit current = (CapeOpen.unit)units[i];
        //        if (current.Name == unit)
        //        {
        //            current.Description = newDescription;
        //            found = true;
        //        }
        //    }
        //    if (!found) throw new CapeOpen.CapeBadArgumentException(String.Concat("Unit: ", unit, " was not found"), 1);
        //}

        /// <summary>
        /// Returns all units matching the unit category.
        /// </summary>
        /// <remarks>A unit category represents a specific combination of dimsionality values. Examples would be 
        /// pressure or temperature. This method would return all units that are in the category, such as Celius,
        /// Kelvin, Farehnheit, and Rankine for temperature.</remarks>
        /// <param name="category">The catgeory of the desired units.</param>
        /// <returns>All units that represent the categoery.</returns>
        public static String[] UnitsMatchingCategory(String category)
        {
            System.Collections.ArrayList unitNames = new System.Collections.ArrayList();
            for (int i = 0; i < units.Count; i++)
            {
                CapeOpen.unit current = (CapeOpen.unit)units[i];
                if (current.Category == category)
                {
                    unitNames.Add(current.Name);
                }
            }
            String[] retVal = new String[unitNames.Count];
            for (int i = 0; i < unitNames.Count; i++)
            {
                retVal[i] = unitNames[i].ToString();
            }
            return retVal;
        }

        /// <summary>
        /// Returns the SI unit associated with the unit.
        /// </summary>
        /// <remarks>A unit category represents a specific combination of dimsionality values. Examples would be 
        /// pressure or temperature. This method would return the SI unit for the category, such as Kelvin (K) for 
        /// temperature or Pascal (N/m^2) for pressure..</remarks>
        /// <param name="Unit">The unit to get the SI unit of.</param>
        /// <returns>The SI unit that corresponds to the unit.</returns>
        public static String FindSIUnit(String Unit)
        {
            String retVal = String.Empty;
            String category = UnitCategory(Unit);
            for (int i = 0; i < unitCategories.Count; i++)
            {
                CapeOpen.unitCategory current = (CapeOpen.unitCategory)unitCategories[i];
                if (current.Name == category)
                {
                    retVal = current.SI_Unit;
                }
            }
            return retVal;
        }

        /// <summary>
        /// The dimensioality of the unit of measure.
        /// </summary>
        /// <remarks>
        /// <para>The dimensionality of the parameter represents the physical dimensional axes of this parameter. It 
        /// is expected that the dimensionality must cover at least 6 fundamental axes (length, mass, time, angle, 
        /// temperature and charge). A possible implementation could consist in being a constant length array vector 
        /// that contains the exponents of each basic SI unit, following directives of SI-brochure (from 
        /// http://www.bipm.fr/). So if we agree on order &lt;m kg s A K,&gt; ... velocity would be &lt;1,0,-1,0,0,0&gt;: 
        /// that is m1 * s-1 =m/s. We have suggested to the  CO Scientific Committee to use the SI base units plus the 
        /// SI derived units with special symbols (for a better usability and for allowing the definition of angles).
        /// </para>
        /// </remarks>
        /// <param name="unit">The unit to get the dimensionality of.</param>
        /// <returns>The dimenality of the unit.</returns>
        public static double[] Dimensionality(String unit)
        {
            string category = CapeOpen.CDimensions.UnitCategory(unit);
            double[] retVal = { 0, 0, 0, 0, 0, 0, 0, 0 };
            for (int i = 0; i < unitCategories.Count; i++)
            {
                CapeOpen.unitCategory current = (CapeOpen.unitCategory)unitCategories[i];
                if (current.Name == category)
                {
                    retVal[0] = current.Length;
                    retVal[1] = current.Mass;
                    retVal[2] = current.Time;
                    retVal[3] = current.ElectricalCurrent;
                    retVal[4] = current.Temperature;
                    retVal[5] = current.AmountOfSubstance;
                    retVal[6] = current.Luminous;
                    retVal[7] = current.Currency;
                }
            }
            return retVal;
        }
    };
}
