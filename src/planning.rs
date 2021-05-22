use crate::descriptions::MaterialDescription;
use crate::tp_rest::{Item, Price};

use std::rc::Rc;
use std::collections::{HashMap, VecDeque};
use std::cmp::Reverse;
use std::iter::FromIterator;
use std::ops::Add;

use priority_queue::PriorityQueue;
use by_address::ByAddress;

#[derive(Debug)]
pub struct Craft {
    recipe: HashMap<String, u32>
}

#[derive(Clone, Debug)]
pub struct Buy {
    name: String,
    quantity: u32,
    cost: u32
}

impl<'a, 'b> Add<&'b Buy> for &'a Buy {
    type Output = Buy;

    fn add(self, other: &'b Buy) -> Buy {
        if self.name != other.name {
            panic!(
                "Name mismatch while adding buys: [{}] vs [{}]",
                self.name, other.name
            );
        }

        Buy{
            name: self.name.clone(),
            quantity: self.quantity + other.quantity,
            cost: self.cost + other.cost
        }
    }
}

#[derive(Clone, Debug)]
pub struct Vendor {
    name: String,
    quantity: u32,
    cost: u32
}

impl<'a, 'b> Add<&'b Vendor> for &'a Vendor {
    type Output = Vendor;

    fn add(self, other: &'b Vendor) -> Vendor {
        if self.name != other.name {
            panic!(
                "Name mismatch while adding vendors: [{}] vs [{}]",
                self.name, other.name
            );
        }

        Vendor{
            name: self.name.clone(),
            quantity: self.quantity + other.quantity,
            cost: self.cost + other.cost
        }
    }
}

#[derive(Debug)]
pub enum Choice {
    Craft(Craft),
    Buy(Buy),
    Vendor(Vendor)
}

#[derive(Debug)]
struct Parent {
    node: Rc<SearchNode>,
    choice: Choice
}

#[derive(Clone,Debug)]
struct Remainder {
    name: String,
    quantity: u32
}

#[derive(Debug)]
struct SearchNode {
    parent: Option<Parent>,
    remaining: Vec<Remainder>,
    listings: HashMap<String, VecDeque<Price>>,
    descriptions: Rc<HashMap<String, MaterialDescription>>,
    total_cost: u32
}

type SearchNodePtr = ByAddress<Rc<SearchNode>>;
type SearchQueue = PriorityQueue<SearchNodePtr, Reverse<u32>>;

impl SearchNode {
    fn expand(
        parent: &Rc<Self>,
        queue: &mut SearchQueue
    ) -> bool {
        if parent.as_ref().remaining.is_empty() {
            return false;
        }

        let mut expansions = SearchQueue::new();
        SearchNode::expand_buy(parent, &mut expansions);
        SearchNode::expand_vendor(parent, &mut expansions);
        SearchNode::expand_recipes(parent, &mut expansions);

        if expansions.is_empty() {
            panic!(
                "Failed to expand {}",
                parent.as_ref().remaining.get(0).unwrap().name
            );
        }

        queue.append(&mut expansions);
        return true;
    }

    fn expand_buy(
        parent: &Rc<Self>,
        queue: &mut SearchQueue
    ) {
        let p = parent.as_ref();
        let mut remaining = p.remaining.clone();

        if let Some(next) = remaining.pop() {
            if let Some(_) = p.listings.get(&next.name) {
                let mut new_listings = p.listings.clone();
                let prices: &mut VecDeque<Price> =
                    new_listings.get_mut(&next.name).unwrap();

                let mut quantity = next.quantity;
                if let Some(next_price) = prices.pop_front() {
                    if next.quantity < next_price.quantity() {
                        prices.push_front(
                            next_price.copy_reduced_by(next.quantity)
                        );
                    } else if next_price.quantity() < next.quantity {
                        quantity = next_price.quantity();
                        remaining.push(
                            Remainder{
                                name: next.name.clone(),
                                quantity: next.quantity - next_price.quantity()
                            }
                        )
                    }

                    let added_cost = quantity * next_price.unit_price();
                    let new_cost = p.total_cost + added_cost;
                    queue.push(
                        ByAddress(Rc::new(
                            SearchNode{
                                parent: Some(
                                    Parent{
                                        node: parent.clone(),
                                        choice: Choice::Buy(Buy{
                                            name: next.name.clone(),
                                            quantity: quantity,
                                            cost: added_cost
                                        }),
                                    }
                                ),
                                remaining: remaining,
                                listings: new_listings,
                                descriptions: p.descriptions.clone(),
                                total_cost: new_cost
                            }
                        )),
                        Reverse(new_cost)
                    );
                }
            }
        }
    }

    fn expand_vendor(
        parent: &Rc<Self>,
        queue: &mut SearchQueue
    ) {
        let p = parent.as_ref();
        let mut remaining = p.remaining.clone();

        if let Some(next) = remaining.pop() {
            if let Some(description) = p.descriptions.as_ref().get(&next.name) {
                if let Some(vendor_price) = description.vendor() {
                    let added_cost = next.quantity * vendor_price;
                    let new_cost = p.total_cost + added_cost;
                    queue.push(
                        ByAddress(Rc::new(
                            SearchNode{
                                parent: Some(
                                    Parent{
                                        node: parent.clone(),
                                        choice: Choice::Vendor(Vendor{
                                            name: next.name.clone(),
                                            quantity: next.quantity,
                                            cost: added_cost
                                        }),
                                    }
                                ),
                                remaining: remaining,
                                listings: p.listings.clone(),
                                descriptions: p.descriptions.clone(),
                                total_cost: new_cost
                            }
                        )),
                        Reverse(new_cost)
                    );
                }
            } else {
                panic!(
                    "[SearchNode::expand_vendor] Missing description for [{}]",
                    next.name
                );
            }
        }
    }

    fn expand_recipes(
        parent: &Rc<Self>,
        queue: &mut SearchQueue
    ) {
        let p = parent.as_ref();
        let mut remaining = p.remaining.clone();

        if let Some(next) = remaining.pop() {
            if let Some(description) = p.descriptions.as_ref().get(&next.name) {
                for recipe in description.recipes() {
                    let mut remaining_with_recipe = remaining.clone();
                    for (name, quantity) in recipe {
                        remaining_with_recipe.push(
                            Remainder{
                                name: name.clone(),
                                quantity: quantity.clone() * next.quantity
                            }
                        );
                    }

                    queue.push(
                        ByAddress(Rc::new(
                            SearchNode{
                                parent: Some(
                                    Parent{
                                        node: parent.clone(),
                                        choice: Choice::Craft(
                                            Craft{
                                                recipe: recipe.clone()
                                            }
                                        )
                                    }
                                ),
                                remaining: remaining_with_recipe,
                                listings: p.listings.clone(),
                                descriptions: p.descriptions.clone(),
                                total_cost: p.total_cost
                            }
                        )),
                        Reverse(p.total_cost)
                    );
                }
            } else {
                panic!(
                    "[SearchNode::expand_recipes] Missing description for [{}]",
                    next.name
                );
            }
        }
    }
}

#[derive(Debug)]
struct Result {
    buy: HashMap<String, Buy>,
    vendor: HashMap<String, Vendor>
}

fn flatten(solution: &Rc<SearchNode>) -> Result {
    let mut result = Result{
        buy: HashMap::<String, Buy>::new(),
        vendor: HashMap::<String, Vendor>::new()
    };

    let mut next = solution.clone();
    while let Some(parent) = &next.parent {
        match &parent.choice {
            Choice::Craft(_) => { },
            Choice::Buy(buy) => {
                if let Some(entry) = result.buy.get_mut(&buy.name) {
                    // println!(
                    //     "[Buy] Adding to {}: {} x {}",
                    //     buy.name, buy.quantity, buy.cost
                    // );
                    *entry = &*entry + buy;
                } else {
                    // println!(
                    //     "[Buy] Setting for {}: {} x {}",
                    //     buy.name, buy.quantity, buy.cost
                    // );
                    result.buy.insert(buy.name.clone(), buy.clone());
                }
            },
            Choice::Vendor(vendor) => {
                if let Some(entry) = result.vendor.get_mut(&vendor.name) {
                    // println!(
                    //     "[Vendor] Adding to {}: {} x {}",
                    //     vendor.name, vendor.quantity, vendor.cost
                    // );
                    *entry = &*entry + vendor;
                } else {
                    // println!(
                    //     "[Vendor] Setting for {}: {} x {}",
                    //     vendor.name, vendor.quantity, vendor.cost
                    // );
                    result.vendor.insert(vendor.name.clone(), vendor.clone());
                }
            }
        }

        next = parent.node.clone();
    }

    return result;
}

pub fn plan(
    targets: &Vec<String>,
    descriptions: HashMap<String, MaterialDescription>,
    listings: &HashMap<String, Item>
) -> bool {

    let initial_remaining = targets.iter()
        .map(|name| Remainder{ name: name.clone(), quantity: 1 })
        .collect();

    let initial_listings: HashMap<String, VecDeque<Price>> = listings.iter()
        .map(|(name, item)| (
            name.clone(),
            VecDeque::from_iter(item.sells().clone().into_iter())
        ))
        .collect();

    let mut queue = SearchQueue::new();
    queue.push(
        ByAddress(Rc::new(
            SearchNode{
                parent: None,
                remaining: initial_remaining,
                listings: initial_listings,
                descriptions: Rc::new(descriptions),
                total_cost: 0
            }
        )),
        Reverse(0));

    while let Some((next, _)) = queue.pop() {
        if !SearchNode::expand(&next, &mut queue) {
            println!("\n\n____ Plan Result ____");
            let result = flatten(&next);

            println!("\nFrom Trading Post, buy:");
            for (_, buy) in &result.buy {
                println!(
                    "{}: {} for a total cost of {}",
                    buy.name, buy.quantity, buy.cost
                );
            }

            println!("\nFrom vendors, buy:");
            for (_, vendor) in &result.vendor {
                println!(
                    "{}: {} for a total cost of {}",
                    vendor.name, vendor.quantity, vendor.cost
                );
            }
            break;
        }
    }

    return false;
}

pub fn plan_isolated(
    target: &String,
    descriptions: HashMap<String, MaterialDescription>,
    listings: &HashMap<String, Item>
) -> bool {
    let mut targets = Vec::<String>::new();
    targets.push(target.clone());
    return plan(&targets, descriptions, listings);
}