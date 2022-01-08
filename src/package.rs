use sqlite::{Connection};
use cursive::views::SelectView;
use nanoid::nanoid;
use chrono::{DateTime, Utc};

pub struct PackageModule {
    pub conn: Connection
}

impl PackageModule {

    pub fn new() -> PackageModule {
        let conn = sqlite::open(dbg!("moving")).unwrap();
        PackageModule {
            conn,
        }
    }

    pub fn create(conn: &Connection, name: String, description: String, nil: String, date: DateTime<Utc>) -> sqlite::Result<()> {
        conn.execute(format!("
                            INSERT INTO Package (nil, name, description, createdAt)
                            VALUES
                            ('{}', '{}', '{}', '{}');
                            ", nil, name, description, date)
        )
    }

    pub fn render_in_select(self, packages: &Vec<Package>) -> SelectView<u32> {
        let mut select = SelectView::new();
        for pkg in packages {
            select.add_item(format!("#{} | {} | {} | {}", pkg.id, pkg.nil, pkg.name, pkg.description), pkg.id)
        }
        select
    }

    pub fn fetch_all_packages(&mut self) -> Vec<Package> {
        let query = "SELECT * FROM Package";
        let mut packages: Vec<Package> = vec![];

        match self.conn.iterate(query, |pairs| {
            PackageModule::collect_packages(pairs, &mut packages);
            true
        }) {
            Ok(_) => {}
            Err(_) => {}
        }
        packages
    }

    pub fn collect_packages(pair: &[(&str, Option<&str>)], packages: &mut Vec<Package>) {
        let id_value = pair.get(0).unwrap().1.unwrap();
        let nil_value = pair.get(1).unwrap().1.unwrap();
        let name_value = pair.get(2).unwrap().1.unwrap();
        let description_value = pair.get(3).unwrap().1.unwrap();
        let package = Package {
            nil: nil_value.to_string(),
            id: id_value.parse().unwrap(),
            name: name_value.to_string(),
            description: description_value.to_string(),
        };
        packages.push(package);
    }
}

pub struct Package {
    pub nil: String,
    pub id: u32,
    pub name: String,
    pub description: String,
}

impl Package {
    pub fn generate_nil() -> String {
        let alphabet: [char; 10] = [
            '1', '2', '3', '4', '5', '6', '7', '8', '9', '0',
        ];

        let first_id = nanoid!(3, &alphabet);
        let second_id = nanoid!(3, &alphabet);
        first_id + "-" + second_id.as_str()
    }
}
