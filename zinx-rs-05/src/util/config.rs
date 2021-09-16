use serde::Deserialize;
use structopt::StructOpt;
use structopt_toml::StructOptToml;

#[derive(Debug, Deserialize, StructOpt, StructOptToml)]
#[serde(default)]
pub struct Opt {
    #[structopt(default_value = "0", short)]
    pub port: u32,
    #[structopt(default_value = "kes", short)]
    pub name: String,
}
