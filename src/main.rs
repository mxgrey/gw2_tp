use std::collections::HashMap;
use std::collections::BTreeMap;

// extern crate yaml_rust;
// use yaml_rust::{YamlLoader, YamlEmitter};

extern crate argparse;
use argparse::{ArgumentParser, Store};

#[derive(Debug)]
struct MaterialDescription {
    post_id: Option<u32>,
    vendor: Option<u32>,
    recipes: Vec<HashMap<String, u32>>
}

fn get_u32_if_available(
    map: &BTreeMap<String, serde_yaml::Value>,
    name: &str
) -> Option<u32> {

    if let Some(value) = map.get(name) {
        match value {
            serde_yaml::Value::Number(v) => {
                return Some(v.as_u64().unwrap() as u32);
            },
            serde_yaml::Value::Null => {
                return None;
             },
            _ => {
                panic!("Invalid data type for [{}] field: {:?}. \
                        Expected an unsigned integer.", name, value);
            }
        }
    } else {
        return None;
    }
}

fn convert_recipe(recipe: &serde_yaml::Value) -> HashMap<String, u32> {
    match recipe {
        serde_yaml::Value::Mapping(map) => {
            return serde_yaml::from_value(
                serde_yaml::Value::Mapping(map.clone())
            ).unwrap();
        },
        _ => {
            panic!("Invalid data type for a recipe entry: {:?}. \
                    Expected a map.", recipe);
        }
    }
}

fn get_recipes(map: &BTreeMap<String, serde_yaml::Value>)
-> Vec<HashMap<String, u32>> {

    let mut recipes: Vec<HashMap<String, u32>> = Vec::new();

    let recipes_key = "recipes";
    if let Some(value) = map.get(recipes_key) {
        match value {
            serde_yaml::Value::Sequence(seq) => {

                for recipe in seq.iter() {
                    recipes.push(convert_recipe(recipe));
                }

                return recipes;
            },
            serde_yaml::Value::Null => {
                return recipes;
            },
            _ => {
                panic!("Invalid data type for [{}] field: {:?}. \
                        Expected a sequence.", recipes_key, value);
            }
        }
    } else {
        return recipes;
    }
}

impl MaterialDescription {
    // fn new(info: &yaml_rust::yaml::Hash) -> MaterialDescription {
    fn new(info: serde_yaml::Value) -> MaterialDescription {

        let dict: BTreeMap<String, serde_yaml::Value> =
            serde_yaml::from_value(info).unwrap();

        return MaterialDescription{
            post_id: get_u32_if_available(&dict, "post_id"),
            vendor: get_u32_if_available(&dict, "vendor"),
            recipes: get_recipes(&dict)
        };
    }
}

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
            MaterialDescription::new(description)
        );
    }

    println!("Parsed result:\n{:?}", descriptions);
}
