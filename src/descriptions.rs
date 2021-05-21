use std::collections::HashMap;
use std::collections::BTreeMap;

#[derive(Debug)]
pub struct MaterialDescription {
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
    pub fn new(info: &serde_yaml::Value) -> MaterialDescription {

        let dict: BTreeMap<String, serde_yaml::Value> =
            serde_yaml::from_value(info.clone()).unwrap();

        return MaterialDescription{
            post_id: get_u32_if_available(&dict, "post_id"),
            vendor: get_u32_if_available(&dict, "vendor"),
            recipes: get_recipes(&dict)
        };
    }

    pub fn post_id(&self) -> &Option<u32> {
      return &self.post_id;
    }

    pub fn vendor(&self) -> &Option<u32> {
      return &self.vendor;
    }

    pub fn recipes(&self) -> &Vec<HashMap<String, u32>> {
      return &self.recipes;
    }
}
