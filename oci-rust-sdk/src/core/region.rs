use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Region {
    AfJohannesburg1,
    ApBatam1,
    ApChuncheon1,
    ApHyderabad1,
    ApMelbourne1,
    ApMumbai1,
    ApOsaka1,
    ApSeoul1,
    ApSingapore1,
    ApSingapore2,
    ApSydney1,
    ApTokyo1,
    CaMontreal1,
    CaToronto1,
    EuAmsterdam1,
    EuFrankfurt1,
    EuJovanovac1,
    EuMadrid1,
    EuMadrid3,
    EuMarseille1,
    EuMilan1,
    EuParis1,
    EuStockholm1,
    EuTurin1,
    EuZurich1,
    IlJerusalem1,
    MeAbudhabi1,
    MeDubai1,
    MeJeddah1,
    MeRiyadh1,
    MxMonterrey1,
    MxQueretaro1,
    SaBogota1,
    SaSantiago1,
    SaSaopaulo1,
    SaValparaiso1,
    SaVinhedo1,
    UkCardiff1,
    UkLondon1,
    UsAshburn1,
    UsChicago1,
    UsPhoenix1,
    UsSanjose1,
}

impl Region {
    pub fn endpoint(&self, service: &str) -> String {
        if service == "iaas" {
            format!("https://{service}.{self}.oraclecloud.com")
        } else {
            format!("https://{service}.{self}.oci.oraclecloud.com")
        }
    }
}

impl std::str::FromStr for Region {
    type Err = crate::core::OciError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            // Americas
            "us-phoenix-1" => Ok(Region::UsPhoenix1),
            "us-ashburn-1" => Ok(Region::UsAshburn1),
            "ca-toronto-1" => Ok(Region::CaToronto1),
            "ca-montreal-1" => Ok(Region::CaMontreal1),
            "sa-saopaulo-1" => Ok(Region::SaSaopaulo1),
            "sa-vinhedo-1" => Ok(Region::SaVinhedo1),
            "sa-santiago-1" => Ok(Region::SaSantiago1),

            // EMEA
            "uk-london-1" => Ok(Region::UkLondon1),
            "uk-cardiff-1" => Ok(Region::UkCardiff1),
            "eu-frankfurt-1" => Ok(Region::EuFrankfurt1),
            "eu-zurich-1" => Ok(Region::EuZurich1),
            "eu-amsterdam-1" => Ok(Region::EuAmsterdam1),
            "eu-madrid-1" => Ok(Region::EuMadrid1),
            "eu-milan-1" => Ok(Region::EuMilan1),
            "eu-paris-1" => Ok(Region::EuParis1),
            "eu-stockholm-1" => Ok(Region::EuStockholm1),
            "me-jeddah-1" => Ok(Region::MeJeddah1),
            "me-dubai-1" => Ok(Region::MeDubai1),
            "il-jerusalem-1" => Ok(Region::IlJerusalem1),
            "af-johannesburg-1" => Ok(Region::AfJohannesburg1),

            // Asia Pacific
            "ap-tokyo-1" => Ok(Region::ApTokyo1),
            "ap-osaka-1" => Ok(Region::ApOsaka1),
            "ap-seoul-1" => Ok(Region::ApSeoul1),
            "ap-chuncheon-1" => Ok(Region::ApChuncheon1),
            "ap-mumbai-1" => Ok(Region::ApMumbai1),
            "ap-hyderabad-1" => Ok(Region::ApHyderabad1),
            "ap-melbourne-1" => Ok(Region::ApMelbourne1),
            "ap-sydney-1" => Ok(Region::ApSydney1),
            "ap-singapore-1" => Ok(Region::ApSingapore1),

            _ => Err(crate::core::OciError::InvalidRegion(s.to_string())),
        }
    }
}

impl Region {
    fn id(&self) -> &'static str {
        match self {
            Region::AfJohannesburg1 => "af-johannesburg-1",
            Region::ApBatam1 => "ap-batam-1",
            Region::ApChuncheon1 => "ap-chuncheon-1",
            Region::ApHyderabad1 => "ap-hyderabad-1",
            Region::ApMelbourne1 => "ap-melbourne-1",
            Region::ApMumbai1 => "ap-mumbai-1",
            Region::ApOsaka1 => "ap-osaka-1",
            Region::ApSeoul1 => "ap-seoul-1",
            Region::ApSingapore1 => "ap-singapore-1",
            Region::ApSingapore2 => "ap-singapore-2",
            Region::ApSydney1 => "ap-sydney-1",
            Region::ApTokyo1 => "ap-tokyo-1",
            Region::CaMontreal1 => "ca-montreal-1",
            Region::CaToronto1 => "ca-toronto-1",
            Region::EuAmsterdam1 => "eu-amsterdam-1",
            Region::EuFrankfurt1 => "eu-frankfurt-1",
            Region::EuJovanovac1 => "eu-jovanovac-1",
            Region::EuMadrid1 => "eu-madrid-1",
            Region::EuMadrid3 => "eu-madrid-3",
            Region::EuMarseille1 => "eu-marseille-1",
            Region::EuMilan1 => "eu-milan-1",
            Region::EuParis1 => "eu-paris-1",
            Region::EuStockholm1 => "eu-stockholm-1",
            Region::EuTurin1 => "eu-turin-1",
            Region::EuZurich1 => "eu-zurich-1",
            Region::IlJerusalem1 => "il-jerusalem-1",
            Region::MeAbudhabi1 => "me-abudhabi-1",
            Region::MeDubai1 => "me-dubai-1",
            Region::MeJeddah1 => "me-jeddah-1",
            Region::MeRiyadh1 => "me-riyadh-1",
            Region::MxMonterrey1 => "mx-monterrey-1",
            Region::MxQueretaro1 => "mx-queretaro-1",
            Region::SaBogota1 => "sa-bogota-1",
            Region::SaSantiago1 => "sa-santiago-1",
            Region::SaSaopaulo1 => "sa-saopaulo-1",
            Region::SaValparaiso1 => "sa-valparaiso-1",
            Region::SaVinhedo1 => "sa-vinhedo-1",
            Region::UkCardiff1 => "uk-cardiff-1",
            Region::UkLondon1 => "uk-london-1",
            Region::UsAshburn1 => "us-ashburn-1",
            Region::UsChicago1 => "us-chicago-1",
            Region::UsPhoenix1 => "us-phoenix-1",
            Region::UsSanjose1 => "us-sanjose-1",
        }
    }
}

impl fmt::Display for Region {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id())
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_region_id() {
        assert_eq!(Region::ApSeoul1.id(), "ap-seoul-1");
        assert_eq!(Region::UsPhoenix1.id(), "us-phoenix-1");
    }

    #[test]
    fn test_region_endpoint() {
        assert_eq!(
            Region::ApSeoul1.endpoint("osmh"),
            "https://osmh.ap-seoul-1.oci.oraclecloud.com"
        );
    }

    #[test]
    fn test_region_from_str() {
        assert_eq!(Region::from_str("ap-seoul-1").unwrap(), Region::ApSeoul1);
        assert_eq!(
            Region::from_str("us-phoenix-1").unwrap(),
            Region::UsPhoenix1
        );
        assert!(Region::from_str("invalid-region").is_err());
    }

    #[test]
    fn test_region_display() {
        assert_eq!(format!("{}", Region::ApSeoul1), "ap-seoul-1");
    }
}
