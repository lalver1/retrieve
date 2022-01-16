use clap::ArgEnum;
use color_eyre::{eyre::Report, Result};
use csv::Reader;
use serde::Deserialize;
use std::fmt;
use std::path::Path;
use url::Url;

pub mod cli;

/// Represent the PFB S3 storage base URL.
const PFB_S3_STORAGE_BASE_URL: &str =
    "https://s3.amazonaws.com/production-pfb-storage-us-east-1/results";

/// Represent the name of the "neighborhood ways" dataset.
const DS_NEIGHBORHOOD_WAYS: &str = "neighborhood_ways";

/// Setup the application.
///
/// Set up the `color_eyre` hooks.
pub fn setup() -> Result<(), Report> {
    color_eyre::install()?;

    Ok(())
}

/// Describe all the available city datasets.
#[derive(Debug, PartialEq, PartialOrd, ArgEnum, Clone, Copy)]
pub enum Dataset {
    NeighborhoodWays,
}

impl From<&str> for Dataset {
    fn from(item: &str) -> Self {
        match item {
            DS_NEIGHBORHOOD_WAYS => Dataset::NeighborhoodWays,
            _ => panic!("Cannot parse dataset name {}", item),
        }
    }
}

impl fmt::Display for Dataset {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Dataset::NeighborhoodWays => write!(f, "{}", DS_NEIGHBORHOOD_WAYS),
        }
    }
}

/// Define a PeopleForBikes city.
#[derive(Debug, Deserialize, Clone)]
pub struct City {
    /// City name.
    #[serde(rename = "City")]
    pub name: String,
    /// Country where the city is located.
    #[serde(rename = "Country")]
    pub country: String,
    /// State where the city is located.
    #[serde(rename = "State")]
    pub state: String,
    /// City's unique identifier.
    ///
    /// It is generated by a specific Bicyle Network Analysis (BNA) run and
    /// should be assimilated to a version number (each run will generate a
    /// new identifier).
    pub uuid: String,
}
impl City {
    /// Create a new City.
    ///
    /// If the `state` is not specified (a lot of countries do not have states),
    /// the name of the country is used instead.
    pub fn new(name: &str, country: &str, state: Option<&str>, uuid: &str) -> Self {
        City {
            name: name.into(),
            country: country.into(),
            state: if let Some(s) = state {
                s.into()
            } else {
                country.into()
            },
            uuid: uuid.into(),
        }
    }

    /// Return the full name of the city.
    ///
    /// The full name has the following format: `{COUNTRY}-{STATE}-{CITY_NAME}`.
    pub fn full_name(&self) -> String {
        format!("{}-{}-{}", self.country, self.state, self.name)
    }

    /// Return the full name of the city with the `.zip` extension.
    pub fn zip_name(&self) -> String {
        format!("{}.zip", self.full_name())
    }

    /// Return the URL of the neighborhood_ways data set.
    pub fn neighborhood_ways_url(&self) -> Result<Url, Report> {
        let dataset_url = format!(
            "{}/{}/{}.zip",
            PFB_S3_STORAGE_BASE_URL,
            self.uuid,
            Dataset::NeighborhoodWays
        );
        let url = Url::parse(&dataset_url)?;
        Ok(url)
    }

    /// Return the URL of the specified `dataset`.
    pub fn url(&self, dataset: Dataset) -> Result<Url, Report> {
        match dataset {
            Dataset::NeighborhoodWays => self.neighborhood_ways_url(),
        }
    }

    /// Read a CSV file and populate a Vector of Cities.
    ///
    /// The CSV file is expected to contain the following fields (case sensitive):
    /// * City
    /// * Country
    /// * State
    /// * uuid
    pub fn from_csv<P>(path: P) -> Result<Vec<City>, Report>
    where
        P: AsRef<Path>,
    {
        let mut csv_reader = Reader::from_path(path)?;
        let mut cities: Vec<City> = vec![];
        for record in csv_reader.deserialize() {
            cities.push(record?);
        }

        Ok(cities)
    }
}
