use std::str::FromStr;

use osmpbfreader::Way;

const TARGET_TAG_KEY: &str = "highway";

#[derive(PartialEq, Eq, Debug, Clone, Copy, Hash)]
pub enum ApprovedHighwayType {
    Secondary,
    LivingStreet,
    Residential,
    Tertiary,
    Trunk,
    Motorway,
    Primary,
    MotorwayLink,
    TrunkLink,
    PrimaryLink,
    SecondaryLink,
    TertiaryLink,
    NA
}

impl FromStr for ApprovedHighwayType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "secondary" => Ok(ApprovedHighwayType::Secondary),
            "living_street" => Ok(ApprovedHighwayType::LivingStreet),
            "residential" => Ok(ApprovedHighwayType::Residential),
            "tertiary" => Ok(ApprovedHighwayType::Tertiary),
            "trunk" => Ok(ApprovedHighwayType::Trunk),
            "motorway" => Ok(ApprovedHighwayType::Motorway),
            "primary" => Ok(ApprovedHighwayType::Primary),
            "motorway_link" => Ok(ApprovedHighwayType::MotorwayLink),
            "trunk_link" => Ok(ApprovedHighwayType::TrunkLink),
            "primary_link" => Ok(ApprovedHighwayType::PrimaryLink),
            "secondary_link" => Ok(ApprovedHighwayType::SecondaryLink),
            "tertiary_link" => Ok(ApprovedHighwayType::TertiaryLink),
            _ => Err(()),
        }
    }
}

/// Filter data to contain only needed values and create graph out of it
/// Accept only ways that contain tag "highway" with values:
/// secondary
/// living_street
/// residential
/// tertiary
/// trunk
/// motorway
/// primary
/// motorway_link
/// trunk_link
/// primary_link
/// secondary_link
/// tertiary_link
pub(crate) fn filter_way_data(osm_way: &osmpbfreader::Way) -> Option<(&Way, ApprovedHighwayType)> {
    osm_way.tags.get(TARGET_TAG_KEY)
        .and_then(|tag_value| {
            match ApprovedHighwayType::from_str(tag_value) {
                Ok(highway_type) => Some((osm_way, highway_type)),
                Err(_) => None
            }
        })
}


#[cfg(test)]
mod tests {
    use osmpbfreader::{Tags, Way as OsmWay, WayId};
    use smartstring::alias::String;

    use super::*;

    #[test]
    fn test_none_when_no_highway_tag() {
        let osm_way = OsmWay {
            id: WayId(1),
            nodes: vec![],
            tags: Default::default(),
        };

        assert_eq!(filter_way_data(&osm_way), None)
    }

    #[test]
    fn test_some_when_highway_tag_as_primary() {
        let osm_node = OsmWay {
            id: WayId(1),
            tags: Tags::from_iter(vec![(String::from(TARGET_TAG_KEY), String::from("primary"))]),
            nodes: vec![],
        };

        assert_eq!(filter_way_data(&osm_node), Some((&osm_node, ApprovedHighwayType::Primary)))
    }

    #[test]
    fn test_some_when_highway_tag_as_primary_and_more() {
        let osm_node = OsmWay {
            id: WayId(1),
            tags: Tags::from_iter(vec![(String::from(TARGET_TAG_KEY), String::from("primary")), (String::from(TARGET_TAG_KEY), String::from("trunk"))]),
            nodes: vec![],
        };

        assert_eq!(filter_way_data(&osm_node), Some((&osm_node, ApprovedHighwayType::Primary)))
    }

    #[test]
    fn test_none_when_no_accepted_highway_tag() {
        let osm_node = OsmWay {
            id: WayId(1),
            tags: Tags::from_iter(vec![(String::from(TARGET_TAG_KEY), String::from("footway"))]),
            nodes: vec![],
        };

        assert_eq!(filter_way_data(&osm_node), None);
    }
}
