use std::collections::HashMap;
use std::collections::HashSet;

use restson::{RestClient, RestPath, Error};

use crate::descriptions::MaterialDescription;

#[derive(Serialize,Deserialize,Clone,Debug)]
pub struct Price {
    listings: u32,
    unit_price: u32,
    quantity: u32
}

impl Price {
    pub fn unit_price(&self) -> u32 {
        return self.unit_price;
    }

    pub fn quantity(&self) -> u32 {
        return self.quantity;
    }

    pub fn copy_reduced_by(&self, quantity: u32) -> Price {
        if quantity < self.quantity {
            return Price{
                listings: self.listings,
                unit_price: self.unit_price,
                quantity: self.quantity - quantity
            };
        } else {
            panic!(
                "Trying to reduce Price quantity {} by {}",
                self.quantity, quantity
            );
        }
    }
}

#[derive(Serialize,Deserialize,Clone,Debug)]
pub struct Item {
    id: u32,
    buys: Vec<Price>,
    sells: Vec<Price>
}

impl Item {
    pub fn id(&self) -> u32 {
        return self.id;
    }

    pub fn buys(&self) -> &Vec<Price> {
        return &self.buys;
    }

    pub fn sells(&self) -> &Vec<Price> {
        return &self.sells;
    }
}

struct ListingRequest {
    ids: Vec<u32>,
    dict: HashMap<u32, String>
}

impl ListingRequest {
    fn new(descriptions: &HashMap<String, MaterialDescription>) -> ListingRequest {
        let mut ids = Vec::<u32>::new();
        let mut dict = HashMap::<u32, String>::new();
        for (name, desc) in descriptions {
            if let Some(id) = desc.post_id() {
                ids.push(id.clone());
                dict.insert(id.clone(), name.clone());
            }
        }

        return ListingRequest{ids: ids, dict: dict};
    }
}

impl RestPath<&ListingRequest> for Vec<Item> {
    fn get_path(param: &ListingRequest) -> Result<String, Error> {
        let id_str: String =
            param.ids.iter()
            .map(|&id| id.to_string())
            .collect::<Vec<String>>().join(",");

        Ok(format!("v2/commerce/listings?ids={}", id_str))
    }
}

pub fn get_listings(descriptions: &HashMap<String, MaterialDescription>)
-> HashMap<String, Item> {
    let mut client = RestClient::new("https://api.guildwars2.com").unwrap();
    let request = ListingRequest::new(&descriptions);
    let response: Vec<Item> = client.get(&request).unwrap();

    let mut result = HashMap::<String, Item>::new();
    for item in response {
        result.insert(request.dict.get(&item.id()).unwrap().clone(), item.clone());
    }

    return result;
}

pub fn get_listings_for_targets(
    mut targets: Vec<String>,
    descriptions: &HashMap<String, MaterialDescription>
) -> HashMap<String, Item> {

    let mut visit = HashSet::<String>::new();

    let mut want_listings_for = HashMap::<String, MaterialDescription>::new();
    while let Some(next) = targets.pop() {

        if !visit.insert(next.clone()) {
            continue;
        }

        if let Some(description) = descriptions.get(&next) {
            for recipe in description.recipes() {
                for (item_name, _) in recipe {
                    targets.push(item_name.clone());
                }
            }

            if let Some(_) = description.post_id() {
                want_listings_for.insert(next, description.clone());
            }
        } else {
            panic!("Missing description for material [{}]", next);
        }
    }

    return get_listings(&want_listings_for);
}