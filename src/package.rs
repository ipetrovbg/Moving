use sqlite::{Connection};
use cursive::views::SelectView;
use nanoid::nanoid;
use chrono::{DateTime, Utc, NaiveDateTime};

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

    pub fn get_one(self, pkg_id: String) -> sqlite::Result<Package> {
        let conn = sqlite::open(format!("moving")).unwrap();
        let query = format!("SELECT * FROM Package WHERE id = '{}'", pkg_id);
        let mut packages: Vec<Package> = vec![];

        match conn.iterate(query.as_str(), |pairs| {
            PackageModule::collect_packages(pairs, &mut packages);
            true
        }) {
            Ok(_) => {}
            Err(_) => {}
        }


        let package = packages.first().unwrap();
        Ok(Package {
            nil: package.nil.clone(),
            id: package.id.clone(),
            name: package.name.clone(),
            description: package.description.clone(),
            created_at: package.created_at.clone()
        })
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
        let mut id_value = 0;
        let mut nil_value: String = "".to_string();
        let mut name_value: String = "".to_string();
        let mut description_value: String = "".to_string();
        let mut created_at_value: DateTime<Utc> = Utc::now();

        pair.iter().for_each(|(key, value)| {
            match key {
                &"id" => id_value = value.unwrap().parse().unwrap(),
                &"nil" => nil_value = value.unwrap().to_string(),
                &"name" => name_value = value.unwrap().to_string(),
                &"description" => description_value = value.unwrap().to_string(),
                &"createdAt" => {
                    let naive_date_time = NaiveDateTime::parse_from_str(value.unwrap(), "%Y-%m-%d %H:%M:%S%.f UTC");
                    match naive_date_time {
                        Ok(naive_date_time) => {
                            created_at_value = DateTime::from_utc(naive_date_time, Utc);
                        }
                        Err(_) => {}
                    }
                }
                _ => {}
            }

        });

        let package = Package {
            nil: nil_value,
            id: id_value,
            name: name_value,
            description: description_value,
            created_at: created_at_value,
        };
        packages.push(package);
    }
}

pub struct Package {
    pub nil: String,
    pub id: u32,
    pub name: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
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
