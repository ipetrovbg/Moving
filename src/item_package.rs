use chrono::{DateTime, Utc};
use cursive::Cursive;
use cursive::traits::Nameable;
use cursive::views::{Dialog, EditView, ListView, SelectView, ScrollView, TextView};
use sqlite::{Connection};

use crate::package::PackageModule;
use crate::printer;
use std::rc::Rc;

pub struct ItemPackage<'a> {
    pub conn: Connection,
    pub layer: &'a mut Cursive,
}

impl ItemPackage<'_> {
    pub fn new(layer: &mut Cursive) -> ItemPackage {
        let conn = sqlite::open(dbg!("moving")).unwrap();
        let item = ItemPackage {
            conn,
            layer,
        };
        item
    }

    pub fn render_option(&mut self) {
        self.layer.add_layer(
            Dialog::new()
                .title("Select item operation")
                .content(
                    SelectView::new()
                        .item("Associate Item", 0)
                        .item("List Items", 1)
                        .item("Search Item", 2)
                        .item("Delete Item", 3)
                        .on_submit(|item_operation_layer, item| {
                            match *item {
                                0 => {
                                    let mut package_mod = PackageModule::new();
                                    let packages = package_mod.fetch_all_packages();
                                    item_operation_layer.add_layer(
                                        Dialog::new()
                                            .title("Select Package to create an associated item")
                                            .content(
                                                package_mod.render_in_select(&packages)
                                                    .on_submit(|ps, pkc_id| {
                                                        let pack_id = pkc_id.clone();
                                                        ItemPackage::create_item(pack_id, ps);
                                                    })
                                            )
                                            .dismiss_button("Cancel")
                                    )
                                }
                                1 => {
                                    let mut package_mod = PackageModule::new();
                                    let packages = package_mod.fetch_all_packages();
                                    item_operation_layer.add_layer(
                                        Dialog::new()
                                            .title("Select Package to list its Items")
                                            .content(
                                                package_mod.render_in_select(&packages)
                                                    .on_submit(|ps, pkc_id| {
                                                        let pack_id = pkc_id.clone();
                                                        ItemPackage::list_items_by_package_id(pack_id, ps);
                                                    })
                                            )
                                            .dismiss_button("Cancel")
                                    )
                                }
                                2 => {
                                    item_operation_layer.add_layer(
                                        Dialog::new()
                                            .title("Search Item by Name/Description")
                                            .content(
                                                ListView::new()
                                                    .child("Name/Description", EditView::new().with_name("name_description"))
                                            )
                                            .button("Search", |s| {
                                                let name_description = s.call_on_name("name_description", |d: &mut EditView| d.get_content()).unwrap();
                                                ItemPackage::search_item(name_description, s);
                                            })
                                            .dismiss_button("Cancel")
                                    )
                                }
                                3 => {
                                    let mut package_mod = PackageModule::new();
                                    let mut packages = package_mod.fetch_all_packages();
                                    item_operation_layer.add_layer(
                                        Dialog::new()
                                            .title("Select Package to list its Items for deletion")
                                            .content(
                                                package_mod.render_in_select(&mut packages)
                                                    .on_submit(|ps, pkc_id| {
                                                        let pack_id = pkc_id.clone();
                                                        ItemPackage::render_items_for_deletion(pack_id, ps);
                                                    })
                                            )
                                            .dismiss_button("Cancel")
                                    )
                                }
                                _ => item_operation_layer.add_layer(Dialog::info("No such operation"))
                            }
                        })
                ).dismiss_button("Close")
        )
    }

    fn search_item(name_description: Rc<String>, layer: &mut Cursive) {
        let conn = sqlite::open(dbg!("moving")).unwrap();
        let query = format!("
            SELECT i.id, i.packageId, i.name, i.description, i.createdAt, p.nil FROM Item as i
            INNER JOIN Package as p ON p.id = i.packageId
            WHERE i.name LIKE '%{0}%' OR i.description LIKE '%{0}%'
        ", name_description);
        let mut items: Vec<Item> = vec![];
        match conn.iterate(query, |pair| {
            Item::collect_items(pair, &mut items, true);
            true
        }) {
            Ok(_) => {
                let mut select = SelectView::new();
                for item in items {
                    select.add_item(
                        format!("#{} | {} | {} | {}",
                                item.id.unwrap(), item.nil, item.name, item.description),
                        item
                    )
                }
                let copy_select = select.on_submit(|s, item| {
                    let item_id = item.clone().id.unwrap().clone();
                    ItemPackage::confirm_delete_item(s, item_id);
                });
                let scroll = ScrollView::new(copy_select);
                layer.add_layer(
                    Dialog::new()
                        .title("Search results")
                        .content(scroll)
                        .dismiss_button("Close")
                )
            }
            Err(e) => {
                layer.add_layer(Dialog::info(format!("{}", e.message.unwrap())))
            }
        }
    }

    pub fn render_items_for_deletion(package_id: u32, layer: &mut Cursive) {
        let conn = sqlite::open(dbg!("moving")).unwrap();
        let query = format!("
            SELECT * FROM Item WHERE packageId = {}
        ", package_id.clone());
        let pkg = package_id.clone();
        let mut items: Vec<Item> = vec![];
        match conn.iterate(query, |pair|{
            Item::collect_items(pair, &mut items, false);
            true
        }) {
            Ok(_) => {
                let mut select = SelectView::new();
                let items_count = items.len();
                for item in items {
                    select.add_item(
                        Item::render_to_string(&item),
                        item.id.unwrap()
                    )
                }
                let copy_select = select.on_submit(|s, item| {
                    ItemPackage::confirm_delete_item(s, item.clone());
                });
                layer.add_layer(Dialog::around(ScrollView::new(copy_select))
                                .title("Select an Item")
                                .button("Print", move |c| {
                                    let package_mod = PackageModule::new();
                                    match package_mod.get_one(format!("{}", pkg.clone())) {
                                        Ok(p) => {
                                            match printer::print_package(p, items_count) {
                                                Ok(_) => {
                                                    c.pop_layer();
                                                }
                                                Err(_) => {
                                                    println!("Error");
                                                }
                                            };
                                        }
                                        Err(_) => {
                                            println!("Error");
                                        }
                                    };
                                })
                    .dismiss_button("Close")
                )
            }
            Err(e) => {
                layer.add_layer(Dialog::info(format!("{}", e.message.unwrap())))
            }
        }
    }

    fn confirm_delete_item(layer: &mut Cursive, item: u32) {
        let title = format!("Are you sure you want to delete Item #{}", item);
        let conn = sqlite::open(dbg!("moving")).unwrap();
        let select_item_query = format!("
            SELECT * FROM Item WHERE id = {} LIMIT 1
        ", item);
        let mut item_vec: Vec<Item> = vec![];
        match conn.iterate(select_item_query, |pair| {
            Item::collect_items(pair, &mut item_vec, false);
            true
        }) {
            Ok(_) => {}
            Err(_) => {}
        }
        let item_object = item_vec.get(0);
        layer.add_layer(
            Dialog::new()
                .title(title)
                .content(Item::render_item(&item_object.unwrap()))
                .button("Delete", move|s| {
                    let delete_query = format!("
                                    DELETE FROM Item WHERE id = {}
                                ", item);


                    match conn.execute(delete_query) {
                        Ok(_) => {
                            s.pop_layer();
                            s.pop_layer();
                            s.add_layer(
                                Dialog::new()
                                    .title("Confirmation")
                                    .content(TextView::new("Successfully delete Item"))
                                    .dismiss_button("Close")
                            )
                        }
                        Err(e) => {
                            s.add_layer(Dialog::info(format!("{}", e.message.unwrap())))
                        }
                    }

                })
                .dismiss_button("Cancel")
        );
    }

    pub fn list_items_by_package_id(package_id: u32, layer: &mut Cursive) {
        let conn = sqlite::open(dbg!("moving")).unwrap();
        let query = format!("
            SELECT * FROM Item WHERE packageId = {}
        ", package_id);
        let mut items: Vec<Item> = vec![];
        match conn.iterate(query, |pair|{
            Item::collect_items(pair, &mut items, false);
            true
        }) {
            Ok(_) => {
                let mut select = SelectView::new();
                for item in items {
                    select.add_item(format!("#{} | ", format!("{} | {}", item.name, item.description)),
                                             item.id.unwrap());
                }
                let scroll = ScrollView::new(select);
                layer.add_layer(
                    Dialog::new()
                        .title(format!("List of Items for Package #{}", package_id))
                        .content(scroll)
                        .dismiss_button("Close")
                );
            }
            Err(e) => {
                layer.add_layer(Dialog::info(format!("{}", e.message.unwrap())))
            }
        }
    }

    fn create_item(package_id: u32, layer: &mut Cursive) {
        let conn = sqlite::open(dbg!("moving")).unwrap();
        layer.add_layer(
            Dialog::new()
                .title(format!("Create item associated to Package #{}", package_id))
                .content(
                    ListView::new()
                        .child("Name", EditView::new().with_name("name"))
                        .child("Description", EditView::new().with_name("description"))
                )
                .button("Create", move |l| {
                    let name = l.call_on_name("name", |d: &mut EditView|
                        d.get_content()).unwrap();
                    let description = l.call_on_name("description", |d: &mut EditView|
                        d.get_content()).unwrap();
                    let created_at = Utc::now();
                    let query = format!("
                        INSERT INTO Item (packageId, name, description, createdAt)
                        VALUES
                        ('{}', '{}', '{}', '{}')
                    ", package_id, name, description, created_at);
                    match conn.execute(query) {
                        Ok(_) => {
                            l.pop_layer();
                            l.add_layer(Dialog::info(format!("Successfully associate \"{}\" to Package #{}", name, package_id)));
                        }
                        Err(e) => {
                            l.add_layer(Dialog::info(format!("{}", e.message.unwrap())));
                        }
                    }
                }).dismiss_button("Cancel")
        );
    }
}

#[allow(dead_code)]
struct Item {
    id: Option<u32>,
    package_id: u32,
    name: String,
    description: String,
    created_at: DateTime<Utc>,
    nil: String
}

impl Item {

    pub fn render_to_string(item: &Item) -> String {
        format!("#{} {} | {}", item.id.unwrap(), item.name, item.description)
    }

    pub fn render_item(item: &Item) -> TextView {
        TextView::new(format!("{} | {}", item.name, item.description))
    }

    pub fn collect_items(pair: &[(&str, Option<&str>)], packages: &mut Vec<Item>, has_nil: bool) {
        let id_value = pair.get(0).unwrap().1.unwrap();
        let package_id = pair.get(1).unwrap().1.unwrap();
        let name_value = pair.get(2).unwrap().1.unwrap();
        let description_value = pair.get(3).unwrap().1.unwrap();
        let created_at = pair.get(4).unwrap().1.unwrap();

        let date = match DateTime::parse_from_str(created_at,  "%Y %b %d %H:%M:%S%.3f %z") {
            Ok(d) => DateTime::from(d),
            Err(_) => Utc::now()
        };

        let nil = if has_nil {
            pair.get(5).unwrap().1.unwrap().to_string()
        } else {
            "N/A".to_string()
        };

        let package = Item {
            id: Some(id_value.parse::<u32>().unwrap()),
            package_id: package_id.parse::<u32>().unwrap(),
            name: name_value.to_string(),
            description: description_value.to_string(),
            created_at: date,
            nil
        };
        packages.push(package);
    }
}
