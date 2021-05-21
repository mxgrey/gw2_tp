use std::collections::HashMap;

mod descriptions;

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
    for (material, description) in descriptions_yaml.into_iter() {
        descriptions.insert(
            material.as_str().unwrap().to_string(),
            MaterialDescription::new(&description)
        );
    }

    println!("Parsed result:\n{:?}", descriptions);
}
