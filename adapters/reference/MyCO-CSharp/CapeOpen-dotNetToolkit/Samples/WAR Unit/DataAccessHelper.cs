using System;
using System.Collections.Generic;
using System.Text;

namespace CapeOpenWar
{
    public class DataAccessHelper
    {
        private DatiUSERDB.ComponentsDataTable mComponentsTable;
        private DatiUSERDB.FactorDataTable mFactorsTable;

        public DataAccessHelper()
        {
            DatiUSERDB.ComponentsDataTable zComponentsTable = new DatiUSERDB.ComponentsDataTable();
            zComponentsTable.ReadXml(Settings.Default.WARDataPath);
            mComponentsTable = zComponentsTable;

            DatiUSERDB.FactorDataTable zFactorsTable = new DatiUSERDB.FactorDataTable();
            zFactorsTable.ReadXml(Settings.Default.FactorsData);
            mFactorsTable = zFactorsTable;
        }


        public double GetSumIndexNorm(string casNumber)
        {
            DatiUSERDB.ComponentsRow zRiga = null;

            foreach (DatiUSERDB.ComponentsRow zComponentRow in mComponentsTable.Rows)
            {
                if (zComponentRow.CAS.Contains(casNumber))
                {
                    zRiga = zComponentRow;
                }
            }

            if (zRiga == null)
            {
                throw new NullReferenceException();
            }

            double LD50Score;
            double OSHA_TWAScore;
            double FHM_LC50Score;
            double GWPScore;
            double ODScore;
            double PCOScore;
            double APScore;

            if (zRiga.IsLD50RatScoreNull())
            {
                LD50Score = 0;
            }
            else
            {
                LD50Score = zRiga.LD50RatScore;
            }

            if (zRiga.IsOSHA_TWA_ScoreNull())
            {
                OSHA_TWAScore = 0;
            }
            else
            {
                OSHA_TWAScore = zRiga.OSHA_TWA_Score;
            }

            if (zRiga.IsFHM_LC50_ScoreNull())
            {
                FHM_LC50Score = 0;
            }
            else
            {
                FHM_LC50Score = zRiga.FHM_LC50_Score;
            }

            if (zRiga.IsGWP_ScoreNull())
            {
                GWPScore = 0;
            }
            else
            {
                GWPScore = zRiga.GWP_Score;
            }

            if (zRiga.IsOD_ScoreNull())
            {
                ODScore = 0;
            }
            else
            {
                ODScore = zRiga.OD_Score;
            }

            if (zRiga.IsPCO_ScoreNull())
            {
                PCOScore = 0;
            }
            else
            {
                PCOScore = zRiga.PCO_Score;
            }

            if (zRiga.IsAP_ScoreNull())
            {
                APScore = 0;
            }
            else
            {
                APScore = zRiga.AP_Score;
            }

            return (LD50Score * mFactorsTable.FindByName("HTPI_factor").Value +
                    OSHA_TWAScore * mFactorsTable.FindByName("HTPE_factor").Value +
                    FHM_LC50Score * mFactorsTable.FindByName("ATP_factor").Value +
                    GWPScore * mFactorsTable.FindByName("GWP_factor").Value +
                    ODScore * mFactorsTable.FindByName("ODP_factor").Value +
                    PCOScore * mFactorsTable.FindByName("PCOP_factor").Value +
                    APScore * mFactorsTable.FindByName("AP_factor").Value);
        }

    }
}
