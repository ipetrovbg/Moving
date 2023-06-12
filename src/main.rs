mod item_package;
mod package;
mod printer;

use chrono::prelude::*;
use cursive::{Cursive, CursiveExt};
use cursive::traits::{Boxable, Identifiable};
use cursive::views::{Dialog, EditView, ListView, ScrollView, SelectView, TextView};
use sqlite::{Connection};
use crate::item_package::ItemPackage;
use crate::package::{Package, PackageModule};

const CANCEL_LABEL: &str = "Cancel";
const CLOSE_LABEL: &str = "Close";
const OK_LABEL: &str = "Ok";

fn main() {
    let mut siv = Cursive::new();
    let info = TextView::new("Some information about the software.");
    siv.add_layer(
        Dialog::new()
            .title("Moving Software")
            .content(
                Dialog::new()
                    .title("Home")
                    .content(
                        ListView::new()
                            .child("", info)
                            .child("Select operation", SelectView::new()
                                .item("Package", 0)
                                .item("Items", 1)
                                .on_submit(|s, item| {
                                    match *item {
                                        0 => {
                                            s.add_layer(
                                                Dialog::new()
                                                    .title("Select Package operation")
                                                    .content(
                                                        SelectView::new()
                                                            .item("Create Package", 1)
                                                            .item("Search Package", 2)
                                                            .item("Delete Package", 3)
                                                            .item("List All Packages", 4)
                                                            .on_submit(|s, item| {
                                                                match *item {
                                                                    1 => {
                                                                        create_package(s)
                                                                    }
                                                                    2 => {
                                                                        s.add_layer(
                                                                            Dialog::new()
                                                                                .title("Select searching by:")
                                                                                .content(
                                                                                    SelectView::new()
                                                                                        .item("Search by Name/Description", 0)
                                                                                        .item("Search by Nil", 1)
                                                                                        .item("Search by ID", 2)
                                                                                        .on_submit(|search_by, i| {
                                                                                            match *i {
                                                                                                0 => {
                                                                                                    search_package_by_name(search_by);
                                                                                                }
                                                                                                1 => {
                                                                                                    search_package_by_nil(search_by);
                                                                                                }
                                                                                                2 => {
                                                                                                    search_package_by_id(search_by);
                                                                                                }
                                                                                                _ => unreachable!("no such item"),
                                                                                            }
                                                                                        })
                                                                                ).dismiss_button(CLOSE_LABEL)
                                                                        );
                                                                    }
                                                                    3 => {
                                                                        delete_package(s)
                                                                    }
                                                                    4 => {
                                                                        list_all_packages(s);
                                                                    }
                                                                    _ => unreachable!("no such item"),
                                                                };
                                                            })
                                                    ).dismiss_button(CLOSE_LABEL)
                                            )
                                        }
                                        1 => {
                                            let mut item_package = ItemPackage::new(s);
                                            item_package.render_option();
                                        }
                                        _ => s.add_layer(Dialog::info("No such operation"))
                                    }
                                }))
                    )
            ).button("Quit", |q| q.quit())
            .full_width()
    );
    siv.run();
}

fn delete_package(s: &mut Cursive) {
    let mut package_mod = PackageModule::new();
    let packages = package_mod.fetch_all_packages();
    let select = package_mod.render_in_select(&packages);

    let select_copy = select.on_submit(|s, item| {
        let id: u32 = item.clone();
        let conn = sqlite::open(dbg!("moving")).unwrap();
        delete_confirmation_modal(conn, s, id);
    });
    let scroll = ScrollView::new(select_copy);
    s.add_layer(
        Dialog::new()
            .title("Delete Packages")
            .content(
                scroll
            ).dismiss_button(CLOSE_LABEL)
            .full_width()
    );
}

fn delete_confirmation_modal(conn: Connection, s: &mut Cursive, package_id: u32) {
    let mut packages: Vec<Package> = vec![];
    let count_items_in_package_query = format!("
        SELECT COUNT(*) FROM Item
        WHERE packageId = {}
    ", package_id);
    match conn.iterate(count_items_in_package_query, |pair| {
        let count = pair.get(0).unwrap().1.unwrap().parse::<u32>();
        match count {
            Ok(c) => {
                match c {
                    0 => {
                        // Save to delete this package
                        let conn = sqlite::open(dbg!("moving")).unwrap();
                        conn
                            .iterate(format!("
                                SELECT * FROM Package WHERE id = {} LIMIT 1
                            ", package_id), |pairs| {
                                PackageModule::collect_packages(pairs, &mut packages);
                                true
                            }).unwrap();
                        let pkg = packages.get(0).unwrap();
                        s.add_layer(
                            Dialog::new()
                                .title("Do you want to delete this Package?")
                                .content(TextView::new(format!("#{} | {} | {}", pkg.id, pkg.nil, pkg.name)))
                                .button("Delete", move |d| {
                                    conn
                                        .execute(format!("DELETE FROM Package WHERE id = {};", package_id)).unwrap();

                                    d.pop_layer();
                                    d.pop_layer();
                                    d.add_layer(Dialog::info("Package deleted successfully!"))
                                }).dismiss_button(CANCEL_LABEL)
                        )
                    },
                    _ => {
                        s.add_layer(
                            Dialog::new()
                                .title("Information")
                                .content(TextView::new("Package has associated Items. Delete them first."))
                            .dismiss_button("Close")
                        )
                    }
                }
            }
            Err(_) => {}
        }
        true
    }) {
        Ok(_) => {

        }
        Err(_) => {

        }
    }
}

fn create_package(s: &mut Cursive) {
    let dialog = Dialog::new()
        .title("Create Package")
        .content(
            ListView::new()
                .child("Name:", EditView::new().with_name("name"))
                .child("Description:", EditView::new().with_name("description"))
        ).button("Create", |cb| {
        match sqlite::open(dbg!("moving")) {
            Ok(conn) => {
                let name = cb.call_on_name("name", |d: &mut EditView| d.get_content()).unwrap().to_string();
                let description = cb.call_on_name("description", |d: &mut EditView| d.get_content()).unwrap().to_string();
                let nil = Package::generate_nil();
                let nil_check_query = format!("SELECT COUNT(*) FROM Package WHERE nil = {}", &nil);
                conn.iterate(nil_check_query, |pairs_count| {
                    let c = pairs_count.get(0).unwrap().1.unwrap();
                    return if c.parse::<u32>().unwrap() == 0 {
                        cb.pop_layer();
                        match PackageModule::create(
                            &conn,
                            name.clone(),
                            description.clone(),
                            nil.clone(),  Utc::now()) {
                            Ok(_) => {
                                cb.add_layer(
                                    Dialog::new()
                                        .title("Package created successfully!")
                                        .content(
                                            display_last_package(&conn)
                                        )
                                        .dismiss_button(OK_LABEL)
                                );
                            }
                            Err(e) => {
                                match e.code {
                                    Some(code) => cb.add_layer(
                                        Dialog::info(format!("DB Error code {}", code))
                                            .dismiss_button(CANCEL_LABEL)
                                    ),
                                    None => println!("No error code")
                                }
                                match e.message {
                                    Some(message) => cb.add_layer(
                                        Dialog::info(format!("DB Error {}", message))
                                            .dismiss_button(CANCEL_LABEL)
                                    ),
                                    None => println!("Unknown error")
                                }
                                cb.add_layer(
                                    Dialog::info("DB Error")
                                        .dismiss_button(CANCEL_LABEL)
                                );
                            }
                        }

                        true
                    } else {
                        cb.add_layer(Dialog::info(format!("Package with Nil: \"{}\" already exists.", &nil)));
                        true
                    };
                }).unwrap();
            }
            Err(_) => {
                cb.add_layer(Dialog::info("Error"));
            }
        }


    })
        .dismiss_button(CANCEL_LABEL)
        .full_width();

    s.add_layer(
        dialog
    );
}

fn display_last_package(conn: &Connection) -> ListView {
    let query_last_id = "SELECT last_insert_rowid();";
    let mut packages: Vec<Package> = vec![];
    match conn.iterate(query_last_id, |pairs| {
        let id = pairs.get(0).unwrap().1.unwrap();
        let query = format!("SELECT * FROM Package WHERE id = {}", id);
        match conn.iterate(query, |package_pair| {
            PackageModule::collect_packages(package_pair, &mut packages);
            true
        }) {
            Ok(_) => {}
            Err(_) => {}
        };
        true
    }) {
        Ok(_) => {}
        Err(_) => {}
    };

    render_packages_in_list(packages)
}

fn search_package_by_name(s: &mut Cursive) {
    s.add_layer(
        Dialog::new()
            .title("Search Package by Name/Description")
            .content(
                ListView::new()
                    .child("Name/Description", EditView::new().with_name("name"))
            )
            .button("Search", |d| {
            let conn = sqlite::open(dbg!("moving")).unwrap();

            let name = d.call_on_name("name", |e: &mut EditView| e.get_content()).unwrap();

            let query = format!("SELECT * FROM Package WHERE name LIKE '%{0}%' OR description LIKE '%{0}%'", name);
            let count_query = format!("SELECT COUNT(*) as count FROM Package WHERE name LIKE '%{0}%' OR description LIKE '%{0}%'", name);
            conn.iterate(count_query, |pairs| {
                let c = pairs.get(0).unwrap().1.unwrap();
                if c == "0" {
                    d.add_layer(Dialog::info("Nothing Found!"))
                } else {
                    let mut packages: Vec<Package> = vec![];
                    conn.iterate(&query, |pairs| {
                        PackageModule::collect_packages(pairs, &mut packages);
                        true
                    }).unwrap();

                    let package_module = PackageModule::new();
                    let select = package_module.render_in_select(&packages);
                    let copy_select = select.on_submit(|s, item| {
                        ItemPackage::render_items_for_deletion(item.clone(), s);
                    });
                    let scroll_view = ScrollView::new(copy_select);
                    d.add_layer(
                        Dialog::new()
                            .title(format!("Search Results for \"{}\"", &name))
                            .content(
                                scroll_view
                            ).dismiss_button(CLOSE_LABEL)
                            .full_width()
                    );
                }
                true
            }).unwrap();
        })
            .dismiss_button(CANCEL_LABEL)
    );
}
fn search_package_by_id(s: &mut Cursive) {
    s.add_layer(
        Dialog::new()
            .title("Search Package by ID")
            .content(
                ListView::new()
                    .child("ID", EditView::new().with_name("id"))
            ).button("Search", |d| {
            let conn = sqlite::open(dbg!("moving")).unwrap();

            let id = d.call_on_name("id", |e: &mut EditView| e.get_content()).unwrap();
            if id.is_empty() {
                d.add_layer(Dialog::info("ID number cannot be empty."));
                return;
            }
            let query = format!("SELECT * FROM Package WHERE id = {} LIMIT 1", id);
            let count_query = format!("SELECT COUNT(*) as count FROM Package WHERE id = {} LIMIT 1", id);
            conn.iterate(count_query, |pairs| {
                let c = pairs.get(0).unwrap().1.unwrap();
                if c == "0" {
                    d.add_layer(Dialog::info("Nothing Found!"))
                } else {
                    let mut packages: Vec<Package> = vec![];
                    conn.iterate(&query, |pairs| {
                        PackageModule::collect_packages(pairs, &mut packages);
                        true
                    }).unwrap();

                    let package_module = PackageModule::new();
                    let select = package_module.render_in_select(&packages);
                    let copy_select = select.on_submit(|s, item| {
                        ItemPackage::render_items_for_deletion(item.clone(), s);
                    });
                    let scroll_view = ScrollView::new(copy_select);
                    d.add_layer(
                        Dialog::new()
                            .title(format!("Search Results for \"{}\"", &id))
                            .content(
                                scroll_view
                            ).dismiss_button(CLOSE_LABEL)
                            .full_width()
                    );
                }
                true
            }).unwrap();
        })
            .dismiss_button(CLOSE_LABEL)
    );
}

fn search_package_by_nil(s: &mut Cursive) {
    s.add_layer(
        Dialog::new()
            .title("Search Package")
            .content(
                ListView::new()
                    .child("Nil", EditView::new().with_name("nil"))
            ).button("Search", |d| {
            let conn = sqlite::open(dbg!("moving")).unwrap();

            let nil = d.call_on_name("nil", |e: &mut EditView| e.get_content()).unwrap();
            if nil.is_empty() {
                d.add_layer(Dialog::info("Nil number cannot be empty."));
                return;
            }
            let query = format!("SELECT * FROM Package WHERE nil LIKE '%{}%'", nil);
            let count_query = format!("SELECT COUNT(*) as count FROM Package WHERE nil LIKE '%{}%'", nil);
            conn.iterate(count_query, |pairs| {
                let c = pairs.get(0).unwrap().1.unwrap();
                if c == "0" {
                    d.add_layer(Dialog::info("Nothing Found!"))
                } else {
                    let mut packages: Vec<Package> = vec![];
                    conn.iterate(&query, |pairs| {
                        PackageModule::collect_packages(pairs, &mut packages);
                        true
                    }).unwrap();

                    let package_module = PackageModule::new();
                    let select = package_module.render_in_select(&packages);
                    let copy_select = select.on_submit(|s, item| {
                        ItemPackage::render_items_for_deletion(item.clone(), s);
                    });
                    let scroll_view = ScrollView::new(copy_select);
                    d.add_layer(
                        Dialog::new()
                            .title(format!("Search Results for \"{}\"", &nil))
                            .content(
                                scroll_view
                            ).dismiss_button(CLOSE_LABEL)
                            .full_width()
                    );
                }
                true
            }).unwrap();
        })
            .dismiss_button(CLOSE_LABEL)
    );
}

fn render_packages_in_list(packages: Vec<Package>) -> ListView {
    let mut list = ListView::new();
    for package in packages {
        list.add_child(format!("#{} | ",
                               package.id.to_string()).as_str(),
                       TextView::new(format!("{} | {} | {}", package.nil, package.name, package.description)))
    }
    list
}

fn list_all_packages(s: &mut Cursive) {
    let package_mod = PackageModule::new();
    let query = "SELECT * FROM Package";
    let mut packages: Vec<Package> = vec![];

    match package_mod.conn.iterate(query, |pairs| {
        PackageModule::collect_packages(pairs, &mut packages);
        true
    }) {
        Ok(_) => {
            let select = package_mod.render_in_select(&packages);
            let copy_select = select.on_submit(|s, p_id| {
                ItemPackage::render_items_for_deletion(p_id.clone(), s);
            });
            let scroll = ScrollView::new(copy_select);
            s.add_layer(
                Dialog::new()
                    .title("List of All Packages")
                    .content(
                        scroll
                    ).dismiss_button(CLOSE_LABEL)
                    .full_width()
            );
        }
        Err(e) => {
            match e.message {
                Some(message) => println!("{}", message),
                None => println!("No error message")
            }
            println!("Error listing packages");
        }
    }
}


