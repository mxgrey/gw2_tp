#[macro_use]
extern crate serde_derive;

use std::collections::HashMap;

mod descriptions;
mod tp_rest;
mod planning;

use descriptions::MaterialDescription;

extern crate argparse;
use argparse::{ArgumentParser, Store};


fn main() {

    let mut material_descriptions_file_name =
        ".material-descriptions.yaml".to_string();

    let mut target_materials_file_name = "material-targets.yaml".to_string();

    {
        let mut parser = ArgumentParser::new();
        parser.set_description(
            "Calculate the best raw materials to buy from the Guild Wars 2 \
            Trading Post for crafting a list of items."
        );

        parser
            .refer(&mut material_descriptions_file_name)
            .add_option(
                &["-d", "--descriptions"],
                Store,
                "Material Descriptions Database"
            );

        parser
            .refer(&mut target_materials_file_name)
            .add_option(
                &["-t", "--targets"],
                Store,
                "Target materials list"
            );

        parser.parse_args_or_exit();
    }

    let material_descriptions_file =
        std::fs::File::open(
            std::path::Path::new(&material_descriptions_file_name)
        ).unwrap();

    let descriptions_yaml: serde_yaml::Mapping =
        serde_yaml::from_reader(material_descriptions_file).unwrap();

    let mut descriptions = HashMap::<String, MaterialDescription>::new();
    for (material, description) in &descriptions_yaml {
        descriptions.insert(
            material.as_str().unwrap().to_string(),
            MaterialDescription::new(&description)
        );
    }

    let target_materials: Vec<String> =
        serde_yaml::from_reader(
            std::fs::File::open(
                std::path::Path::new(&target_materials_file_name)
            ).unwrap()
        ).unwrap();

    println!("Target materials: {:?}", target_materials);

    println!("Parsed result:\n{:?}", descriptions);

    let listings = tp_rest::get_listings_for_targets(
        target_materials.clone(),
        &descriptions
    );

    println!("\nPrices:");
    for (name, item) in &listings {
        println!(
            "{} lowest price: {}",
            name,
            item.sells().get(0).unwrap().unit_price()
        );
    }

    planning::plan(&target_materials, descriptions, &listings);
}
